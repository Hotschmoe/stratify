# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stratify is a cross-platform structural engineering calculation suite written in Rust. It provides native desktop applications (Windows, macOS, Linux) and a WebAssembly-based web application for structural analysis, design, and PDF report generation.

**Current Status**: Phase 2 complete. Working GUI with beam calculations, diagram rendering, PDF export, and file management. See `ROADMAP.md` for detailed progress.

## Build Commands

```bash
# Build and run GUI (native)
cargo run --bin calc_gui

# Build release
cargo build --release --bin calc_gui

# Run tests
cargo test

# Build CLI (placeholder only)
cargo run --bin calc_cli

# Build WebAssembly (not yet tested)
rustup target add wasm32-unknown-unknown
trunk build --release
```

## Architecture

The project is a Rust workspace with three crates:

```
stratify/
├── calc_core/           # [LIB] Pure Rust calculation engine
│   ├── calculations/    # Beam, column, etc. calculation modules
│   │   ├── beam.rs      # BeamInput, BeamResults, calculate_beam()
│   │   └── column.rs    # Placeholder
│   ├── materials.rs     # WoodSpecies, WoodGrade, WoodProperties (NDS 2018)
│   ├── units.rs         # Type-safe wrappers: Feet, Inches, Kips, Psi, etc.
│   ├── project.rs       # Project, ProjectMetadata, GlobalSettings, CalculationItem
│   ├── file_io.rs       # Atomic saves, FileLock with .lock files
│   ├── pdf.rs           # Typst-based PDF generation
│   └── errors.rs        # CalcError enum for structured errors
├── calc_gui/            # [BIN] Iced 0.13 GUI application
│   └── main.rs          # ~1600 lines: toolbar, forms, canvas diagrams
└── calc_cli/            # [BIN] Placeholder CLI
```

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
  "settings": { "code": "IBC2021", "seismic_design_cat": "D", ... },
  "items": {
    "uuid-here": {
      "item_type": "Beam",
      "label": "B-1",
      "data": { "span_ft": 20.0, "tributary_width_ft": 12.0, ... }
    }
  }
}
```

## Current Implementation

### What's Working
- **Beam calculations**: Simply-supported with uniform load, NDS bending/shear/deflection checks
- **Wood materials**: Species (DF-L, SP, HF, SPF, DF-S), grades (Sel Str through Utility), NDS 2018 Table 4A values
- **PDF export**: Professional reports with Typst, pass/fail status, multi-beam export
- **GUI**: Toolbar (New/Open/Save/Export), beam input form, results display, diagram canvas
- **Diagrams**: Beam schematic, shear (V), moment (M), deflection (δ), support reactions
- **File I/O**: Atomic saves, file locking, read-only mode for locked files

### What's Placeholder/Incomplete
- Column calculations (stub only)
- Steel and concrete materials
- NDS adjustment factors (C_D, C_M, C_t, C_L, C_F, etc.)
- Load combinations (ASCE 7)
- CLI interface (basic stdin demo only)

## Tech Stack

- **GUI**: Iced 0.13 with canvas feature for diagrams
- **PDF**: typst 0.14 + typst-pdf + typst-assets (BerkeleyMono font)
- **Serialization**: serde/serde_json
- **File Locking**: fs2 for cross-platform network drive safety
- **File Dialogs**: rfd (native file dialogs)
- **Units**: Wrapper types around f64 with From/Into traits

## Engineering Context

- US codes only: IBC 2012-2025, ASCE 7, NDS (wood), AISC 360 (steel), ACI 318 (concrete)
- Current focus: NDS wood design (sawn lumber)
- Next priority: Full NDS adjustment factors, then load combinations

### NDS Reference (Phase 3 Focus)
Adjustment factors to implement:
- C_D: Load duration (Table 2.3.2) - 0.9 to 2.0 based on load type
- C_M: Wet service (Table 4A footnotes) - 0.85 typical for wet conditions
- C_t: Temperature (Table 2.3.3) - 1.0 for T ≤ 100°F
- C_L: Beam stability (3.3.3) - function of slenderness
- C_F: Size factor (Table 4A) - varies by depth and width
- C_fu: Flat use (Table 4A) - for members loaded on wide face
- C_i: Incising (4.3.8) - 0.80 for incised lumber
- C_r: Repetitive member (4.3.9) - 1.15 for joists/rafters/studs

## Design Decisions

- **Target users**: Small teams (3-5) on desktop, individuals working on separate projects, files on NAS/Google Drive
- **Offline-first**: No network connectivity required, no license checks
- **Robust file locking**: Critical for NAS/cloud drive scenarios - handle stale locks, crashes, network drops
- **Rust edition**: 2021
- **Keep it simple**: Avoid over-abstraction. Direct, readable code over clever patterns.
