# Delta for Summarization

## Purpose

Define summary generation via Ollama with thinking model support.

## ADDED Requirements

### Requirement: Summary Template Selection

The system MUST allow user to select summary template: executive, tasks, or decisions.

The system SHALL generate summary using configured Ollama model.

#### Scenario: Generate executive summary

- GIVEN user has transcription available
- WHEN user selects "Executive Summary" template
- THEN Ollama generates summary in executive format

#### Scenario: Generate task list summary

- GIVEN user has transcription available
- WHEN user selects "Tasks" template
- THEN Ollama extracts action items and tasks

#### Scenario: Generate decisions summary

- GIVEN user has transcription available
- WHEN user selects "Decisions" template
- THEN Ollama extracts decisions made in meeting

### Requirement: Thinking Model Support

The system SHALL detect thinking/reasoning content in model responses.

The system MUST NOT expose thinking content to end users in normal flow.

#### Scenario: Extract final response from thinking model

- GIVEN model is a thinking model (e.g., deepseek-r1, qwen3)
- WHEN response contains thinking content
- THEN extract only final content for display
- AND store raw thinking internally if policy allows

### Requirement: Streaming Support

The system SHALL support streaming and non-streaming modes.

The system SHOULD auto-detect best mode based on model and connection.

#### Scenario: Streaming summary generation

- GIVEN streaming is enabled
- WHEN user requests summary
- THEN partial results appear progressively
- AND user can see content before completion

## MODIFIED Requirements

None - this is a new specification.

## REMOVED Requirements

None.

---

## Success Criteria

- [ ] User can select executive/tasks/decisions template
- [ ] Summary generates successfully via Ollama
- [ ] Thinking models show only final response
- [ ] Streaming mode works when enabled
- [ ] Clear error when Ollama unavailable