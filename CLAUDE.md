# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stratify is a cross-platform structural engineering calculation suite written in Rust. It provides native desktop applications (Windows, macOS, Linux) and a WebAssembly-based web application for structural analysis, design, and PDF report generation.

**Current Status**: Phase 3 in progress. NDS adjustment factors implemented with GUI. See `ROADMAP.md` for detailed progress and `NEXT_TODO.md` for current priorities.

## Build Commands

```bash
# Build and run GUI (native)
cargo run --bin calc_gui

# Build release
cargo build --release --bin calc_gui

# Run tests (140 tests: 112 unit + 28 doc)
cargo test

# Build CLI (placeholder only)
cargo run --bin calc_cli

# Build WebAssembly (compiles, runtime issue documented)
rustup target add wasm32-unknown-unknown
trunk build --release
```

## Architecture

The project is a Rust workspace with three crates:

```
stratify/
├── calc_core/           # [LIB] Pure Rust calculation engine
│   ├── calculations/    # Beam, column calculation modules
│   │   ├── beam.rs      # BeamInput, BeamResult, calculate()
│   │   └── column.rs    # Placeholder
│   ├── loads/           # Load handling
│   │   ├── mod.rs       # LoadCase, DesignMethod (ASD/LRFD)
│   │   ├── load_types.rs # LoadType enum (D, L, Lr, S, W, E, H)
│   │   ├── discrete.rs  # DiscreteLoad, EnhancedLoadCase
│   │   └── combinations.rs # ASCE 7 load combinations
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
├── calc_gui/            # [BIN] Iced 0.13 GUI application
│   └── main.rs          # ~2400 lines (see GUI_LAYOUT.md for panel structure)
└── calc_cli/            # [BIN] Placeholder CLI
```

### GUI Panel Structure

See `calc_gui/GUI_LAYOUT.md` for detailed layout. Main panels:
- **Toolbar**: File operations (New, Open, Save, Export PDF)
- **Items Panel** (left): Project navigation, beam list with [+] to create
- **Input Panel** (center): Editor for selected item (beam properties, loads, material, adjustment factors)
- **Results Panel** (right): Calculation results, diagrams (shear, moment, deflection)
- **Status Bar**: File path, lock status, messages

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
- **Beam calculations**: Simply-supported with uniform/point loads, full NDS checks
- **NDS adjustment factors**: C_D, C_M, C_t, C_L, C_F, C_fu, C_i, C_r with GUI controls
- **Load combinations**: ASCE 7 ASD (16 combos) and LRFD (7 combos), auto-governing
- **Discrete loads**: Multiple loads per beam (D, L, Lr, S, W, E, H), point and uniform
- **Wood materials**: Sawn lumber (5 species, 8 grades), Glulam, LVL, PSL
- **PDF export**: Professional reports with Typst, multi-beam export
- **GUI**: Live preview, auto-save, diagram rendering, file locking
- **Diagrams**: Beam schematic with reactions, shear (V), moment (M), deflection (δ)

### What's Placeholder/Incomplete
- Column calculations (stub only)
- Steel and concrete materials
- Point load/partial uniform calculations in beam solver (UI ready, calc treats as uniform)
- CLI interface (basic stdin demo only)
- WASM runtime (compiles but canvas context conflict - documented in NEXT_TODO.md)

## Tech Stack

- **GUI**: Iced 0.13 with canvas feature for diagrams
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
