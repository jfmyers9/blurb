use super::*;
use crate::data::migrations::{get_user_version, run_migration_list, run_migrations, Migration};
use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;

fn create_test_db(dir: &TempDir) -> (Connection, std::path::PathBuf) {
    let db_path = dir.path().join("blurb.db");
    let conn = Connection::open(&db_path).unwrap();
    run_migrations(&conn).unwrap();
    (conn, db_path)
}

#[test]
fn test_backup_creates_valid_sqlite_file() {
    let dir = TempDir::new().unwrap();
    let (conn, db_path) = create_test_db(&dir);
    let version = get_user_version(&conn).unwrap();

    backup_before_migration(&conn, &db_path, version).unwrap();

    let backup_path = db_path.with_extension(format!("db.bak-v{version}"));
    assert!(backup_path.exists(), "backup file should exist");

    let backup_conn = Connection::open(&backup_path).unwrap();
    let backup_version: i32 = backup_conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(backup_version, version);
}

#[test]
fn test_cleanup_removes_old_backups() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("blurb.db");
    fs::write(&db_path, "fake db").unwrap();

    for v in 1..=3 {
        let bak = db_path.with_extension(format!("db.bak-v{v}"));
        fs::write(&bak, format!("backup v{v}")).unwrap();
    }

    cleanup_old_backups(&db_path, 3).unwrap();

    assert!(
        !db_path.with_extension("db.bak-v1").exists(),
        "v1 backup should be deleted"
    );
    assert!(
        !db_path.with_extension("db.bak-v2").exists(),
        "v2 backup should be deleted"
    );
    assert!(
        db_path.with_extension("db.bak-v3").exists(),
        "v3 backup should be kept"
    );
}

#[test]
fn test_no_backup_files_when_db_is_current() {
    let dir = TempDir::new().unwrap();
    let (conn, db_path) = create_test_db(&dir);

    // DB is already at latest — backup logic should not create any .bak-v* files
    let current = get_user_version(&conn).unwrap();
    if current < crate::data::migrations::latest_version() {
        backup_before_migration(&conn, &db_path, current).unwrap();
    }

    let has_backup = fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .any(|e| e.file_name().to_string_lossy().contains(".bak-v"));
    assert!(!has_backup, "no .bak-v files should exist for a current DB");
}

#[test]
fn test_backup_survives_failed_migration() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("blurb.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        .unwrap();

    // Run only the first migration to leave DB at version 1
    let first_migration = crate::data::migrations::migrations_for_testing();
    run_migration_list(&conn, &first_migration[..1]).unwrap();
    let original_version = get_user_version(&conn).unwrap();
    assert_eq!(original_version, 1);

    backup_before_migration(&conn, &db_path, original_version).unwrap();
    let backup_path = db_path.with_extension(format!("db.bak-v{original_version}"));
    assert!(backup_path.exists());

    // Attempt a migration that fails
    let failing_migrations = vec![Migration {
        version: 999,
        description: "intentionally broken",
        up: |_conn| Err(rusqlite::Error::ExecuteReturnedResults),
    }];
    let result = run_migration_list(&conn, &failing_migrations);
    assert!(result.is_err());

    // Backup file must still exist and be a valid SQLite DB
    assert!(
        backup_path.exists(),
        "backup must survive a failed migration"
    );
    let backup_conn = Connection::open(&backup_path).unwrap();
    let backup_version: i32 = backup_conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(backup_version, original_version);
}

#[test]
fn test_backup_migrate_cleanup_flow() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("blurb.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        .unwrap();

    // Start at version 1 (below latest)
    let all_migrations = crate::data::migrations::migrations_for_testing();
    run_migration_list(&conn, &all_migrations[..1]).unwrap();
    let original_version = get_user_version(&conn).unwrap();
    assert_eq!(original_version, 1);

    // Full init_db-equivalent sequence: backup, migrate, cleanup
    backup_before_migration(&conn, &db_path, original_version).unwrap();
    run_migration_list(&conn, &all_migrations).unwrap();
    cleanup_old_backups(&db_path, original_version).unwrap();

    // The backup from the original version should still exist after cleanup
    let backup_path = db_path.with_extension(format!("db.bak-v{original_version}"));
    assert!(
        backup_path.exists(),
        "backup of original version should be retained after cleanup"
    );

    // DB should now be at latest version
    let final_version = get_user_version(&conn).unwrap();
    assert_eq!(final_version, crate::data::migrations::latest_version());
}

#[test]
fn resolves_production_profile_by_default() {
    let paths = resolve_data_paths(None, None).unwrap();

    assert_eq!(paths.profile, DataProfile::Production);
    assert_eq!(paths.app_id, PRODUCTION_APP_ID);
    assert!(paths.is_production());
    assert!(paths.app_dir.ends_with(PRODUCTION_APP_ID));
    assert_eq!(paths.db_path, paths.app_dir.join("blurb.db"));
    assert_eq!(paths.covers_dir, paths.app_dir.join("covers"));
}

#[test]
fn resolves_development_profile_from_env_value() {
    let paths = resolve_data_paths(Some("dev"), None).unwrap();

    assert_eq!(paths.profile, DataProfile::Development);
    assert_eq!(paths.app_id, DEVELOPMENT_APP_ID);
    assert!(!paths.is_production());
    assert_eq!(paths.label(), "Dev");
    assert_eq!(paths.window_title(), "Blurb (Dev)");
    assert!(paths.app_dir.ends_with(DEVELOPMENT_APP_ID));
    assert!(paths.log_dir.unwrap().ends_with(DEVELOPMENT_APP_ID));
}

#[test]
fn rejects_unknown_profile_names() {
    let err = resolve_data_paths(Some("deev"), None).unwrap_err();

    assert!(err.contains("invalid BLURB_PROFILE"));
}

#[test]
fn custom_data_dir_overrides_profile_namespace() {
    let dir = TempDir::new().unwrap();
    let paths = resolve_data_paths(Some("dev"), Some(dir.path().join("sandbox"))).unwrap();

    assert_eq!(paths.profile, DataProfile::Custom);
    assert!(!paths.is_production());
    assert_eq!(paths.app_dir, dir.path().join("sandbox"));
    assert_eq!(paths.db_path, dir.path().join("sandbox/blurb.db"));
    assert_eq!(paths.covers_dir, dir.path().join("sandbox/covers"));
    assert_eq!(paths.log_dir, Some(dir.path().join("sandbox/logs")));
}

#[test]
fn init_db_at_uses_supplied_profile_path() {
    let dir = TempDir::new().unwrap();
    let paths = DataPaths::new(
        DataProfile::Development,
        DEVELOPMENT_APP_ID,
        dir.path().join("dev-root"),
        None,
    );

    let conn = init_db_at(&paths).unwrap();
    let version = get_user_version(&conn).unwrap();

    assert_eq!(version, crate::data::migrations::latest_version());
    assert!(paths.db_path.exists());
    assert!(!dir.path().join(PRODUCTION_APP_ID).exists());
}

#[test]
fn covers_dir_for_uses_supplied_profile_path() {
    let dir = TempDir::new().unwrap();
    let paths = DataPaths::new(
        DataProfile::Development,
        DEVELOPMENT_APP_ID,
        dir.path().join("dev-root"),
        None,
    );

    let covers = covers_dir_for(&paths).unwrap();

    assert_eq!(covers, paths.covers_dir);
    assert!(covers.exists());
}
