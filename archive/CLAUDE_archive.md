# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stratify is a cross-platform structural engineering calculation suite written in Rust. It provides native desktop applications (Windows, macOS, Linux) and a WebAssembly-based web application for structural analysis, design, and PDF report generation.

# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds


<!-- bv-agent-instructions-v1 -->

---

## Beads Workflow Integration

This project uses [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) for issue tracking. Issues are stored in `.beads/` and tracked in git.

### Essential Commands

```bash
# View issues (launches TUI - avoid in automated sessions)
bv

# CLI commands for agents (use these instead)
bd ready              # Show issues ready to work (no blockers)
bd list --status=open # All open issues
bd show <id>          # Full issue details with dependencies
bd create --title="..." --type=task --priority=2
bd update <id> --status=in_progress
bd close <id> --reason="Completed"
bd close <id1> <id2>  # Close multiple issues at once
bd sync               # Commit and push changes
```

### Workflow Pattern

1. **Start**: Run `bd ready` to find actionable work
2. **Claim**: Use `bd update <id> --status=in_progress`
3. **Work**: Implement the task
4. **Complete**: Use `bd close <id>`
5. **Sync**: Always run `bd sync` at session end

### Key Concepts

- **Dependencies**: Issues can block other issues. `bd ready` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers, not words)
- **Types**: task, bug, feature, epic, question, docs
- **Blocking**: `bd dep add <issue> <depends-on>` to add dependencies

### Session Protocol

**Before ending any session, run this checklist:**

```bash
git status              # Check what changed
git add <files>         # Stage code changes
bd sync                 # Commit beads changes
git commit -m "..."     # Commit code
bd sync                 # Commit any new beads changes
git push                # Push to remote
```

### Best Practices

- Check `bd ready` at session start to find available work
- Update status as you work (in_progress → closed)
- Create new issues with `bd create` when you discover tasks
- Use descriptive titles and set appropriate priority/type
- Always `bd sync` before ending session

<!-- end-bv-agent-instructions -->

**Current Status**: Phase 3 in progress. Point loads, wind uplift (±W), and equations module complete. **WASM browser support working** with Iced 0.14 + WebGPU. Run `bd ready` for current priorities or `bd list --status=open` for all open issues.

## Build Commands

```bash
# Build and run GUI (native)
cargo run --bin calc_gui

# Build release
cargo build --release --bin calc_gui

# Run tests (185 tests: 149 unit + 36 doc)
cargo test

# Build CLI (placeholder only)
cargo run --bin calc_cli

# Build and run WebAssembly (working with WebGPU)
rustup target add wasm32-unknown-unknown
cd calc_gui && trunk serve --open
```

## Architecture

The project is a Rust workspace with three crates:

```
stratify/
├── calc_core/           # [LIB] Pure Rust calculation engine
│   ├── calculations/    # Beam, column calculation modules
│   │   ├── beam.rs      # BeamInput, BeamResult, calculate()
│   │   ├── beam_analysis.rs # Superposition analysis, SingleLoad, diagrams
│   │   └── column.rs    # Placeholder
│   ├── equations/       # Documented statics formulas (for manual review)
│   │   ├── beam.rs      # Point/uniform/partial load M, V, δ (Roark's refs)
│   │   └── section.rs   # Rectangular A, I, S, r, nominal-to-actual
│   ├── loads/           # Load handling
│   │   ├── mod.rs       # LoadCase, DesignMethod (ASD/LRFD)
│   │   ├── load_types.rs # LoadType enum (D, L, Lr, S, W, E, H)
│   │   ├── discrete.rs  # DiscreteLoad, EnhancedLoadCase
│   │   └── combinations.rs # ASCE 7 load combinations (±W for uplift)
│   ├── materials/       # Material properties
│   │   ├── mod.rs       # Material enum, unified interface
│   │   ├── sawn_lumber.rs # WoodSpecies, WoodGrade, NDS Table 4A
│   │   └── engineered_wood.rs # Glulam, LVL, PSL
│   ├── nds_factors.rs   # NDS adjustment factors (C_D, C_M, C_t, etc.)
│   ├── units.rs         # Type-safe wrappers: Feet, Inches, Kips, Psi
│   ├── project.rs       # Project, ProjectMetadata, GlobalSettings
│   ├── file_io.rs       # Atomic saves, FileLock with .lock files
│   ├── pdf.rs           # Typst-based PDF generation
│   └── errors.rs        # CalcError enum for structured errors
├── calc_gui/            # [BIN] Iced 0.14 GUI application
│   └── src/
│       ├── main.rs      # App state, Message enum, update logic (~1500 lines)
│       └── ui/          # Modular UI components (see GUI_LAYOUT.md)
│           ├── mod.rs           # Module exports
│           ├── toolbar.rs       # File operations, settings buttons
│           ├── items_panel.rs   # Left sidebar: Project Info, Beams list
│           ├── input_panel.rs   # Center panel dispatcher
│           ├── input_project_info.rs  # Project info editor
│           ├── input_wood_beam.rs     # Beam editor (spans, loads, material, NDS factors)
│           ├── results_panel.rs # Right panel dispatcher
│           ├── result_project_info.rs # Project summary view
│           ├── result_wood_beam.rs    # Calculation results + diagrams
│           ├── status_bar.rs    # Bottom status messages
│           └── shared/
│               ├── mod.rs       # Shared component exports
│               └── diagrams.rs  # Canvas drawing (shear, moment, deflection)
└── calc_cli/            # [BIN] Placeholder CLI
```

