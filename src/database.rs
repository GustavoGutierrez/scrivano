use rusqlite::{Connection, Result, params};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RecordingEntry {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub created_at: String,
    pub duration_secs: f64,
    pub ollama_used: bool,
    pub ollama_model: Option<String>,
}

impl RecordingEntry {
    pub fn duration_display(&self) -> String {
        let total = self.duration_secs as u64;
        let h = total / 3600;
        let m = (total % 3600) / 60;
        let s = total % 60;
        if h > 0 {
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else {
            format!("{:02}:{:02}", m, s)
        }
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(db_path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS recordings (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                filename      TEXT NOT NULL,
                filepath      TEXT NOT NULL,
                created_at    TEXT NOT NULL,
                duration_secs REAL NOT NULL DEFAULT 0.0,
                ollama_used   INTEGER NOT NULL DEFAULT 0,
                ollama_model  TEXT
            );",
        )?;
        Ok(())
    }

    pub fn insert_recording(
        &self,
        filename: &str,
        filepath: &str,
        created_at: &str,
        duration_secs: f64,
        ollama_used: bool,
        ollama_model: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO recordings (filename, filepath, created_at, duration_secs, ollama_used, ollama_model)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                filename,
                filepath,
                created_at,
                duration_secs,
                ollama_used as i64,
                ollama_model,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_recordings(&self) -> Result<Vec<RecordingEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, filepath, created_at, duration_secs, ollama_used, ollama_model
             FROM recordings ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(RecordingEntry {
                id: row.get(0)?,
                filename: row.get(1)?,
                filepath: row.get(2)?,
                created_at: row.get(3)?,
                duration_secs: row.get(4)?,
                ollama_used: row.get::<_, i64>(5)? != 0,
                ollama_model: row.get(6)?,
            })
        })?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    pub fn delete_recording(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM recordings WHERE id = ?1", params![id])?;
        Ok(())
    }
}