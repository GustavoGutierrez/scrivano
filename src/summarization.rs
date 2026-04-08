//! Summarization module for generating meeting summaries via Ollama.
//!
//! Supports multiple templates (executive, tasks, decisions) and handles
//! thinking models by extracting final content from reasoning blocks.

use crate::ollama::OllamaClient;
use anyhow::Result;

/// Summary template types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryTemplate {
    Executive,
    Complete,
    Tasks,
    Jira,
    Decisions,
}

impl SummaryTemplate {
    pub fn as_str(&self) -> &'static str {
        match self {
            SummaryTemplate::Executive => "executive",
            SummaryTemplate::Complete => "complete",
            SummaryTemplate::Tasks => "tasks",
            SummaryTemplate::Jira => "jira",
            SummaryTemplate::Decisions => "decisions",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "executive" => Some(SummaryTemplate::Executive),
            "complete" => Some(SummaryTemplate::Complete),
            "tasks" => Some(SummaryTemplate::Tasks),
            "jira" => Some(SummaryTemplate::Jira),
            "decisions" => Some(SummaryTemplate::Decisions),
            _ => None,
        }
    }
}

/// Summary result
#[derive(Debug, Clone)]
pub struct SummaryResult {
    pub content: String,
    pub model_name: String,
    pub is_thinking_model: bool,
    pub raw_thinking: Option<String>,
}

/// Prompt builder for summary templates
pub fn build_summary_prompt(
    transcript: &str,
    template: SummaryTemplate,
    custom_prompt: Option<&str>,
) -> String {
    let instruction = match template {
        SummaryTemplate::Executive => {
            "Provide a concise executive summary of the meeting in 3-5 bullet points. \
             Focus on: key topics discussed, main decisions made, and action items."
        }
        SummaryTemplate::Complete => {
            "Provide a comprehensive summary of the entire meeting. Include: attendee names mentioned, \
             all topics discussed, detailed decisions made, all action items with owners and deadlines, \
             important questions raised, and next steps. Be thorough and preserve all key information."
        }
        SummaryTemplate::Tasks => {
            "Extract all action items and tasks from the meeting. \
             For each task, include: what needs to be done, who is responsible (if mentioned), and deadline (if mentioned)."
        }
        SummaryTemplate::Jira => {
            "Extract all tasks and action items from the meeting and format them as Jira-style tickets. \
             For each task, provide: Summary (short title), Description (what needs to be done), \
             Assignee (who is responsible), Priority (Critical/High/Medium/Low), \
             Due Date (if mentioned). Format as:\n\n- **Summary:** [task title] \
             \n  **Description:** [details]\n  **Assignee:** [name or 'Unassigned']\n  **Priority:** [level]\n  **Due Date:** [date or 'TBD']"
        }
        SummaryTemplate::Decisions => {
            "List all decisions made during the meeting. \
             For each decision, briefly describe what was decided and any relevant context."
        }
    };

    let mut prompt = instruction.to_string();

    if let Some(custom) = custom_prompt {
        if !custom.trim().is_empty() {
            prompt.push_str("\n\nAdditional instructions: ");
            prompt.push_str(custom);
        }
    }

    format!(
        "{}\n\nMeeting Transcript:\n{}\n\nSummary:",
        prompt, transcript
    )
}

/// Known thinking model prefixes
const THINKING_MODEL_PREFIXES: &[&str] = &[
    "deepseek-r1",
    "deepseek-coder-r1",
    "qwen3",
    "qwen2.5-coder",
    "qwq",
    "calude-3-opus",
    "o1",
    "o1-mini",
    "o1-preview",
    "o3",
    "o3-mini",
];

/// Check if a model is a thinking model based on its name
pub fn is_thinking_model(model_name: &str) -> bool {
    let model_lower = model_name.to_lowercase();
    THINKING_MODEL_PREFIXES
        .iter()
        .any(|prefix| model_lower.contains(prefix))
}

