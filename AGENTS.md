# Scrivano — Agent Guidelines

This document provides guidelines for agents working on the Scrivano codebase.

## Project Overview

Scrivano is a Rust desktop application that records system audio on Linux (via PulseAudio/PipeWire) and transcribes it locally using Whisper. The UI is built with egui/eframe.

---

## Build, Lint, and Test Commands

### Standard Commands

```bash
# Build the project
cargo build                    # Debug build
cargo build --release         # Optimized release build

# Run the application
cargo run                     # Run in debug mode
cargo run --release           # Run in release mode

# Run tests
cargo test                    # Run all unit and integration tests
cargo test --doc             # Run doctests only

# Run a single test
cargo test <test_name>       # Run specific test by name
cargo test --test <test_name> # Run specific integration test file

# Linting and formatting
cargo fmt                     # Format code (use --check to verify only)
cargo fmt --check            # Check formatting without modifying files

cargo clippy                 # Run lints
cargo clippy --all-targets --all-features  # Full linting
cargo clippy --fix           # Auto-fix clippy warnings

# Check everything before commit
cargo fmt --check && cargo clippy --all-targets && cargo test
```

### Project-Specific Dependencies

Some features require system libraries:
- **Audio capture**: Requires `libpulse-dev` (PulseAudio) or PipeWire development headers
- **Whisper compilation**: Requires `libclang-dev`
- **Audio playback (rodio)**: Requires `libasound2-dev` (ALSA)
- **Whisper models**: Download ggml-small.bin or ggml-medium.bin to `models/` directory

```bash
# On Debian/Ubuntu - Full dependencies for all features
sudo apt install libpulse-dev libasound2-dev libclang-dev

# Download Whisper model
mkdir -p models
# Copy ggml-small.bin or ggml-medium.bin from whisper.cpp to models/
```

#### Feature Flags

| Feature | Required System Libraries | Description |
|---------|--------------------------|-------------|
| Default (without features) | None | Basic build without tray or audio playback |
| `audio-playback` | `libasound2-dev` | Native audio playback using rodio |
| `tray-icon` | `libxdo-dev` | System tray icon support |

```bash
# Build with audio playback support (requires libasound2-dev)
cargo build --release --features "audio-playback tray-icon"

# Build without native audio (works without ALSA)
cargo build --release --no-default-features
```

---

## MCP Tools Usage Guidelines

### Context7 - Library Research

**Always use Context7** when researching external libraries or crates. This provides accurate, up-to-date documentation with code examples.

```bash
# Use the context7_resolve-library-id tool to find the library
# Then use context7_query-docs to get documentation and examples
```

When to use:
- Adding new dependencies to Cargo.toml
- Understanding crate API and usage patterns
- Finding code examples for library features
- Checking library features and configuration

### DevForge - Development Utilities

**Use DevForge** for development tasks like:
- Text transformations (slugify, base64, UUID generation)
- Code formatting and metrics
- Date/time operations
- Color conversions
- JWT/加密 utilities
- Image processing
- Audio/video transformations

Do NOT use for:
- Web requests (use devforge_http_request instead)
- Documentation lookup (use Context7)

---

## MCP Integration Examples

### Context7 Example - Adding a new crate

```rust
// 1. Search for the library
context7_resolve_library_id(query: "rust audio playback library", libraryName: "rodio")

// 2. Get documentation
context7_query_docs(libraryId: "/rustaudio/rodio", query: "play audio file with sink")
```

### DevForge Example - Generate a UUID

```rust
devforge_text_uuid(kind: "uuid4")
```

---

## Code Style Guidelines

### Formatting

- Use `cargo fmt` with default settings (rustfmt)
- Maximum line length: 100 characters
- Use 4 spaces for indentation (no tabs)
- Use trailing commas in multi-line structs/enums

### Imports

- Use absolute paths for standard library: `use std::sync::Arc;`
- Group imports in this order:
  1. Standard library (`std`, `core`)
  2. External crates
  3. Local modules (`crate::`, `super::`)
