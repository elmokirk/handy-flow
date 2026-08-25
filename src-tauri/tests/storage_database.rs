//! DATA-101 acceptance tests: connection policy + concurrent access.
//!
//! These are cargo integration tests: they exercise the storage module
//! exactly like future repositories will (public API only).

use handy_app_lib::storage::database::{AppDatabase, BUSY_TIMEOUT_MS};
use handy_app_lib::storage::ids;

fn temp_db_path(tag: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("handy-storage-test-{}-{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir.join("history.db")
}

#[test]
fn write_policy_pragmas_are_applied() {
    let path = temp_db_path("policy");
    let db = AppDatabase::open(&path).expect("open");

    let journal: String = db
        .conn()
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(journal.to_lowercase(), "wal", "WAL must be active");

    let fk: i64 = db
        .conn()
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fk, 1, "foreign_keys must be ON");

    let timeout: i64 = db
        .conn()
        .query_row("PRAGMA busy_timeout", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        timeout as u64, BUSY_TIMEOUT_MS,
        "busy timeout must match policy constant"
    );

    db.integrity_check().expect("fresh database must be intact");
}

#[test]
fn open_read_only_enforces_immutability() {
    let path = temp_db_path("readonly");

    // Seed with a writable connection and a real table.
    {
        let db = AppDatabase::open(&path).expect("seed open");
        db.conn()
            .execute_batch("CREATE TABLE t(x TEXT); INSERT INTO t VALUES('v');")
            .unwrap();
    }

    let ro = AppDatabase::open_read_only(&path).expect("read-only open");
    let value: String = ro
        .conn()
        .query_row("SELECT x FROM t", [], |r| r.get(0))
        .unwrap();
    assert_eq!(value, "v");

    let write_attempt = ro.conn().execute("INSERT INTO t VALUES('nope')", []);
    assert!(
        write_attempt.is_err(),
        "read-only connection must reject writes"
    );
}

#[test]
fn concurrent_reader_and_writer_do_not_fail_within_busy_timeout() {
    let path = temp_db_path("concurrent");
    let writer_db = AppDatabase::open(&path).expect("writer open");
    writer_db
        .conn()
        .execute_batch(
            "CREATE TABLE events(
                id INTEGER PRIMARY KEY,
                payload TEXT NOT NULL
             );",
        )
        .unwrap();

    // Independent connections: WAL allows one writer + concurrent readers.
    let ro_reader = AppDatabase::open_read_only(&path).expect("reader open");
    let second_writer = writer_db.clone();

    let writer_handle = std::thread::spawn(move || {
        for i in 0..200u32 {
            second_writer
                .conn()
                .execute(
                    "INSERT INTO events(payload) VALUES (?1)",
                    [format!("row-{i}")],
                )
                .expect("bounded writes must succeed within busy timeout");
        }
    });

    let reader_handle = std::thread::spawn(move || {
        let mut seen = 0usize;
        for _ in 0..50 {
            let count: i64 = ro_reader
                .conn()
                .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
                .expect("reader must not fail while writer runs");
            seen = seen.max(count as usize);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        seen
    });

    // Same-handle multi-thread access serializes on the mutex — must stay
    // deadlock-free and lossless.
    let same_handle_writer = {
        let db = writer_db.clone();
        std::thread::spawn(move || {
            for i in 200..300u32 {
                db.conn()
                    .execute(
                        "INSERT INTO events(payload) VALUES (?1)",
                        [format!("row-{i}")],
                    )
                    .expect("same-handle writes succeed");
            }
        })
    };

    writer_handle.join().expect("writer thread");
    same_handle_writer.join().expect("same-handle thread");
    let observed = reader_handle.join().expect("reader thread");
    assert!(observed <= 300);
}

#[test]
fn corrupt_file_is_reported_as_corrupt_data() {
    let path = temp_db_path("corrupt");
    std::fs::write(&path, b"this is definitely not sqlite").unwrap();

    let err = AppDatabase::open(&path).expect_err("must fail on non-sqlite file");
    assert!(
        matches!(
            err,
            handy_app_lib::storage::database::StorageError::CorruptData(_)
        ),
        "expected CorruptData, got: {err:?}"
    );
}

#[test]
fn uuidv7_ids_roundtrip_through_storage() {
    let path = temp_db_path("ids");
    let db = AppDatabase::open(&path).expect("open");
    db.conn()
        .execute_batch("CREATE TABLE rec(id TEXT PRIMARY KEY);")
        .unwrap();

    let id = ids::new_id();
    db.conn()
        .execute("INSERT INTO rec(id) VALUES (?1)", [&id])
        .unwrap();
    let stored: String = db
        .conn()
        .query_row("SELECT id FROM rec", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, id);
}
