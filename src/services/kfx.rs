use std::fs;
use std::path::Path;

use ion_rs::Element;
use tracing::debug;

#[cfg(test)]
#[path = "kfx_tests.rs"]
mod tests;

const CONT_MAGIC: &[u8; 4] = b"CONT";
const ENTY_MAGIC: &[u8; 4] = b"ENTY";
const ION_MAGIC: &[u8; 4] = &[0xe0, 0x01, 0x00, 0xea];

/// Metadata entity type IDs that carry book metadata in KFX containers.
const METADATA_ENTITY_TYPES: &[u32] = &[164, 258, 417, 490];

pub struct KfxFields {
    pub title: Option<String>,
    pub author: Option<String>,
    pub asin: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub cover_data: Option<String>,
    pub cde_type: Option<String>,
}

pub fn read_kfx_metadata(path: &Path) -> Option<KfxFields> {
    let data = fs::read(path).ok()?;
    parse_kfx_container(&data)
}

fn parse_kfx_container(data: &[u8]) -> Option<KfxFields> {
    if data.len() < 10 || &data[0..4] != CONT_MAGIC {
        return None;
    }

    // Container header: magic(4) + version(2) + header_len(4)
    let header_len = u32::from_le_bytes(data[6..10].try_into().ok()?) as usize;
    if header_len > data.len() {
        return None;
    }

    // After the base header (10 bytes), read container_info_offset and length
    if data.len() < 18 {
        return None;
    }
    let container_info_offset = u32::from_le_bytes(data[10..14].try_into().ok()?) as usize;
    let container_info_length = u32::from_le_bytes(data[14..18].try_into().ok()?) as usize;

    // Parse the container info Ion data to get index table location
    let ci_start = header_len + container_info_offset;
    let ci_end = ci_start + container_info_length;
    if ci_end > data.len() {
        return None;
    }
    let container_info = &data[ci_start..ci_end];

    let (index_offset, index_length) = parse_container_info_ion(container_info)?;

    let idx_start = header_len + index_offset;
    let idx_end = idx_start + index_length;
    if idx_end > data.len() {
        return None;
    }
    let index_data = &data[idx_start..idx_end];

    // Each index entry: entity_id(4) + entity_type(4) + offset(8) + length(8) = 24 bytes
    let entry_size = 24;
    let entry_count = index_data.len() / entry_size;

    let mut fields = KfxFields {
        title: None,
        author: None,
        asin: None,
        isbn: None,
        publisher: None,
        description: None,
        published_date: None,
        language: None,
        cover_data: None,
        cde_type: None,
    };

    for i in 0..entry_count {
        let base = i * entry_size;
        if base + entry_size > index_data.len() {
            break;
        }
        let entity_type = u32::from_le_bytes(index_data[base + 4..base + 8].try_into().ok()?);
        let entity_offset =
            u64::from_le_bytes(index_data[base + 8..base + 16].try_into().ok()?) as usize;
        let entity_len =
            u64::from_le_bytes(index_data[base + 16..base + 24].try_into().ok()?) as usize;

        if !METADATA_ENTITY_TYPES.contains(&entity_type) {
            continue;
        }

        let ent_start = header_len + entity_offset;
        let ent_end = ent_start + entity_len;
        if ent_end > data.len() {
            continue;
        }

        let entity_data = &data[ent_start..ent_end];
        let ion_payload = extract_ion_from_entity(entity_data);

        if let Some(ion_data) = ion_payload {
            extract_fields_from_ion(ion_data, &mut fields);
        }
    }

    // If we found nothing at all via the index table, try a brute-force Ion scan
    if fields.title.is_none() && fields.author.is_none() && fields.asin.is_none() {
        brute_scan_for_ion(data, &mut fields);
    }

    if fields.title.is_some() || fields.author.is_some() || fields.asin.is_some() {
        Some(fields)
    } else {
        None
    }
}

/// Parse container info Ion payload to find bcIndexTabOffset and bcIndexTabLength.
fn parse_container_info_ion(data: &[u8]) -> Option<(usize, usize)> {
    let ion_start = find_ion_start(data)?;
    let ion_data = &data[ion_start..];

    let elements = Element::read_all(ion_data).ok()?;
    for elem in elements.iter() {
        let s = elem.as_struct()?;
        let offset = s
            .get("bcIndexTabOffset")
            .and_then(|e| e.as_i64())
            .map(|v| v as usize);
        let length = s
            .get("bcIndexTabLength")
            .and_then(|e| e.as_i64())
            .map(|v| v as usize);
        if let (Some(o), Some(l)) = (offset, length) {
            return Some((o, l));
        }
    }
    None
}

