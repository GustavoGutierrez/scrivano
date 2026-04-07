# MeetWhisperer — Agent Guidelines

This document provides guidelines for agents working on the MeetWhisperer codebase.

## Project Overview

MeetWhisperer is a Rust desktop application that records system audio on Linux (via PulseAudio/PipeWire) and transcribes it locally using Whisper. The UI is built with egui/eframe.

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
- **Whisper models**: Download ggml-small.bin or ggml-medium.bin to `models/` directory

```bash
# On Debian/Ubuntu
sudo apt install libpulse-dev

# Download Whisper model
mkdir -p models
# Copy ggml-small.bin or ggml-medium.bin from whisper.cpp to models/
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
MeetWhisperer/
├── Cargo.toml
├── models/
│   └── ggml-small.bin        # Whisper model (user-provided)
├── src/
│   ├── main.rs               # Entry point
│   ├── audio.rs              # Audio capture (PulseAudio/PipeWire)
│   ├── transcription.rs      # Whisper integration
│   └── ui.rs                 # egui UI
└── AGENTS.md                 # This file
```

---

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| egui / eframe | 0.27+ | UI framework |
| whisper-rs | 0.15+ | Whisper transcription |
| tray-icon | 0.15+ | System tray |
| simple-pulse-desktop-capture | 0.2+ | System audio capture |
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

---

## Validation Before Committing

Always run these commands before submitting code:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

Fix any warnings or test failures before committing.
