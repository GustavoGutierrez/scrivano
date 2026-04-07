use rusqlite::{params, Connection, Result};
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
    pub title: Option<String>,
    pub tags: Option<String>,
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

#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    pub id: i64,
    pub recording_id: i64,
    pub start_sec: f64,
    pub end_sec: f64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Highlight {
    pub id: i64,
    pub recording_id: i64,
    pub timestamp_sec: f64,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Summary {
    pub id: i64,
    pub recording_id: i64,
    pub template: String,
    pub content: String,
    pub model_name: Option<String>,
    pub is_thinking_model: bool,
}

#[derive(Debug, Clone)]
pub struct UserSettings {
    pub id: i64,
    pub language_default: String,
    pub hotkey_start_stop: String,
    pub hotkey_highlight: String,
    pub audio_input_device: Option<String>,
    pub whisper_model_path: String,
    pub ollama_host: String,
    pub ollama_port: i64,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            id: 1,
            language_default: "es".to_string(),
            hotkey_start_stop: "Ctrl+Shift+R".to_string(),
            hotkey_highlight: "Ctrl+Shift+H".to_string(),
            audio_input_device: None,
            whisper_model_path: "models/ggml-small.bin".to_string(),
            ollama_host: "localhost".to_string(),
            ollama_port: 11434,
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
                ollama_model  TEXT,
                title         TEXT,
                tags          TEXT
            );",
        )?;

        // Migration: add title and tags columns if they don't exist
        let _ = self
            .conn
            .execute("ALTER TABLE recordings ADD COLUMN title TEXT", []);
        let _ = self
            .conn
            .execute("ALTER TABLE recordings ADD COLUMN tags TEXT", []);

        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transcript_segments (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                recording_id  INTEGER NOT NULL,
                start_sec     REAL NOT NULL,
                end_sec       REAL NOT NULL,
                text          TEXT NOT NULL,
                FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE
            );",
        )?;

        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS highlights (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                recording_id   INTEGER NOT NULL,
                timestamp_sec  REAL NOT NULL,
                label          TEXT,
                FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE
            );",
        )?;

        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS summaries (
                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                recording_id      INTEGER NOT NULL,
                template          TEXT NOT NULL,
                content           TEXT NOT NULL,
                model_name        TEXT,
                is_thinking_model INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE
            );",
        )?;

        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS user_settings (
                id                    INTEGER PRIMARY KEY,
                language_default      TEXT NOT NULL DEFAULT 'es',
                hotkey_start_stop     TEXT NOT NULL DEFAULT 'Ctrl+Shift+R',
                hotkey_highlight      TEXT NOT NULL DEFAULT 'Ctrl+Shift+H',
                audio_input_device    TEXT,
                whisper_model_path    TEXT NOT NULL DEFAULT 'models/ggml-small.bin',
                ollama_host           TEXT NOT NULL DEFAULT 'localhost',
                ollama_port           INTEGER NOT NULL DEFAULT 11434
            );",
        )?;

        // Insert default settings if not exists
        self.conn
            .execute("INSERT OR IGNORE INTO user_settings (id) VALUES (1)", [])?;

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
        title: Option<&str>,
        tags: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO recordings (filename, filepath, created_at, duration_secs, ollama_used, ollama_model, title, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                filename,
                filepath,
                created_at,
                duration_secs,
                ollama_used as i64,
                ollama_model,
                title,
                tags,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_recording_title_and_tags(
        &self,
        recording_id: i64,
        title: Option<&str>,
        tags: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE recordings SET title = ?1, tags = ?2 WHERE id = ?3",
            params![title, tags, recording_id],
        )?;
        Ok(())
    }

    pub fn list_recordings(&self) -> Result<Vec<RecordingEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, filepath, created_at, duration_secs, ollama_used, ollama_model, title, tags
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
                title: row.get(7)?,
                tags: row.get(8)?,
            })
        })?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    pub fn delete_recording(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM recordings WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Transcript segments methods
    pub fn insert_segment(
        &self,
        recording_id: i64,
        start_sec: f64,
        end_sec: f64,
        text: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO transcript_segments (recording_id, start_sec, end_sec, text)
             VALUES (?1, ?2, ?3, ?4)",
            params![recording_id, start_sec, end_sec, text],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_segments_by_recording(&self, recording_id: i64) -> Result<Vec<TranscriptSegment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, recording_id, start_sec, end_sec, text
             FROM transcript_segments WHERE recording_id = ?1 ORDER BY start_sec",
        )?;
        let rows = stmt.query_map(params![recording_id], |row| {
            Ok(TranscriptSegment {
                id: row.get(0)?,
                recording_id: row.get(1)?,
                start_sec: row.get(2)?,
                end_sec: row.get(3)?,
                text: row.get(4)?,
            })
        })?;
        let mut segments = Vec::new();
        for row in rows {
            segments.push(row?);
        }
        Ok(segments)
    }

    // Highlights methods
    pub fn insert_highlight(
        &self,
        recording_id: i64,
        timestamp_sec: f64,
        label: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO highlights (recording_id, timestamp_sec, label)
             VALUES (?1, ?2, ?3)",
            params![recording_id, timestamp_sec, label],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_highlights_by_recording(&self, recording_id: i64) -> Result<Vec<Highlight>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, recording_id, timestamp_sec, label
             FROM highlights WHERE recording_id = ?1 ORDER BY timestamp_sec",
        )?;
        let rows = stmt.query_map(params![recording_id], |row| {
            Ok(Highlight {
                id: row.get(0)?,
                recording_id: row.get(1)?,
                timestamp_sec: row.get(2)?,
                label: row.get(3)?,
            })
        })?;
        let mut highlights = Vec::new();
        for row in rows {
            highlights.push(row?);
        }
        Ok(highlights)
    }

    // Summaries methods
    pub fn insert_summary(
        &self,
        recording_id: i64,
        template: &str,
        content: &str,
        model_name: Option<&str>,
        is_thinking_model: bool,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO summaries (recording_id, template, content, model_name, is_thinking_model)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                recording_id,
                template,
                content,
                model_name,
                is_thinking_model as i64,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_summaries_by_recording(&self, recording_id: i64) -> Result<Vec<Summary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, recording_id, template, content, model_name, is_thinking_model
             FROM summaries WHERE recording_id = ?1",
        )?;
        let rows = stmt.query_map(params![recording_id], |row| {
            Ok(Summary {
                id: row.get(0)?,
                recording_id: row.get(1)?,
                template: row.get(2)?,
                content: row.get(3)?,
                model_name: row.get(4)?,
                is_thinking_model: row.get::<_, i64>(5)? != 0,
            })
        })?;
        let mut summaries = Vec::new();
        for row in rows {
            summaries.push(row?);
        }
        Ok(summaries)
    }

    // User settings methods
    pub fn save_settings(&self, settings: &UserSettings) -> Result<()> {
        self.conn.execute(
            "UPDATE user_settings SET 
                language_default = ?1,
                hotkey_start_stop = ?2,
                hotkey_highlight = ?3,
                audio_input_device = ?4,
                whisper_model_path = ?5,
                ollama_host = ?6,
                ollama_port = ?7
             WHERE id = 1",
            params![
                settings.language_default,
                settings.hotkey_start_stop,
                settings.hotkey_highlight,
                settings.audio_input_device,
                settings.whisper_model_path,
                settings.ollama_host,
                settings.ollama_port,
            ],
        )?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<UserSettings> {
        let mut stmt = self.conn.prepare(
            "SELECT id, language_default, hotkey_start_stop, hotkey_highlight,
                    audio_input_device, whisper_model_path, ollama_host, ollama_port
             FROM user_settings WHERE id = 1",
        )?;
        let settings = stmt.query_row([], |row| {
            Ok(UserSettings {
                id: row.get(0)?,
                language_default: row.get(1)?,
                hotkey_start_stop: row.get(2)?,
                hotkey_highlight: row.get(3)?,
                audio_input_device: row.get(4)?,
                whisper_model_path: row.get(5)?,
                ollama_host: row.get(6)?,
                ollama_port: row.get(7)?,
            })
        })?;
        Ok(settings)
    }
}
