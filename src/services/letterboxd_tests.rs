use super::*;

const DIARY_CSV: &str = "Date,Name,Year,Letterboxd URI,Rating,Rewatch,Tags,Watched Date
2024-03-15,The Matrix,1999,https://letterboxd.com/film/the-matrix/,4.5,No,,2024-03-15
2024-03-10,Inception,2010,https://letterboxd.com/film/inception/,5.0,Yes,\"sci-fi, mind-bending\",2024-03-10
2024-02-28,Parasite,2019,https://letterboxd.com/film/parasite-2019/,4.0,No,thriller,2024-02-28
";

const RATINGS_CSV: &str = "Date,Name,Year,Letterboxd URI,Rating
2024-03-15,The Matrix,1999,https://letterboxd.com/film/the-matrix/,4.5
2024-01-20,Spirited Away,2001,https://letterboxd.com/film/spirited-away/,5.0
2024-01-15,The Godfather,1972,https://letterboxd.com/film/the-godfather/,4.0
";

#[test]
fn test_parse_diary_csv() {
    let entries = parse_diary_csv(DIARY_CSV).unwrap();
    assert_eq!(entries.len(), 3);

    let matrix = &entries[0];
    assert_eq!(matrix.title, "The Matrix");
    assert_eq!(matrix.year, Some(1999));
    assert_eq!(matrix.rating, Some(4.5));
    assert_eq!(matrix.rating_int, Some(5));
    assert_eq!(matrix.watched_date.as_deref(), Some("2024-03-15"));
    assert!(!matrix.rewatch);
    assert!(matrix.tags.is_empty());

    let inception = &entries[1];
    assert_eq!(inception.title, "Inception");
    assert!(inception.rewatch);
    assert_eq!(inception.tags, vec!["sci-fi", "mind-bending"]);
    assert_eq!(inception.rating, Some(5.0));
    assert_eq!(inception.rating_int, Some(5));
}

#[test]
fn test_parse_ratings_csv() {
    let entries = parse_ratings_csv(RATINGS_CSV).unwrap();
    assert_eq!(entries.len(), 3);

    let spirited = &entries[1];
    assert_eq!(spirited.title, "Spirited Away");
    assert_eq!(spirited.year, Some(2001));
    assert_eq!(spirited.rating, Some(5.0));
    assert_eq!(spirited.rating_int, Some(5));
    assert!(spirited.watched_date.is_none());
    assert!(!spirited.rewatch);
}

#[test]
fn test_merge_diary_overrides_ratings() {
    let diary = parse_diary_csv(DIARY_CSV).unwrap();
    let ratings = parse_ratings_csv(RATINGS_CSV).unwrap();
    let merged = merge_entries(diary, ratings);

    // 3 from diary + 2 unique from ratings = 5
    assert_eq!(merged.len(), 5);

    // The Matrix appears in both; diary version wins (has watched_date)
    let matrix = merged.iter().find(|e| e.title == "The Matrix").unwrap();
    assert_eq!(matrix.watched_date.as_deref(), Some("2024-03-15"));
    assert_eq!(matrix.rating, Some(4.5));

    // Spirited Away only in ratings
    let spirited = merged.iter().find(|e| e.title == "Spirited Away").unwrap();
    assert_eq!(spirited.rating, Some(5.0));
    assert!(spirited.watched_date.is_none());
}

#[test]
fn test_merge_diary_inherits_rating_from_ratings() {
    let diary_csv = "Date,Name,Year,Letterboxd URI,Rating,Rewatch,Tags,Watched Date
2024-03-15,Some Film,2020,https://letterboxd.com/film/some-film/,,No,,2024-03-15
";
    let ratings_csv = "Date,Name,Year,Letterboxd URI,Rating
2024-01-01,Some Film,2020,https://letterboxd.com/film/some-film/,3.5
";
    let diary = parse_diary_csv(diary_csv).unwrap();
    let ratings = parse_ratings_csv(ratings_csv).unwrap();
    let merged = merge_entries(diary, ratings);

    assert_eq!(merged.len(), 1);
    let entry = &merged[0];
    assert_eq!(entry.title, "Some Film");
    assert_eq!(entry.rating, Some(3.5));
    assert_eq!(entry.rating_int, Some(4));
    assert_eq!(entry.watched_date.as_deref(), Some("2024-03-15"));
}

#[test]
fn test_rating_conversion() {
    assert_eq!(convert_rating(0.5), 1);
    assert_eq!(convert_rating(1.0), 1);
    assert_eq!(convert_rating(1.5), 2);
    assert_eq!(convert_rating(2.0), 2);
    assert_eq!(convert_rating(2.5), 3);
    assert_eq!(convert_rating(3.0), 3);
    assert_eq!(convert_rating(3.5), 4);
    assert_eq!(convert_rating(4.0), 4);
    assert_eq!(convert_rating(4.5), 5);
    assert_eq!(convert_rating(5.0), 5);
}

#[test]
fn test_empty_csv() {
    let diary_csv = "Date,Name,Year,Letterboxd URI,Rating,Rewatch,Tags,Watched Date\n";
    let entries = parse_diary_csv(diary_csv).unwrap();
    assert!(entries.is_empty());
}

#[test]
fn test_missing_optional_fields() {
    let diary_csv = "Date,Name,Year,Letterboxd URI,Rating,Rewatch,Tags,Watched Date
2024-01-01,Mystery Film,,,,,,
";
    let entries = parse_diary_csv(diary_csv).unwrap();
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.title, "Mystery Film");
    assert_eq!(entry.year, None);
    assert_eq!(entry.rating, None);
    assert_eq!(entry.rating_int, None);
    assert!(entry.watched_date.is_none());
    assert!(!entry.rewatch);
    assert!(entry.tags.is_empty());
}

#[test]
fn test_invalid_rating_ignored() {
    let ratings_csv = "Date,Name,Year,Letterboxd URI,Rating
2024-01-01,Bad Rating Film,2020,https://example.com,notanumber
2024-01-01,Too High,2020,https://example.com,6.0
2024-01-01,Too Low,2020,https://example.com,0.0
";
    let entries = parse_ratings_csv(ratings_csv).unwrap();
    assert_eq!(entries.len(), 3);
    assert!(entries[0].rating.is_none());
    assert!(entries[1].rating.is_none());
    assert!(entries[2].rating.is_none());
}

#[test]
fn test_merge_sorted_by_title() {
    let diary_csv = "Date,Name,Year,Letterboxd URI,Rating,Rewatch,Tags,Watched Date
2024-01-01,Zebra Movie,2020,,,No,,2024-01-01
2024-01-01,Alpha Movie,2020,,,No,,2024-01-01
";
    let diary = parse_diary_csv(diary_csv).unwrap();
    let merged = merge_entries(diary, Vec::new());
    assert_eq!(merged[0].title, "Alpha Movie");
    assert_eq!(merged[1].title, "Zebra Movie");
}