- Use `use` for bringing items into scope; prefer bringing only what you need

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Modules | snake_case | `audio_capture` |
| Structs | PascalCase | `AudioBuffer` |
| Enums | PascalCase | `RecordingState` |
| Enum variants | PascalCase | `Recording`, `Stopped` |
| Functions | snake_case | `start_recording()` |
| Variables | snake_case | `audio_buffer` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_BUFFER_SIZE` |
| Traits | PascalCase | `AudioSource` |
| Types (type aliases) | PascalCase | `AudioSamples` |

### Error Handling

- Use `Result<T, E>` for fallible operations; avoid `unwrap()` in production
- Prefer `thiserror` for library code (structured errors)
- Use `anyhow` for application code (simpler error handling)
- Use `?` operator for error propagation
- Use `expect()` with descriptive messages when unwrapping is acceptable

```rust
// Good: Custom error with thiserror
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Audio device error: {0}")]
    Device(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Good: Using anyhow for application code
use anyhow::{Context, Result};

fn process_audio(path: &str) -> Result<Vec<f32>> {
    let data = std::fs::read(path)
        .context("Failed to read audio file")?;
    // ...
    Ok(data)
}
```

### Ownership and Borrowing

- Prefer borrowing over cloning: `fn process(data: &[f32])` over `fn process(data: Vec<f32>)`
- Use `Arc` for shared ownership across threads
- Use `Mutex` or atomic types for shared mutable state
- Explicitly annotate lifetimes where inference is insufficient

```rust
// Good: Borrow input
fn process_samples(samples: &[f32]) -> Vec<f32> {
    samples.iter().map(|&s| s * 2.0).collect()
}

// Good: Shared state with Arc
pub struct AppState {
    pub recording: Arc<AtomicBool>,
    pub buffer: Arc<Mutex<Vec<f32>>>,
}
```

### Unsafe Code

- Minimize use of `unsafe` blocks
- Document safety invariants for every `unsafe` block
- Prefer safe abstractions over unsafe code

### Documentation

- Add doc comments (`///`) for public APIs
- Include usage examples in documentation
- Document error conditions in function docs

---

## Project Structure

```
Scrivano/
├── Cargo.toml
├── models/
│   └── ggml-small.bin        # Whisper model (user-provided)
├── src/
│   ├── main.rs               # Entry point
│   ├── audio.rs              # Audio capture (PulseAudio/PipeWire)
│   ├── transcription.rs      # Whisper integration
│   ├── ui.rs                 # egui UI
│   ├── ollama.rs             # Ollama client
│   ├── database.rs           # SQLite persistence
│   └── export.rs             # Export functionality
├── assets/
│   ├── favicons/             # App icons
│   └── logo.png              # App logo
├── PRPs/                     # Product Requirement Documents
├── openspec/                 # SDD specifications
└── AGENTS.md                 # This file
```

---

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| egui / eframe | 0.27+ | UI framework |
| whisper-rs | 0.15+ | Whisper transcription |
| tray-icon | 0.15+ | System tray |
| libpulse-binding | 2 | Audio capture |
| rusqlite | 0.31 | SQLite database |
| anyhow | 1.0 | Error handling |

---

## Common Patterns

### Audio Recording Thread

```rust
pub fn spawn_recorder(
    recording: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
) {
    thread::spawn(move || {
        let mut recorder = DesktopAudioRecorder::new()
            .expect("Failed to create audio recorder");
        
        while recording.load(Ordering::SeqCst) {
            if let Ok(frame) = recorder.read_frame() {
                let data = frame.pcm_data();
                buffer.lock().unwrap().extend_from_slice(data);
            }
        }
    });
}
```

### Thread-Safe State in egui

```rust
pub struct App {
    pub recording: Arc<AtomicBool>,
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    pub transcript: Arc<Mutex<String>>,
}
```

---

## Testing Guidelines

- Write unit tests in the same module using `#[cfg(test)]`
- Write integration tests in `tests/` directory
- Include doctests in public API documentation
- Run `cargo test --doc` to verify doctests pass

### Testing Requirements (MANDATORY)

**Critical Rule**: Any significant change to the codebase MUST include tests for the modified or new functionality. This applies to:

1. **Core modules** (`audio.rs`, `transcription.rs`, `database.rs`, `ollama.rs`):
   - MUST have unit tests for public functions
   - MUST test error handling paths
   
2. **Export functionality** (`export.rs`):
   - MUST test all format outputs (TXT, Markdown, JSON, SRT, WebVTT)
   - MUST test timestamp formatting utilities

3. **Database operations**:
   - MUST test CRUD operations
   - MUST test schema migrations

4. **Ollama integration**:
   - MUST test API response parsing
   - MUST handle offline/unavailable scenarios gracefully (mock or skip)

**When to write tests**:
- Before implementing a new feature
- After fixing a bug (add regression test)
- When modifying existing functionality

**Test categories**:
| Category | Location | When Required |
|----------|----------|---------------|
| Unit tests | `src/module.rs` (#[cfg(test)]) | Always |
| Integration tests | `tests/*.rs` | For multi-module features |
| Doctests | In documentation | For public APIs |

---

## Validation Before Committing

Always run these commands before submitting code:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

Fix any warnings or test failures before committing.