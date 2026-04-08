//! Ollama integration for Scrivano.
//!
//! Provides:
//! - Detection of a running Ollama instance (`is_available`)
//! - Listing installed models (`list_models`)
//! - Post-processing transcripts to fix redaction and typos (`improve_transcript`)
//! - Summary generation via Ollama
//! - STT fallback via Ollama

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

/// Ollama client for managing connections and requests
pub struct OllamaClient {
    host: String,
    port: u16,
}

impl OllamaClient {
    /// Create a new Ollama client with custom host and port
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }

    /// Create a client with default localhost:11434
    pub fn default_client() -> Self {
        Self::new("localhost", 11434)
    }

    /// Get the base URL for Ollama requests
    fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Check if Ollama is available
    pub fn is_available(&self) -> bool {
        ureq::get(&format!("{}/api/tags", self.base_url()))
            .timeout(std::time::Duration::from_secs(2))
            .call()
            .map(|r| r.status() == 200)
            .unwrap_or(false)
    }

    /// Check if the client supports streaming (always true for v1)
    pub fn supports_streaming(&self) -> bool {
        // For MVP, assume streaming is supported
        // Can be refined based on model capabilities
        true
    }

    /// Generate non-streaming response
    pub fn generate_non_streaming(&self, prompt: &str, model: &str) -> Result<String> {
        eprintln!(
            "[ollama] Generating with model: {} (prompt length: {} chars)",
            model,
            prompt.len()
        );

        let body = serde_json::json!({
            "model": model,
            "stream": false,
            "think": false,  // Disable thinking for models like qwen3.5, deepseek-r1, etc.
            "prompt": prompt,
            "options": {
                "temperature": 0.3,
                "top_p": 0.9,
                "num_predict": 2048
            }
        });

        eprintln!(
            "[ollama] Sending request to {}/api/generate",
            self.base_url()
        );

        let response = ureq::post(&format!("{}/api/generate", self.base_url()))
            .timeout(std::time::Duration::from_secs(300))
            .send_json(body)
            .context("Failed to connect to Ollama")?;

        eprintln!("[ollama] Response received, parsing...");

        #[derive(Deserialize)]
        struct GenerateResponse {
            response: String,
        }

        let gen_response: GenerateResponse = response
            .into_json()
            .context("Invalid response from Ollama")?;

        let resp_len = gen_response.response.len();
        eprintln!("[ollama] Response parsed successfully: {} chars", resp_len);

        if resp_len == 0 {
            return Err(anyhow::anyhow!("Ollama returned empty response"));
        }

        // Log first 200 chars for debugging
        let preview: String = gen_response.response.chars().take(200).collect();
        eprintln!(
            "[ollama] Response preview: '{}'",
            preview.replace('\n', " ")
        );

        Ok(gen_response.response)
    }

    /// List available models
    pub fn list_models(&self) -> Vec<String> {
        let resp = ureq::get(&format!("{}/api/tags", self.base_url()))
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
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns `true` if a local Ollama instance is reachable.
pub fn is_available() -> bool {
    OllamaClient::default_client().is_available()
}

/// Returns `true` if an Ollama instance is reachable at the given host:port.
pub fn is_available_at(host: &str, port: u16) -> bool {
    OllamaClient::new(host, port).is_available()
}

/// List the names of all models installed in the local Ollama instance.
/// Returns an empty `Vec` if Ollama is not reachable.
pub fn list_models() -> Vec<String> {
    OllamaClient::default_client().list_models()
}

/// List the names of all models installed in the Ollama instance at host:port.
/// Returns an empty `Vec` if Ollama is not reachable.
pub fn list_models_at(host: &str, port: u16) -> Vec<String> {
    OllamaClient::new(host, port).list_models()
}

/// Send `raw_text` to Ollama for post-processing using the Chat API.
///
/// `progress_cb` is called with values 0–100 as the request progresses.
/// Uses a background thread to simulate progress (0→90 over ~30 s).
///
/// Uses `"think": false` so reasoning/thinking models (e.g. qwen3.5, deepseek-r1)
/// skip the internal chain-of-thought and return only the final answer directly.
///
/// `custom_prompt` is an optional additional instruction to append to the system prompt.
pub fn improve_transcript<F>(
    model: &str,
    raw_text: &str,
    progress_cb: F,
    custom_prompt: Option<&str>,
) -> Result<String>
where
    F: Fn(i32) + Send + Sync + 'static,
{
    let system_prompt = build_system_prompt(custom_prompt);
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

/// Generate text using Ollama with a simple prompt (no transcript wrapping)
/// Used for title and tag generation
pub fn generate_text<F>(model: &str, prompt: &str, progress_cb: F) -> Result<String>
where
    F: Fn(i32) + Send + Sync + 'static,
{
    let body = serde_json::json!({
        "model": model,
        "stream": false,
        "think": false,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "num_predict": 256
        }
    });

    // Shared flag: progress thread increments 0→90 over ~10 s until HTTP returns.
    let done_flag = Arc::new(AtomicI32::new(0));
    let done_thread = Arc::clone(&done_flag);
    let progress_cb = Arc::new(progress_cb);
    let progress_thread = Arc::clone(&progress_cb);

    std::thread::spawn(move || {
        for pct in (10..=90_i32).step_by(10) {
            std::thread::sleep(std::time::Duration::from_millis(800));
            if done_thread.load(Ordering::SeqCst) != 0 {
                break;
            }
            progress_thread(pct);
        }
    });

    let response = ureq::post(&format!("{}/api/chat", OLLAMA_BASE))
        .timeout(std::time::Duration::from_secs(60))
        .send_json(body)
        .context("No se pudo conectar con Ollama")?;

    let chat: ChatResponse = response
        .into_json()
        .context("Respuesta de Ollama inválida")?;

    done_flag.store(1, Ordering::SeqCst);
    progress_cb(100);

    let cleaned = chat.message.content.trim().to_string();

    // Clean up common prefixes/suffixes that models might add
    let cleaned = cleaned
        .trim_start_matches("Título: ")
        .trim_start_matches("título: ")
        .trim_start_matches("Tags: ")
        .trim_start_matches("tags: ")
        .trim()
        .to_string();

    if cleaned.is_empty() {
        return Err(anyhow::anyhow!("Ollama returned empty response"));
    }

    eprintln!("[ollama] Generated text: {} chars", cleaned.len());
    Ok(cleaned)
}

// ── Prompt ────────────────────────────────────────────────────────────────────

fn build_system_prompt(custom_prompt: Option<&str>) -> String {
    let mut base = "Eres un corrector experto de transcripciones automáticas de audio generadas por Whisper. \
     Tu única tarea es mejorar el texto transcrito siguiendo estas reglas:\n\
     1. Corrige errores ortográficos y tipográficos obvios producto del reconocimiento de voz.\n\
     2. Restaura palabras cortadas o mal unidas (ej: \"estoy bien ven ido\" → \"estoy bienvenido\").\n\
     3. Agrega puntuación y mayúsculas donde corresponda.\n\
     4. Respeta el idioma original del texto (no traduzcas ni cambies el idioma).\n\
     5. NO agregues ni inventes información que no esté en el texto original.\n\
     6. NO agregues explicaciones, comentarios, prefijos ni notas al pie.\n\
     7. Devuelve SOLO el texto corregido, nada más.\n\
     8. Si el texto ya es correcto, devuélvelo sin cambios.".to_string();

    if let Some(custom) = custom_prompt {
        if !custom.trim().is_empty() {
            base.push_str("\n\nInstrucciones adicionales:\n");
            base.push_str(custom);
        }
    }

    base
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
        let result = improve_transcript("qwen3.5:4b", "hola komo estas ke tal", |_| {}, None);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(!text.is_empty());
        eprintln!("Improved: {}", text);
    }
}
