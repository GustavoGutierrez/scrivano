//! Ollama integration for MeetWhisperer.
//!
//! Provides:
//! - Detection of a running Ollama instance (`is_available`)
//! - Listing installed models (`list_models`)
//! - Post-processing transcripts to fix redaction and typos (`improve_transcript`)

use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

const OLLAMA_BASE: &str = "http://localhost:11434";

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    name: String,
}

/// Shape returned by POST /api/chat (stream:false)
#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns `true` if a local Ollama instance is reachable.
pub fn is_available() -> bool {
    ureq::get(&format!("{}/api/tags", OLLAMA_BASE))
        .timeout(std::time::Duration::from_secs(2))
        .call()
        .map(|r| r.status() == 200)
        .unwrap_or(false)
}

/// List the names of all models installed in the local Ollama instance.
/// Returns an empty `Vec` if Ollama is not reachable.
pub fn list_models() -> Vec<String> {
    let resp = ureq::get(&format!("{}/api/tags", OLLAMA_BASE))
        .timeout(std::time::Duration::from_secs(5))
        .call();

    match resp {
        Ok(r) => r
            .into_json::<ModelsResponse>()
            .map(|m| m.models.into_iter().map(|e| e.name).collect())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Send `raw_text` to Ollama for post-processing using the Chat API.
///
/// `progress_cb` is called with values 0–100 as the request progresses.
/// Uses a background thread to simulate progress (0→90 over ~30 s).
///
/// Uses `"think": false` so reasoning/thinking models (e.g. qwen3.5, deepseek-r1)
/// skip the internal chain-of-thought and return only the final answer directly.
pub fn improve_transcript<F>(model: &str, raw_text: &str, progress_cb: F) -> Result<String>
where
    F: Fn(i32) + Send + Sync + 'static,
{
    let system_prompt = build_system_prompt();
    let user_prompt = format!(
        "Texto a corregir:\n---\n{}\n---\n\nTexto corregido:",
        raw_text
    );

    let body = serde_json::json!({
        "model": model,
        "stream": false,
        "think": false,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user",   "content": user_prompt   }
        ],
        "options": {
            "temperature": 0.2,
            "top_p": 0.9,
            "num_predict": 4096
        }
    });

    // Shared flag: progress thread increments 0→90 over ~30 s until HTTP returns.
    let done_flag = Arc::new(AtomicI32::new(0));
    let done_thread = Arc::clone(&done_flag);
    let progress_cb = Arc::new(progress_cb);
    let progress_thread = Arc::clone(&progress_cb);

    std::thread::spawn(move || {
        for pct in (3..=90_i32).step_by(3) {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if done_thread.load(Ordering::SeqCst) != 0 {
                break;
            }
            progress_thread(pct);
        }
    });

    let response = ureq::post(&format!("{}/api/chat", OLLAMA_BASE))
        .timeout(std::time::Duration::from_secs(180))
        .send_json(body)
        .context("No se pudo conectar con Ollama")?;

    let chat: ChatResponse = response
        .into_json()
        .context("Respuesta de Ollama inválida")?;

    done_flag.store(1, Ordering::SeqCst);
    progress_cb(100);

    let cleaned = chat.message.content.trim().to_string();

    if cleaned.is_empty() {
        return Ok(raw_text.to_string());
    }

    eprintln!(
        "[ollama] {} chars → {} chars",
        raw_text.len(),
        cleaned.len()
    );
    Ok(cleaned)
}

// ── Prompt ────────────────────────────────────────────────────────────────────

fn build_system_prompt() -> &'static str {
    "Eres un corrector experto de transcripciones automáticas de audio generadas por Whisper. \
     Tu única tarea es mejorar el texto transcrito siguiendo estas reglas:\n\
     1. Corrige errores ortográficos y tipográficos obvios producto del reconocimiento de voz.\n\
     2. Restaura palabras cortadas o mal unidas (ej: \"estoy bien ven ido\" → \"estoy bienvenido\").\n\
     3. Agrega puntuación y mayúsculas donde corresponda.\n\
     4. Respeta el idioma original del texto (no traduzcas ni cambies el idioma).\n\
     5. NO agregues ni inventes información que no esté en el texto original.\n\
     6. NO agregues explicaciones, comentarios, prefijos ni notas al pie.\n\
     7. Devuelve SOLO el texto corregido, nada más.\n\
     8. Si el texto ya es correcto, devuélvelo sin cambios."
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires running Ollama instance"]
    fn detect_ollama() {
        assert!(is_available());
    }

    #[test]
    #[ignore = "requires running Ollama instance"]
    fn list_at_least_one_model() {
        let models = list_models();
        assert!(!models.is_empty(), "expected at least one installed model");
    }

    #[test]
    #[ignore = "requires running Ollama instance with qwen3.5:4b"]
    fn improve_returns_nonempty() {
        let result = improve_transcript("qwen3.5:4b", "hola komo estas ke tal", |_| {});
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(!text.is_empty());
        eprintln!("Improved: {}", text);
    }
}
