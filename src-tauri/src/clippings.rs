use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clipping {
    pub title: String,
    pub author: Option<String>,
    pub clip_type: String,
    pub location_start: Option<i64>,
    pub location_end: Option<i64>,
    pub page: Option<i64>,
    pub clipped_at: Option<String>,
    pub text: String,
}

pub fn parse_clippings(content: &str) -> Vec<Clipping> {
    content
        .split("==========")
        .filter_map(|block| parse_block(block.trim()))
        .collect()
}

fn parse_block(block: &str) -> Option<Clipping> {
    if block.is_empty() {
        return None;
    }

    let lines: Vec<&str> = block.lines().collect();
    if lines.len() < 2 {
        return None;
    }

    let (title, author) = parse_title_author(lines[0]);
    let (clip_type, location_start, location_end, page, clipped_at) = parse_metadata(lines[1])?;

    let text = if lines.len() > 2 {
        // Skip blank line after metadata, join remaining
        lines[2..]
            .iter()
            .skip_while(|l| l.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    } else {
        String::new()
    };

    Some(Clipping {
        title,
        author,
        clip_type,
        location_start,
        location_end,
        page,
        clipped_at,
        text,
    })
}

fn parse_title_author(line: &str) -> (String, Option<String>) {
    let line = line.trim().trim_start_matches('\u{feff}');
    if let Some(paren_start) = line.rfind('(') {
        if line.ends_with(')') {
            let title = line[..paren_start].trim().to_string();
            let author = line[paren_start + 1..line.len() - 1].trim().to_string();
            if !title.is_empty() {
                return (title, Some(author));
            }
        }
    }
    (line.to_string(), None)
}

fn parse_metadata(
    line: &str,
) -> Option<(String, Option<i64>, Option<i64>, Option<i64>, Option<String>)> {
    let line = line.trim().trim_start_matches("- ");

    let clip_type = if line.starts_with("Your Highlight") {
        "highlight"
    } else if line.starts_with("Your Note") {
        "note"
    } else if line.starts_with("Your Bookmark") {
        "bookmark"
    } else {
        return None;
    };

    let mut location_start = None;
    let mut location_end = None;
    let mut page = None;
    let mut clipped_at = None;

    // Extract location: "on Location 234-238" or "on Location 50"
    if let Some(loc_idx) = line.find("Location ") {
        let after = &line[loc_idx + 9..];
        let loc_str: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
        if let Some(dash) = loc_str.find('-') {
            location_start = loc_str[..dash].parse().ok();
            location_end = loc_str[dash + 1..].parse().ok();
        } else {
            location_start = loc_str.parse().ok();
        }
    }

    // Extract page: "on page 42"
    if let Some(page_idx) = line.find("page ") {
        let after = &line[page_idx + 5..];
        let page_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        page = page_str.parse().ok();
    }

    // Extract timestamp after "Added on "
    if let Some(added_idx) = line.find("Added on ") {
        let ts_str = line[added_idx + 9..].trim();
        clipped_at = parse_kindle_timestamp(ts_str);
    }

    Some((
        clip_type.to_string(),
        location_start,
        location_end,
        page,
        clipped_at,
    ))
}

fn parse_kindle_timestamp(s: &str) -> Option<String> {
    // "Friday, March 15, 2024 2:30:15 PM"
    // Strip the day-of-week prefix
    let rest = s.find(", ").map(|i| &s[i + 2..])?;

    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let month_name = months.iter().find(|m| rest.starts_with(**m))?;
    let month_num = months.iter().position(|m| m == month_name)? + 1;

    // "March 15, 2024 2:30:15 PM"
    let after_month = rest[month_name.len()..].trim_start();
    let parts: Vec<&str> = after_month.splitn(2, ", ").collect();
    if parts.len() < 2 {
        return None;
    }
    let day: u32 = parts[0].parse().ok()?;

    // "2024 2:30:15 PM"
    let rest2: Vec<&str> = parts[1].splitn(2, ' ').collect();
    if rest2.len() < 2 {
        return None;
    }
    let year: u32 = rest2[0].parse().ok()?;

    let time_str = rest2[1].trim();
    let is_pm = time_str.ends_with("PM");
    let time_clean = time_str
        .trim_end_matches("AM")
        .trim_end_matches("PM")
        .trim();

    let time_parts: Vec<&str> = time_clean.split(':').collect();
    if time_parts.len() < 3 {
        return None;
    }
    let mut hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = time_parts[2].parse().ok()?;

    if is_pm && hour != 12 {
        hour += 12;
    } else if !is_pm && hour == 12 {
        hour = 0;
    }

    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        year, month_num, day, hour, minute, second
    ))
}

pub fn count_clipping_blocks(content: &str) -> usize {
    content
        .split("==========")
        .filter(|block| !block.trim().is_empty())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_highlight() {
        let input = r#"The Great Gatsby (F. Scott Fitzgerald)
- Your Highlight on Location 234-238 | Added on Friday, March 15, 2024 2:30:15 PM

So we beat on, boats against the current, borne back ceaselessly into the past.
=========="#;
        let clips = parse_clippings(input);
        assert_eq!(clips.len(), 1);
        let c = &clips[0];
        assert_eq!(c.title, "The Great Gatsby");
        assert_eq!(c.author.as_deref(), Some("F. Scott Fitzgerald"));
        assert_eq!(c.clip_type, "highlight");
        assert_eq!(c.location_start, Some(234));
        assert_eq!(c.location_end, Some(238));
        assert!(c.text.contains("So we beat on"));
    }

    #[test]
    fn test_parse_note() {
        let input = r#"1984 (George Orwell)
- Your Note on page 42 | Added on Saturday, April 20, 2024 10:15:00 AM

This reminds me of modern surveillance
=========="#;
        let clips = parse_clippings(input);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].clip_type, "note");
        assert_eq!(clips[0].page, Some(42));
    }

    #[test]
    fn test_parse_bookmark() {
        let input = r#"Dune (Frank Herbert)
- Your Bookmark on Location 1500 | Added on Sunday, May 5, 2024 8:00:00 AM


=========="#;
        let clips = parse_clippings(input);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].clip_type, "bookmark");
        assert_eq!(clips[0].location_start, Some(1500));
        assert!(clips[0].text.is_empty());
    }

    #[test]
    fn test_parse_timestamp_midnight() {
        let block = "Test Book (Author)\n- Your Highlight on page 1 | Location 1-2 | Added on Monday, January 1, 2024 12:00:00 AM\n\nSome text";
        let clippings = parse_clippings(&format!("{}\n==========", block));
        assert_eq!(clippings.len(), 1);
        assert!(
            clippings[0].clipped_at.as_ref().unwrap().contains("T00:00:00"),
            "12:00:00 AM should map to hour 0, got: {}",
            clippings[0].clipped_at.as_ref().unwrap()
        );
    }

    #[test]
    fn test_parse_timestamp_noon() {
        let block = "Test Book (Author)\n- Your Highlight on page 1 | Location 1-2 | Added on Monday, January 1, 2024 12:30:00 PM\n\nSome text";
        let clippings = parse_clippings(&format!("{}\n==========", block));
        assert_eq!(clippings.len(), 1);
        assert!(
            clippings[0].clipped_at.as_ref().unwrap().contains("T12:30:00"),
            "12:30:00 PM should stay hour 12, got: {}",
            clippings[0].clipped_at.as_ref().unwrap()
        );
    }
}
