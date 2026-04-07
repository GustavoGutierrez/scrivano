# Skill Registry — Scrivano

**Generated**: 2026-04-07
**Project**: Scrivano (formerly MeetWhisperer)
**Mode**: engram

---

## Project Conventions

- **AGENTS.md** — Agent guidelines for the project
- **PRP Generator** skill in `.agents/skills/`
- **Rust Engineer** skill in `.agents/skills/`

---

## User-Level Skills

| Skill | Trigger | Source |
|-------|---------|--------|
| sdd-init | Initialize SDD in a project | ~/.config/opencode/skills/ |
| sdd-explore | Explore and investigate ideas | ~/.config/opencode/skills/ |
| sdd-propose | Create a change proposal | ~/.config/opencode/skills/ |
| sdd-spec | Write specifications | ~/.config/opencode/skills/ |
| sdd-design | Create technical design | ~/.config/opencode/skills/ |
| sdd-tasks | Break down tasks | ~/.config/opencode/skills/ |
| sdd-apply | Implement tasks | ~/.config/opencode/skills/ |
| sdd-verify | Validate implementation | ~/.config/opencode/skills/ |
| sdd-archive | Archive completed change | ~/.config/opencode/skills/ |
| skill-creator | Create new AI agent skills | ~/.config/opencode/skills/ |
| skill-registry | Update skill registry | ~/.config/opencode/skills/ |
| branch-pr | Create pull requests | ~/.config/opencode/skills/ |
| issue-creation | Create GitHub issues | ~/.config/opencode/skills/ |
| judgment-day | Adversarial review | ~/.config/opencode/skills/ |
| go-testing | Go testing patterns | ~/.config/opencode/skills/ |

---

## Project-Level Skills

| Skill | Trigger | Location |
|-------|---------|----------|
| rust-engineer | Rust-specific code tasks | .agents/skills/rust-engineer/ |
| prp-generator | Create PRPs for features | .agents/skills/prp-generator/ |

---

## Active Documentation

- **AGENTS.md** — Main agent guidelines (Rust, egui, Whisper)
- **PRPs/** — Product Requirements Documents
- **PRD.md** — Product Requirements Definition

---

## Notes

- Project is transitioning from MeetWhisperer to Scrivano
- Primary stack: Rust + egui/eframe + Whisper
- System audio capture via PulseAudio/PipeWire