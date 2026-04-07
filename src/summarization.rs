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
/// Handles responses containing 「</think>」 or similar thinking delimiters
pub fn extract_thinking_content(response: &str) -> (String, Option<String>) {
    // Common patterns for thinking delimiters
    let thinking_patterns = ["<think>", "<think>", "<thinking>", "]~b]"];

    let end_patterns = ["</think>", "</think>", "</thinking>", "]~b]"];

    // Try to find thinking blocks
    for start_pattern in &thinking_patterns {
        if let Some(start_idx) = response.find(start_pattern) {
            // Find the end pattern after start
            for end_pattern in &end_patterns {
                if let Some(end_idx) = response.find(end_pattern) {
                    if end_idx > start_idx + start_pattern.len() {
                        // Extract thinking content
                        let thinking =
                            response[start_idx + start_pattern.len()..end_idx].to_string();

                        // Build final content (everything before thinking + everything after)
                        let before = &response[..start_idx];
                        let after = &response[end_idx + end_pattern.len()..];
                        let final_content = format!("{}{}", before, after).trim().to_string();

                        return (final_content, Some(thinking));
                    }
                }
            }
        }
    }

    // No thinking block found, return original as final content
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
    let prompt = build_summary_prompt(transcript, template, custom_prompt);

    // Check if we should use streaming
    let use_streaming = ollama_client.supports_streaming();

    let response = if use_streaming {
        // For streaming, we'll collect the final response
        // Note: Full streaming implementation would be more complex
        ollama_client.generate_non_streaming(&prompt, model_name)?
    } else {
        ollama_client.generate_non_streaming(&prompt, model_name)?
    };

    let is_thinking = is_thinking_model(model_name);
    let (content, raw_thinking) = if is_thinking {
        extract_thinking_content(&response)
    } else {
        (response.trim().to_string(), None)
    };

    Ok(SummaryResult {
        content,
        model_name: model_name.to_string(),
        is_thinking_model: is_thinking,
        raw_thinking,
    })
}