### GUI Panel Structure

See `calc_gui/GUI_LAYOUT.md` for detailed layout. The GUI uses a modular architecture:

- **main.rs**: App state (App struct), Message enum, update logic, and view orchestration
- **ui/**: Modular panel components with dispatcher pattern
  - `toolbar.rs`: File ops (New, Open, Save, Export PDF), theme toggle
  - `items_panel.rs`: Left sidebar with Project Info and Beams list
  - `input_panel.rs`: Dispatcher that routes to input_project_info or input_wood_beam
  - `results_panel.rs`: Dispatcher that routes to result_project_info or result_wood_beam
  - `status_bar.rs`: File path, lock status, messages
  - `shared/diagrams.rs`: Canvas drawing for beam schematic and V/M/δ diagrams

**Adding a new item type** (e.g., columns):
1. Create `input_wood_column.rs` with the editor UI
2. Create `result_wood_column.rs` with results display
3. Add `Column` variant to `EditorSelection` in main.rs
4. Update dispatchers in `input_panel.rs` and `results_panel.rs`
5. Update `items_panel.rs` to enable the section

### Key Architectural Principles

- **calc_core is the source of truth**: All engineering logic, data structures, serialization, and PDF generation live here. No UI dependencies.
- **GUI and CLI are thin wrappers**: They import calc_core and handle only user interaction/rendering.
- **Data permanence first**: Atomic saves (write to .tmp, verify, rename), sentinel file locking (.lock files), human-readable JSON for recoverability.
- **Stateless calculations**: Pure functions that take input structs and return result structs. No global state.

### File Format

Projects use `.stf` extension (JSON). Items are stored in a flat UUID-keyed map for O(1) lookups:

```json
{
  "meta": { "version": "0.1.0", "engineer": "Name", "job_id": "25-001", ... },
  "settings": { "code": "IBC2024", "design_method": "Asd", ... },
  "items": {
    "uuid-here": {
      "Beam": {
        "label": "B-1",
        "span_ft": 12.0,
        "load_case": { "loads": [...], "include_self_weight": true },
        "material": { "SawnLumber": { "species": "DouglasFirLarch", "grade": "No2" } },
        "width_in": 1.5,
        "depth_in": 9.25,
        "adjustment_factors": { "load_duration": "Normal", "wet_service": "Dry", ... }
      }
    }
  }
}
```

## Current Implementation

### What's Working
- **Beam calculations**: Simply-supported with uniform/point loads, superposition analysis
- **Point loads**: Full support with correct M, V, δ formulas at any position
- **Wind uplift**: ±W combinations for uplift design, min/max reactions tracked
- **NDS adjustment factors**: C_D, C_M, C_t, C_L, C_F, C_fu, C_i, C_r with GUI controls
- **Load combinations**: ASCE 7 ASD (21 combos) and LRFD (23 combos) with ±W uplift
- **Discrete loads**: Multiple loads per beam (D, L, Lr, S, W, E, H), point and uniform
- **Wood materials**: Sawn lumber (5 species, 8 grades), Glulam, LVL, PSL
- **PDF export**: Professional reports with Typst, multi-beam export
- **GUI**: Live preview, auto-save, diagram rendering, file locking
- **Diagrams**: Beam schematic with reactions, shear (V), moment (M), deflection (δ)
- **Equations module**: Documented statics formulas with Roark's references

### What's Placeholder/Incomplete
- Column calculations (stub only)
- Steel and concrete materials
- Partial uniform loads (structure ready, formulas implemented)
- CLI interface (basic stdin demo only)

## Tech Stack

- **GUI**: Iced 0.14 with canvas feature for diagrams, wgpu 27.0 for WebGPU
- **PDF**: typst 0.14 + typst-pdf + typst-assets (BerkeleyMono font)
- **Serialization**: serde/serde_json
- **File Locking**: fs2 for cross-platform network drive safety
- **File Dialogs**: rfd (native file dialogs)
- **Units**: Wrapper types around f64 with From/Into traits

## Engineering Context

- US codes only: IBC 2024, ASCE 7, NDS 2018 (wood)
- Current focus: Wood beam design at full depth
- Future: Steel (AISC 360), concrete (ACI 318)

### NDS Adjustment Factors (Implemented)
All factors in `calc_core/src/nds_factors.rs`:
- C_D: Load duration (Permanent 0.9 → Impact 2.0)
- C_M: Wet service (Dry 1.0, Wet 0.85-0.97)
- C_t: Temperature (Normal 1.0, Elevated 0.7-0.8, High 0.5-0.7)
- C_L: Beam stability (calculated from slenderness, or 1.0 if braced)
- C_F: Size factor (from NDS Table 4A based on depth)
- C_fu: Flat use factor
- C_i: Incising factor (0.80-0.95 for incised lumber)
- C_r: Repetitive member (1.15 for 3+ members at ≤24" OC)

## Design Decisions

- **Target users**: Small teams (3-5) on desktop, files on NAS/Google Drive
- **Offline-first**: No network connectivity required, no license checks
- **Robust file locking**: Critical for NAS/cloud drive scenarios
- **Rust edition**: 2021
- **Keep it simple**: Direct, readable code over clever patterns