/// Extract Ion payload from an entity block. Entity may have ENTY header or raw Ion.
fn extract_ion_from_entity(data: &[u8]) -> Option<&[u8]> {
    if data.len() >= 4 && &data[0..4] == ENTY_MAGIC {
        // ENTY header: magic(4) + version(2) + header_len(4)
        if data.len() < 10 {
            return None;
        }
        let ent_header_len = u32::from_le_bytes(data[6..10].try_into().ok()?) as usize;
        if ent_header_len <= data.len() {
            let payload = &data[ent_header_len..];
            if let Some(start) = find_ion_start(payload) {
                return Some(&payload[start..]);
            }
            return Some(payload);
        }
        return None;
    }

    // Raw data — look for Ion magic
    if let Some(start) = find_ion_start(data) {
        return Some(&data[start..]);
    }

    // Maybe the data itself is Ion without the magic (unlikely but handle gracefully)
    None
}

fn find_ion_start(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|w| w == ION_MAGIC)
}

fn extract_fields_from_ion(ion_data: &[u8], fields: &mut KfxFields) {
    let Ok(elements) = Element::read_all(ion_data) else {
        return;
    };

    for elem in elements.iter() {
        extract_from_element(elem, fields);
    }
}

fn extract_from_element(elem: &Element, fields: &mut KfxFields) {
    if let Some(s) = elem.as_struct() {
        // Direct metadata fields
        if fields.title.is_none() {
            fields.title =
                get_string_field(s, "title").or_else(|| get_string_field(s, "content_title"));
        }
        if fields.author.is_none() {
            fields.author = get_string_field(s, "author")
                .or_else(|| get_list_as_string(s, "authors"))
                .or_else(|| get_string_field(s, "authors"));
        }
        if fields.asin.is_none() {
            fields.asin = get_string_field(s, "ASIN").or_else(|| get_string_field(s, "asin"));
        }
        if fields.isbn.is_none() {
            fields.isbn = get_string_field(s, "isbn").or_else(|| get_string_field(s, "ISBN"));
        }
        if fields.publisher.is_none() {
            fields.publisher = get_string_field(s, "publisher");
        }
        if fields.description.is_none() {
            fields.description =
                get_string_field(s, "description").or_else(|| get_string_field(s, "synopsis"));
        }
        if fields.published_date.is_none() {
            fields.published_date = get_string_field(s, "issue_date")
                .or_else(|| get_string_field(s, "publication_date"));
        }
        if fields.language.is_none() {
            fields.language =
                get_string_field(s, "language").or_else(|| get_list_first_string(s, "languages"));
        }
        if fields.cde_type.is_none() {
            fields.cde_type =
                get_string_field(s, "cde_contenttype").or_else(|| get_string_field(s, "cde_type"));
        }

        // Recurse into nested structs (categorised_metadata, metadata values)
        for (_name, value) in s.fields() {
            extract_from_element(value, fields);
        }
    } else if let Some(list) = elem.as_sequence() {
        for item in list.iter() {
            extract_from_element(item, fields);
        }
    }
}

fn get_string_field(s: &ion_rs::Struct, name: &str) -> Option<String> {
    let elem = s.get(name)?;
    let text = elem.as_text().filter(|t| !t.is_empty())?;
    Some(text.to_string())
}

fn get_list_as_string(s: &ion_rs::Struct, name: &str) -> Option<String> {
    let elem = s.get(name)?;
    let seq = elem.as_sequence()?;
    let parts: Vec<String> = seq
        .iter()
        .filter_map(|e| e.as_text().map(|t| t.to_string()))
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn get_list_first_string(s: &ion_rs::Struct, name: &str) -> Option<String> {
    let elem = s.get(name)?;
    let seq = elem.as_sequence()?;
    seq.iter()
        .find_map(|e| e.as_text().map(|t| t.to_string()))
        .filter(|t| !t.is_empty())
}

/// Fallback: scan raw bytes for Ion payloads and try to extract metadata.
fn brute_scan_for_ion(data: &[u8], fields: &mut KfxFields) {
    let mut pos = 0;
    while pos + 4 < data.len() {
        if let Some(offset) = data[pos..].windows(4).position(|w| w == ION_MAGIC) {
            let start = pos + offset;
            let ion_slice = &data[start..];
            extract_fields_from_ion(ion_slice, fields);
            if fields.title.is_some() || fields.author.is_some() {
                return;
            }
            pos = start + 4;
        } else {
            break;
        }
    }
    debug!("KFX brute scan found no metadata");
}
