# Stratify

**Native Structural Engineering Suite**  
Target Platforms: Windows, macOS, Linux, WebAssembly (Browser)

A high-performance, GPU-accelerated structural analysis tool designed for data permanence, auditability, and cross-platform deployment. Replaces legacy black-box software with transparent, version-control-friendly architecture.

## Overview

Stratify is a comprehensive structural engineering calculation suite built in Rust. It provides native desktop applications and a full-featured web application (via WebAssembly) for structural analysis, design, and reporting. The application prioritizes data integrity, human-readable project files, and seamless integration with modern engineering workflows including version control systems.

## Key Features

- **Cross-Platform**: Native executables for Windows, macOS, and Linux, plus full WebAssembly support for browser deployment
- **GPU-Accelerated GUI**: Built on `wgpu` for high-performance rendering of diagrams and visualizations
- **Data Permanence**: JSON-based project files (.stf) that are human-readable, version-controllable, and recoverable
- **Network Drive Safe**: Atomic saves and file locking prevent corruption on Google Drive, Dropbox, and NAS systems
- **Git-Friendly Workflow**: Text-based files enable diffing and version control for engineering audits
- **Professional PDF Reports**: Typst-based PDF generation with custom fonts, logos, and structural seals
- **Comprehensive Material Databases**: Full wood species, steel specifications, and concrete specs
- **Code Compliance**: IBC specifications support (2012-2025)
- **Calculation Types**: Beams, columns, frames, shear walls, retaining walls, footings, and more

## Architecture

The project is structured as a Rust Workspace to ensure separation of concerns between the calculation engine and user interfaces.

```
stratify/
├── Cargo.toml              # Workspace root
├── assets/                 # Compiled-in assets (Fonts, Logos, Simpson XMLs)
├── calc_core/              # [LIB] Pure Rust. Math, JSON logic, PDF engine.
├── calc_gui/               # [BIN] Iced/WGPU application. Native + WASM.
└── calc_cli/               # [BIN] Ratatui interface. Terminal only.
```

### Component Breakdown

- **calc_core**: The source of truth. Contains all engineering logic, data structures, serialization, and PDF generation. No UI dependencies. Designed as an LLM-friendly API (see below).
- **calc_gui**: Full-featured GUI application using Iced framework. Compiles to native executables and WebAssembly.
- **calc_cli**: Terminal-based interface using Ratatui. Useful for quick calculations and batch processing.

### LLM API Design (calc_core)

The `calc_core` library is designed to be consumed by AI systems (Claude, GPT, Gemini, etc.) via MCP servers, function calling, or similar interfaces. Design principles:

1. **Self-Documenting API**: All public types and functions include comprehensive rustdoc comments with examples. Generated documentation serves as the LLM's reference.

2. **JSON-First Interface**: All inputs and outputs serialize cleanly to JSON. An LLM can construct a `BeamCalc` as JSON, pass it to the calculation engine, and receive structured results.

3. **Stateless Calculations**: Pure functions where possible. `calculate_beam(BeamInput) -> BeamResult` with no hidden state.

4. **Rich Error Types**: Errors return structured data explaining what went wrong, not just strings. LLMs can programmatically handle failures.

5. **Schema Export**: The library can export JSON Schema definitions for all input/output types, enabling LLMs to validate their requests before submission.

6. **Example-Heavy Documentation**: Every calculation type includes worked examples in the docs that LLMs can use as templates.

```rust
// Example: LLM-friendly beam calculation
let input = serde_json::from_str::<BeamInput>(r#"{
    "span_ft": 12.0,
    "uniform_load_plf": 150.0,
    "material": "DF-L No.2",
    "width_in": 1.5,
    "depth_in": 9.25
}"#)?;

let result = calc_core::beam::calculate(&input)?;
// Returns structured JSON with moment, shear, deflection, unity checks, pass/fail
```

This architecture enables future integration as an MCP server, giving AI assistants direct structural engineering calculation capabilities.

## Tech Stack

### Core Dependencies

