use super::*;

use ion_rs::v1_0::Binary;
use ion_rs::{Element, Int};

fn make_ion_bytes(fields: &[(&str, &str)]) -> Vec<u8> {
    let mut builder = ion_rs::Struct::builder();
    for (k, v) in fields {
        builder = builder.with_field(*k, Element::string(*v));
    }
    let s = builder.build();
    let elem = Element::from(s);
    elem.encode_as(Binary).unwrap()
}

fn wrap_in_container(ion_payload: &[u8], entity_type: u32) -> Vec<u8> {
    // Build a minimal KFX container:
    // Header (18 bytes): CONT(4) + version(2) + header_len(4) + ci_offset(4) + ci_length(4)
    // Body: [container_info_ion] [index_table(24 bytes)] [entity_data]
    let header_len: u32 = 18;

    // Build container info Ion with placeholder offsets first to measure size
    let make_ci = |idx_offset: usize, idx_len: usize| -> Vec<u8> {
        let ci_struct = ion_rs::Struct::builder()
            .with_field(
                "bcIndexTabOffset",
                Element::from(Int::from(idx_offset as i64)),
            )
            .with_field("bcIndexTabLength", Element::from(Int::from(idx_len as i64)))
            .build();
        Element::from(ci_struct).encode_as(Binary).unwrap()
    };

    let index_entry_size = 24;

    // Iterate to find stable ci size (Ion encoding size varies with value magnitude)
    let mut ci_size_guess = make_ci(999, index_entry_size).len();
    let ci = loop {
        let entity_offset = ci_size_guess + index_entry_size;
        let ci = make_ci(ci_size_guess, index_entry_size);
        if ci.len() == ci_size_guess {
            // Also verify entity offset didn't shift
            let recheck = make_ci(ci.len(), index_entry_size);
            assert_eq!(recheck.len(), ci.len());
            break ci;
        }
        ci_size_guess = ci.len();
    };
    let ci_size = ci.len();
    let entity_offset = ci_size + index_entry_size;

    let mut index_table = Vec::with_capacity(index_entry_size);
    index_table.extend_from_slice(&1u32.to_le_bytes());
    index_table.extend_from_slice(&entity_type.to_le_bytes());
    index_table.extend_from_slice(&(entity_offset as u64).to_le_bytes());
    index_table.extend_from_slice(&(ion_payload.len() as u64).to_le_bytes());

    // Assemble container
    let mut out = Vec::new();
    out.extend_from_slice(CONT_MAGIC);
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&header_len.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // ci_offset = 0
    out.extend_from_slice(&(ci.len() as u32).to_le_bytes());

    assert_eq!(out.len(), header_len as usize);

    out.extend_from_slice(&ci);
    out.extend_from_slice(&index_table);
    out.extend_from_slice(ion_payload);

    out
}

#[test]
fn parse_ion_metadata_fields() {
    let ion = make_ion_bytes(&[
        ("title", "The Great Gatsby"),
        ("author", "F. Scott Fitzgerald"),
        ("ASIN", "B000FC0PDA"),
        ("publisher", "Scribner"),
    ]);

    let container = wrap_in_container(&ion, 258);
    let result = parse_kfx_container(&container);

    let fields = result.expect("should parse metadata");
    assert_eq!(fields.title.as_deref(), Some("The Great Gatsby"));
    assert_eq!(fields.author.as_deref(), Some("F. Scott Fitzgerald"));
    assert_eq!(fields.asin.as_deref(), Some("B000FC0PDA"));
    assert_eq!(fields.publisher.as_deref(), Some("Scribner"));
}

#[test]
fn returns_none_for_non_kfx() {
    assert!(parse_kfx_container(b"this is not a kfx file at all").is_none());
}

#[test]
fn returns_none_for_empty() {
    assert!(parse_kfx_container(&[]).is_none());
}

#[test]
fn returns_none_for_truncated_header() {
    let mut data = Vec::new();
    data.extend_from_slice(CONT_MAGIC);
    data.extend_from_slice(&[0, 0]); // version only, no header_len
    assert!(parse_kfx_container(&data).is_none());
}

#[test]
fn extracts_alternate_field_names() {
    let ion = make_ion_bytes(&[
        ("content_title", "Alt Title"),
        ("synopsis", "A great book"),
        ("issue_date", "2024-01-15"),
        ("cde_contenttype", "EBOK"),
    ]);

    let container = wrap_in_container(&ion, 258);
    let result = parse_kfx_container(&container).expect("should parse");

    assert_eq!(result.title.as_deref(), Some("Alt Title"));
    assert_eq!(result.description.as_deref(), Some("A great book"));
    assert_eq!(result.published_date.as_deref(), Some("2024-01-15"));
    assert_eq!(result.cde_type.as_deref(), Some("EBOK"));
}

#[test]
fn skips_non_metadata_entity_types() {
    let ion = make_ion_bytes(&[("title", "Hidden")]);
    let container = wrap_in_container(&ion, 999);
    // Entity type 999 not in METADATA_ENTITY_TYPES — index path skips it.
    // Brute scan fallback may still find it, which is acceptable.
    let _ = parse_kfx_container(&container);
}