/// Extract final content from thinking model response
/// Handles responses containing </think> or similar thinking delimiters
pub fn extract_thinking_content(response: &str) -> (String, Option<String>) {
    eprintln!(
        "[summarization] Extracting thinking from {} chars",
        response.len()
    );

    // Common patterns for thinking delimiters
    let thinking_patterns = ["<think>", "<thinking>", "[思考开始]", "【思考】"];
    let end_patterns = ["</think>", "</thinking>", "[思考结束]", "【/思考】"];

    // Try to find thinking blocks
    for (i, start_pattern) in thinking_patterns.iter().enumerate() {
        if let Some(start_idx) = response.find(start_pattern) {
            eprintln!(
                "[summarization] Found thinking start pattern '{}' at index {}",
                start_pattern, start_idx
            );

            // Find the corresponding end pattern after start
            let end_pattern = end_patterns.get(i).copied().unwrap_or("</think>");

            if let Some(end_idx) = response[start_idx + start_pattern.len()..].find(end_pattern) {
                let actual_end_idx = start_idx + start_pattern.len() + end_idx;
                eprintln!(
                    "[summarization] Found thinking end at index {}",
                    actual_end_idx
                );

                // Extract thinking content
                let thinking_start = start_idx + start_pattern.len();
                let thinking = response[thinking_start..actual_end_idx].to_string();

                // Build final content (everything before thinking + everything after)
                let before = &response[..start_idx];
                let after_end = actual_end_idx + end_pattern.len();
                let after = if after_end < response.len() {
                    &response[after_end..]
                } else {
                    ""
                };
                let final_content = format!("{}{}", before, after).trim().to_string();

                eprintln!(
                    "[summarization] Extracted {} chars of thinking, {} chars of final content",
                    thinking.len(),
                    final_content.len()
                );

                return (final_content, Some(thinking));
            } else {
                eprintln!(
                    "[summarization] Found start pattern but no end pattern '{}'",
                    end_pattern
                );
            }
        }
    }

    // No thinking block found - check if response contains only thinking-like content
    // Some models output thinking without proper delimiters
    eprintln!("[summarization] No standard thinking delimiters found");

    // Check for common patterns that indicate thinking content
    let lines: Vec<&str> = response.lines().collect();
    if lines.len() > 5 {
        // Look for separator lines like "---" or blank lines that might separate thinking from content
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "---" || line.trim().is_empty() {
                // Check if there's substantial content after this separator
                let after: String = lines[i + 1..].join("\n");
                if after.trim().len() > 50 {
                    eprintln!(
                        "[summarization] Found separator at line {}, using content after",
                        i
                    );
                    let thinking = lines[..i].join("\n");
                    return (after.trim().to_string(), Some(thinking));
                }
            }
        }
    }

    // No thinking block found, return original as final content
    eprintln!("[summarization] No thinking detected, returning full response as content");
    (response.trim().to_string(), None)
}

/// Generate a summary using Ollama
pub fn generate_summary(
    ollama_client: &OllamaClient,
    transcript: &str,
    template: SummaryTemplate,
    model_name: &str,
    custom_prompt: Option<&str>,
) -> Result<SummaryResult> {
    eprintln!("[summarization] ===== GENERATE_SUMMARY START =====");
    eprintln!(
        "[summarization] Template: {:?}, Model: {}, Transcript: {} chars",
        template,
        model_name,
        transcript.len()
    );
    eprintln!("[summarization] Custom prompt: {:?}", custom_prompt);

    let prompt = build_summary_prompt(transcript, template, custom_prompt);
    eprintln!("[summarization] Prompt built ({} chars)", prompt.len());
    eprintln!(
        "[summarization] Prompt preview: '{}'",
        prompt
            .chars()
            .take(150)
            .collect::<String>()
            .replace('\n', " ")
    );

    // Check if we should use streaming
    let use_streaming = ollama_client.supports_streaming();
    eprintln!("[summarization] Streaming supported: {}", use_streaming);

    eprintln!("[summarization] Calling Ollama API...");
    let response = if use_streaming {
        // For streaming, we'll collect the final response
        // Note: Full streaming implementation would be more complex
        ollama_client.generate_non_streaming(&prompt, model_name)?
    } else {
        ollama_client.generate_non_streaming(&prompt, model_name)?
    };

    eprintln!("[summarization] ===== OLLAMA RESPONSE =====");
    eprintln!(
        "[summarization] Raw response length: {} chars",
        response.len()
    );
    eprintln!(
        "[summarization] Raw response preview: '{}'",
        response
            .chars()
            .take(200)
            .collect::<String>()
            .replace('\n', " ")
    );

    let is_thinking = is_thinking_model(model_name);
    eprintln!("[summarization] Is thinking model: {}", is_thinking);

    let (content, raw_thinking) = if is_thinking {
        eprintln!("[summarization] Extracting thinking content...");
        extract_thinking_content(&response)
    } else {
        eprintln!("[summarization] Not a thinking model, using raw response");
        (response.trim().to_string(), None)
    };

    eprintln!("[summarization] ===== EXTRACTION RESULT =====");
    eprintln!(
        "[summarization] Final content length: {} chars",
        content.len()
    );
    if !content.is_empty() {
        eprintln!(
            "[summarization] Final content preview: '{}'",
            content
                .chars()
                .take(100)
                .collect::<String>()
                .replace('\n', " ")
        );
    } else {
        eprintln!("[summarization] WARNING: Final content is EMPTY!");
    }

    if let Some(ref thinking) = raw_thinking {
        eprintln!(
            "[summarization] Thinking extracted: {} chars",
            thinking.len()
        );
    }

    eprintln!("[summarization] ===== GENERATE_SUMMARY END =====");

    Ok(SummaryResult {
        content,
        model_name: model_name.to_string(),
        is_thinking_model: is_thinking,
        raw_thinking,
    })
}