- **Language**: Rust (Edition 2021)
- **Serialization**: `serde`, `serde_json` - Industry standard for JSON handling
- **File Locking**: `fs2` - Cross-platform file locking for network drives
- **UUID Generation**: `uuid` - Unique identifiers for calculation items
- **Asset Embedding**: `rust-embed` - Compile fonts, logos, and catalogs into binary

### GUI & Graphics

- **GUI Framework**: `iced` - Type-safe, cross-platform GUI framework
- **Graphics Backend**: `wgpu` - Portable WebGPU implementation (via Iced)
  - Native: Vulkan (Linux), Metal (macOS), DirectX 12 (Windows)
  - Web: WebGPU with WebGL2 fallback

### TUI

- **TUI Framework**: `ratatui` - Terminal user interface library

### PDF Generation

- **Typesetting**: `typst` - Modern, programmable typesetting system
  - Vector-perfect output
  - Mathematical typesetting
  - Template-based generation

### Math & Units

- **Units**: Raw `f64` with wrapper types (e.g., `struct Kips(f64)`) for type safety without verbosity

## Data Strategy

### File Format: `.stf` (Stratify Project)

Projects are stored as plain text JSON files. This design choice prioritizes:

1. **Human Readability**: Engineers can inspect and manually edit files if needed
2. **Version Control**: Git-friendly format enables diffing and auditing
3. **Recovery**: Corrupted files can be partially recovered from text
4. **Interoperability**: Easy to parse, validate, and migrate

### JSON Schema Structure

```json
{
  "meta": {
    "version": "1.0.0",
    "engineer": "Engineer Name",
    "job_id": "25-001",
    "client": "Client Name",
    "created": "2025-01-15T10:30:00Z",
    "modified": "2025-01-15T14:22:00Z"
  },
  "settings": {
    "code": "IBC2024",
    "seismic_design_cat": "D",
    "risk_category": "II",
    "default_materials": {
      "wood": "DF-L No.2",
      "steel": "A992"
    }
  },
  "items": {
    "b8d543-21a4-4e5f-9c12-123456789abc": {
      "type": "Beam",
      "label": "B-1",
      "span": 12.5,
      "loads": [...],
      "material": "..."
    },
    "c9e112-99b2-4a3d-8e7f-987654321def": {
      "type": "Column",
      "label": "C-1",
      "height": 10.0,
      "loads": [...]
    }
  }
}
```

**Design Note**: Items are stored in a flat map keyed by UUID rather than an array. This provides O(1) lookup for dependencies (e.g., "Beam A rests on Column B") and prevents duplicate IDs.

### Network Drive Safety

#### Atomic Saves

The application never writes directly to an open file. The save process:

1. Serialize `Project` struct to JSON in memory
2. Write to `project.tmp`
3. Flush to disk
4. Verify write integrity
5. Rename `project.tmp` to `project.stf`

This prevents corruption if the network connection drops during a save operation.

#### Sentinel Locking

When a user opens `project.stf`, the application immediately creates `~project.stf.lock` containing:
- User ID (from system)
- Timestamp
- Process ID

**Mechanism**: If another instance attempts to open the file, it detects the lock and forces "Read-Only" mode, preventing write conflicts.

**Google Drive Integration**: Lock files sync to the cloud, preventing write collisions from other engineers working on the same project.

### Git Workflow for Engineers

Because `.stf` files are plain text, they integrate seamlessly with version control systems. This solves the "filename_v1_final_rev2.doc" problem.

#### Workflow for Engineering Audits

1. **Initialize**: Create a Git repository in the job folder
2. **Commit Milestones**: When a calculation package is submitted to the city, commit the `.stf` file with a tag:
   ```bash
   git tag submittal_1
   git commit -m "Initial submittal package"
   ```
3. **Diffing (The Killer Feature)**: When a client requests changes, modify the file and save. Use a diff tool to compare:
   ```bash
   git diff HEAD~1 project.stf
   ```
   
   **Result**: You see exactly what changed in the math:
   ```diff
   - "span": 12.0,
   + "span": 14.0,
   - "moment_max": 15400,
   + "moment_max": 21300,
   ```

