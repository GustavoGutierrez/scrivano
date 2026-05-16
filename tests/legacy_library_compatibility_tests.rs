use rusqlite::Connection;
use scrivano::database::Database;
use scrivano::export::export_to_txt;

#[test]
fn legacy_runtime_recording_is_visible_in_library_query_path() {
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let db_path = dir.path().join("recordings.db");

    let conn = Connection::open(&db_path).expect("legacy db must open");
    conn.execute_batch(
        "
        CREATE TABLE recordings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            filepath TEXT NOT NULL,
            created_at TEXT NOT NULL,
            duration_secs REAL NOT NULL DEFAULT 0.0,
            ollama_used INTEGER NOT NULL DEFAULT 0,
            ollama_model TEXT
        );

        CREATE TABLE transcript_segments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recording_id INTEGER NOT NULL,
            start_sec REAL NOT NULL,
            end_sec REAL NOT NULL,
            text TEXT NOT NULL
        );
        ",
    )
    .expect("legacy schema must be created");

    conn.execute(
        "INSERT INTO recordings (filename, filepath, created_at, duration_secs, ollama_used, ollama_model)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            "legacy.wav",
            "/tmp/legacy.wav",
            "2026-05-01T10:00:00Z",
            92.0_f64,
            0_i64,
            Option::<String>::None,
        ),
    )
    .expect("legacy recording insert must work");

    conn.execute(
        "INSERT INTO transcript_segments (recording_id, start_sec, end_sec, text)
         VALUES (?1, ?2, ?3, ?4)",
        (1_i64, 0.0_f64, 1.5_f64, "hola legacy"),
    )
    .expect("legacy segment insert must work");
    drop(conn);

    let db = Database::open(&db_path).expect("modern open/migration must work on legacy db");
    let recordings = db.list_recordings().expect("library list query must work");

    assert_eq!(recordings.len(), 1);
    assert_eq!(recordings[0].filename, "legacy.wav");
    assert_eq!(recordings[0].duration_display(), "01:32");

    let segments = db
        .get_segments_by_recording(recordings[0].id)
        .expect("expanded library row transcript query must work");
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].text, "hola legacy");
}

#[test]
fn legacy_runtime_recording_remains_usable_for_transcript_export() {
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let db_path = dir.path().join("recordings.db");
    let export_path = dir.path().join("legacy.txt");

    let conn = Connection::open(&db_path).expect("legacy db must open");
    conn.execute_batch(
        "
        CREATE TABLE recordings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            filepath TEXT NOT NULL,
            created_at TEXT NOT NULL,
            duration_secs REAL NOT NULL DEFAULT 0.0,
            ollama_used INTEGER NOT NULL DEFAULT 0,
            ollama_model TEXT
        );

        CREATE TABLE transcript_segments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recording_id INTEGER NOT NULL,
            start_sec REAL NOT NULL,
            end_sec REAL NOT NULL,
            text TEXT NOT NULL
        );
        ",
    )
    .expect("legacy schema must be created");

    conn.execute(
        "INSERT INTO recordings (filename, filepath, created_at, duration_secs, ollama_used, ollama_model)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            "legacy-2.wav",
            "/tmp/legacy-2.wav",
            "2026-05-01T10:00:00Z",
            8.0_f64,
            0_i64,
            Option::<String>::None,
        ),
    )
    .expect("legacy recording insert must work");

    conn.execute(
        "INSERT INTO transcript_segments (recording_id, start_sec, end_sec, text)
         VALUES (?1, ?2, ?3, ?4)",
        (1_i64, 0.0_f64, 2.0_f64, "primera linea"),
    )
    .expect("segment one insert must work");
    conn.execute(
        "INSERT INTO transcript_segments (recording_id, start_sec, end_sec, text)
         VALUES (?1, ?2, ?3, ?4)",
        (1_i64, 2.0_f64, 4.0_f64, "segunda linea"),
    )
    .expect("segment two insert must work");
    drop(conn);

    let db = Database::open(&db_path).expect("modern open/migration must work on legacy db");
    let recordings = db.list_recordings().expect("library list query must work");
    let segments = db
        .get_segments_by_recording(recordings[0].id)
        .expect("transcript read must work");

    export_to_txt(&segments, &export_path).expect("legacy transcript export should work");
    let exported = std::fs::read_to_string(&export_path).expect("txt export must exist");

    assert!(exported.contains("primera linea"));
    assert!(exported.contains("segunda linea"));
}
