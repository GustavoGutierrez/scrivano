# Delta for Export

## Purpose

Define export functionality for recordings, transcripts, and summaries.

## ADDED Requirements

### Requirement: Export to TXT

The system MUST export transcription to plain text file.

The export SHALL include all segments concatenated.

#### Scenario: Export transcript to TXT

- GIVEN user has transcription available
- WHEN user exports to TXT format
- THEN text file created with full transcript

### Requirement: Export to Markdown

The system MUST export transcription to Markdown file.

The export SHOULD include timestamps as headers.

#### Scenario: Export transcript to Markdown

- GIVEN user has transcription available
- WHEN user exports to Markdown format
- THEN markdown file created with formatted transcript

### Requirement: Export to JSON

The system MUST export data to JSON file.

The export SHALL include recording metadata, segments, highlights, summaries.

#### Scenario: Export to JSON

- GIVEN user selects JSON export
- WHEN user clicks export
- THEN JSON file created with all data

### Requirement: Export to SRT

The system MUST export transcription as SRT subtitle format.

Each segment MUST become an SRT block with index, timestamps, text.

#### Scenario: Export to SRT

- GIVEN user has transcription available
- WHEN user exports to SRT format
- THEN valid SRT file created with timecoded segments

### Requirement: Export to WebVTT

The system MUST export transcription as WebVTT subtitle format.

#### Scenario: Export to WebVTT

- GIVEN user has transcription available
- WHEN user exports to WebVTT format
- THEN valid VTT file created

## MODIFIED Requirements

None - this is a new specification.

## REMOVED Requirements

None.

---

## Success Criteria

- [ ] TXT export contains full transcript text
- [ ] Markdown export includes timestamps
- [ ] JSON export includes all data fields
- [ ] SRT export produces valid SRT format
- [ ] WebVTT export produces valid VTT format