This eliminates "mystery changes" where a load was accidentally deleted or a parameter was modified without documentation.

#### Best Practices

- Commit after each major calculation milestone
- Use descriptive commit messages: "Updated beam B-1 span per client request"
- Tag releases: `submittal_1`, `submittal_2`, `final`
- Use branches for experimental designs: `feature/alternative-layout`

## Material Databases

### Current Status (MVP)

The MVP uses hardcoded placeholder values for common materials. This is sufficient for demonstrating the input→calculation→PDF pipeline. Full database integration is planned for post-MVP.

### Wood Species

- **Sawn Lumber**: DF-L, SP, HF, SPF, DF-S (NDS 2018 Table 4A values)
- **Grades**: Select Structural, No.1, No.2, No.3, Stud, Construction, Standard, Utility
- **Engineered**: Glulam, LVL, PSL (planned)
- **NDS Adjustment Factors**: C_D, C_M, C_t, C_F, C_r, etc.

### Steel Specifications

- **Materials**: A992, A36, A500, A572, etc. (planned)
- **Section Properties**: W-shapes, HSS, Channels, Angles, etc. (planned)
- **AISC 360**: Unity checks, slenderness ratios, etc.

### Concrete

- **Compressive Strengths**: f'c definitions (2500, 3000, 4000, 5000+ psi) (planned)
- **Rebar Sizes**: #3 through #18, standard grades (planned)

### Catalogs

- **Simpson Strong-Tie**: Hangers, hold-downs, portal frames, connectors (planned)
- **Format**: XML/CSV import for attachment options

### Future: Official Database Sources

**Goal**: Acquire official, licensed material databases for production use. The underlying data is industry-standard and publicly documented (though not always freely available in digital form).

#### Target Data Sources

| Source | Data | Format | Notes |
|--------|------|--------|-------|
| **AISC** | Steel shapes (W, HSS, C, L, etc.) | CSV/Database | Official shapes database available for purchase |
| **AWC/NDS** | Wood design values | PDF/Tables | NDS Supplement contains all reference values |
| **ACI** | Concrete/rebar properties | Standards docs | Values derived from ACI 318 |
| **Simpson Strong-Tie** | Connector capacities | XML/CSV | May provide official digital catalog |
| **USP Structural** | Connector capacities | TBD | Alternative connector manufacturer |
| **MiTek** | Connector capacities | TBD | Alternative connector manufacturer |

#### Implementation Strategy

1. **MVP Phase**: Hardcoded values for common materials (current)
2. **Beta Phase**: CSV/JSON files with manually transcribed values
3. **Production Phase**:
   - Contact manufacturers for official digital catalogs
   - Purchase AISC shapes database license
   - Implement automatic catalog update mechanism
   - Build tooling to convert official formats to our internal schema

#### Technical Notes

- **ENERCALC Observation**: ENERCALC uses `.tbl` binary format for fast lookups. This appears to be their proprietary conversion of industry-standard databases. The underlying data (AISC shapes, NDS values, etc.) is the same publicly documented information.
- **Build-Time Embedding**: Use `build.rs` or `phf` crate to compile databases into static HashMaps at compile time for zero-runtime file dependencies.
- **Catalog Updates**: Design system to accept catalog updates without code changes (versioned JSON/CSV files embedded at build time).

## Calculation Types

### Supported Calculations

- **Beams**: Simple span, multi-support, cantilever
- **Columns**: Axial, combined axial + bending
- **Frames**: Moment frames, braced frames
- **Shear Walls**: Overturning, sliding, anchorage
- **Diaphragms**: Chord forces, drag struts
- **Retaining Walls**: Overturning, sliding, bearing
- **Footings**: Spread footings, continuous footings
- **Posts**: Wood posts, steel posts

### Load Analysis

