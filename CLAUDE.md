# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stratify is a cross-platform structural engineering calculation suite written in Rust. It provides native desktop applications (Windows, macOS, Linux) and a WebAssembly-based web application for structural analysis, design, and PDF report generation.

## Build Commands

```bash
# Build GUI (native)
cargo build --release --bin calc_gui

# Build CLI
cargo build --release --bin calc_cli

# Build WebAssembly
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --bin calc_gui
trunk build --release

# Run tests
cargo test

# Run GUI
cargo run --bin calc_gui

# Run CLI
cargo run --bin calc_cli
```

## Architecture

The project is a Rust workspace with three crates:

```
stratify/
├── calc_core/    # [LIB] Pure Rust calculation engine - all math, JSON I/O, PDF generation
├── calc_gui/     # [BIN] Iced/wgpu GUI application (native + WASM)
└── calc_cli/     # [BIN] Ratatui terminal interface
```

### Key Architectural Principles

- **calc_core is the source of truth**: All engineering logic, data structures, serialization, and PDF generation live here. No UI dependencies.
- **GUI and CLI are thin wrappers**: They import calc_core and handle only user interaction/rendering.
- **Data permanence first**: Atomic saves (write to .tmp, verify, rename), sentinel file locking (.lock files), human-readable JSON for recoverability.

### File Format

Projects use `.stf` extension (JSON). Items are stored in a flat UUID-keyed map for O(1) lookups:

```json
{
  "meta": { "version", "engineer", "job_id", "client", "created", "modified" },
  "settings": { "code", "seismic_design_cat", "risk_category", "default_materials" },
  "items": { "uuid": { "type": "Beam", "label": "B-1", ... } }
}
```

## Tech Stack

- **GUI**: Iced framework with wgpu backend (Vulkan/Metal/DX12 native, WebGPU/WebGL2 web)
- **TUI**: Ratatui
- **PDF**: Typst for programmable typesetting
- **Serialization**: serde/serde_json
- **File Locking**: fs2 for cross-platform network drive safety
- **Assets**: rust-embed for compile-time embedding of fonts, logos, catalogs
- **Units**: Wrapper types around f64 (e.g., `struct Kips(f64)`) for type safety

## Engineering Context

- US codes only: IBC 2012-2025, ASCE 7, NDS (wood), AISC 360 (steel), ACI 318 (concrete)
- Material databases: wood species/grades, steel sections (W-shapes, HSS, etc.), concrete specs
- Simpson Strong-Tie catalog integration via XML/CSV
- Calculation types: beams, columns, frames, shear walls, diaphragms, retaining walls, footings

## Design Decisions

- **Target users**: Small teams (3-5) on desktop, individuals working on separate projects, files on NAS/Google Drive
- **Offline-first**: No network connectivity required, no license checks
- **Robust file locking**: Critical for NAS/cloud drive scenarios - handle stale locks, crashes, network drops
- **Rust edition**: Use 2021 (not 2024, which doesn't exist yet)
- **CAD interop (future)**: Design import/export traits in calc_core for pluggable IFC/Revit/ArchiCAD support
