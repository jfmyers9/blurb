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