- **Load Combinations**: ASCE 7 (ASD/LRFD)
- **Wind Loads**: ASCE 7 Chapter 30
- **Seismic Loads**: ASCE 7 Chapter 12
- **Dead/Live Loads**: User-defined or code-prescribed

## PDF Reports

### Features

- **Professional Layout**: Title blocks, headers, footers
- **Custom Fonts**: User-provided fonts (TTF/OTF)
- **Company Branding**: Logo placement and structural seal options
- **Mathematical Typesetting**: Proper equations and formulas
- **Vector Graphics**: High-quality diagrams and charts
- **Multi-Page Support**: Automatic pagination

### Template System

PDFs are generated using Typst templates. Templates are defined as Rust string templates with injection points for calculation data. The system supports:

- Custom page layouts
- Reusable components (title blocks, tables)
- Conditional sections based on calculation type
- Dynamic content (loads, results, notes)

## Building & Running

### Prerequisites

- Rust 1.70+ (Edition 2021)
- Cargo

### Build Native Applications

```bash
# Build GUI (native)
cargo build --release --bin calc_gui

# Build CLI
cargo build --release --bin calc_cli
```

### Build WebAssembly

```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Build WASM
cargo build --release --target wasm32-unknown-unknown --bin calc_gui

# Use trunk or wasm-pack for bundling
trunk build --release
```

### Run

```bash
# GUI
./target/release/calc_gui

# CLI
./target/release/calc_cli
```

## Development Philosophy

### Design Principles

1. **Data Permanence**: Never lose data. Atomic saves, file locking, and human-readable formats ensure recoverability.
2. **Transparency**: Engineers should understand what the software is calculating. No black-box magic.
3. **Auditability**: Every change should be traceable. Git integration enables full audit trails.
4. **Performance**: GPU acceleration where beneficial, but never at the cost of correctness.
5. **Minimal Dependencies**: Use industry-standard libraries. Avoid "magic" frameworks that hide complexity.

### Code Organization

- **calc_core**: Pure Rust, zero UI dependencies. Can be used as a library by other tools.
- **calc_gui**: Imports `calc_core`. Handles all user interaction and rendering.
- **calc_cli**: Imports `calc_core`. Provides terminal interface for power users.

This architecture allows:
- Testing calculation logic independently of UI
- Building new interfaces without rewriting math
- Sharing code between GUI and CLI

## Contributing

This is a private/internal project. For questions or contributions, contact the project maintainers.

## License

[To be determined]

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed implementation phases and checklists.

## Future Features / TODO

### High Priority (Next Steps)

- [x] ~~PDF generation with Typst templates~~ ✅ DONE
- [ ] Basic GUI with Iced framework ← **NEXT PRIORITY**
- [ ] Steel section database (common W-shapes)
- [ ] Concrete material properties

### Material Database Automation

- [ ] **AISC Shapes Database**: Purchase official license, build import tooling
- [ ] **Simpson Strong-Tie Catalog**:
  - [ ] Search for official XML/CSV digital catalog from manufacturer
  - [ ] If unavailable, manually transcribe common connectors
  - [ ] Build automatic catalog update mechanism
- [ ] **AWC/NDS Database**: Automate extraction from NDS Supplement tables
- [ ] **Catalog Build Pipeline**: Create `build.rs` script to compile CSV/JSON databases into static Rust HashMaps at build time

### Engineering Features

- [ ] Full NDS adjustment factor implementation (C_D, C_M, C_t, C_L, C_F, C_fu, C_i, C_r, C_P, C_b)
- [ ] AISC 360 steel beam/column design
- [ ] ACI 318 concrete design
- [ ] Load combination generator (ASCE 7 ASD/LRFD)
- [ ] Continuous beam analysis (moment distribution or stiffness method)
- [ ] Frame analysis (matrix stiffness method)

### Platform Features

- [ ] WebAssembly deployment
- [ ] Cloud sync integration (beyond file locking)
- [ ] CAD interoperability (IFC import/export)
- [ ] MCP server for LLM integration

