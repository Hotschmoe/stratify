# Stratify Development Roadmap

This document outlines the phased development approach for Stratify, from initial foundation through production deployment.

## Current Status

**MVP COMPLETE!** ✓ (as of 2024-12-17)

The core input → calculation → PDF pipeline is functional:
- ✅ Beam input parameters
- ✅ NDS calculations (moment, shear, deflection, unity checks)
- ✅ Professional PDF report generation with Typst
- ✅ Project file save/load with atomic writes and file locking

**Next Priority**: GUI implementation with Iced framework

---

## Phase 1: Foundation & CLI (Weeks 1-4)

**Goal**: Establish the core architecture and create a working CLI that can calculate a simple beam and save/load safely.

### Week 1: Workspace Setup & Core Data Structures ✅ COMPLETE

- [x] **Initialize Rust Workspace**
  - [x] Create `Cargo.toml` workspace file
  - [x] Create `calc_core/` library crate
  - [x] Create `calc_gui/` binary crate
  - [x] Create `calc_cli/` binary crate
  - [x] Set up basic directory structure (`assets/`, `calc_core/src/`, etc.)

- [x] **Core: Project Data Structure**
  - [x] Define `Project` struct with `meta`, `settings`, `items` fields
  - [x] Implement `Serialize` and `Deserialize` for `Project`
  - [x] Define `ProjectMetadata` struct (version, engineer, job_id, client, timestamps)
  - [x] Define `GlobalSettings` struct (IBC year, risk category, default materials)
  - [x] Create `CalculationItem` enum with variants for each calculation type
  - [x] Implement UUID generation for calculation items

- [x] **Core: Basic Calculation Types**
  - [x] Define `BeamInput` struct (span, loads, material, size)
  - [x] Define `ColumnInput` struct (height, loads, material)
  - [x] Implement real beam calculation logic (NDS-based)
  - [x] Implement placeholder column calculation logic

### Week 2: File I/O & Locking ✅ COMPLETE

- [x] **Core: File Locking**
  - [x] Add `fs2` dependency
  - [x] Implement `FileLock` struct to manage `.lock` files
  - [x] Create `acquire()` function
  - [x] Lock released on drop
  - [x] Create `check()` function for read-only detection
  - [x] Handle lock file format (JSON with user_id, timestamp, pid, machine)
  - [x] Stale lock detection (process check, 24h timeout)

- [x] **Core: Atomic Save Logic**
  - [x] Implement `save_project()` function
  - [x] Write to `.tmp` file first
  - [x] Flush to disk (fsync)
  - [x] Rename `.tmp` to `.stf` (atomic)
  - [x] Handle errors gracefully

- [x] **Core: Load Logic**
  - [x] Implement `load_project()` function
  - [x] Check for lock file before loading
  - [x] Parse JSON and deserialize to `Project`
  - [x] Validate schema version compatibility
  - [x] Return lock info if locked by another user

### Week 3: Material Database Foundation ⚠️ PARTIAL

- [x] **Core: Material Database Structure**
  - [x] Define `WoodSpecies` enum (DF-L, SP, HF, SPF, DF-S)
  - [x] Define `WoodGrade` enum (Sel Str, No.1, No.2, No.3, Stud, Construction, Standard, Utility)
  - [x] Implement `WoodProperties` with NDS 2018 Table 4A values
  - [ ] Define `SteelGrade` enum (A992, A36, A500, etc.) - *planned*
  - [ ] Define `ConcreteStrength` struct (f'c values) - *planned*

- [ ] **Core: Static Material Tables**
  - [ ] Create CSV files for AISC steel sections (W-shapes, HSS, etc.) - *planned*
  - [x] Hardcoded NDS wood properties (MVP placeholder)
  - [ ] Use `build.rs` or `phf` to compile tables at build time - *planned*
  - [x] Implement lookup functions: `WoodProperties::lookup()`

- [x] **Core: Material Selection Logic**
  - [x] Implement material selection for calculations
  - [x] Store material references in calculation structs
  - [x] Validate material in calculation inputs

### Week 4: CLI Interface ⚠️ PLACEHOLDER ONLY

- [ ] **CLI: Ratatui Setup**
  - [ ] Add `ratatui` dependency
  - [ ] Create basic TUI layout (file list, calculation list, properties panel)
  - [ ] Implement event loop and key bindings

- [x] **CLI: Basic Beam Calculation**
  - [x] Simple stdin demo for beam parameters
  - [x] Call calculation logic from `calc_core`
  - [x] Display results in console
  - [ ] Full TUI interface - *planned*

- [x] **CLI: Testing**
  - [x] Test file locking with multiple instances
  - [x] Test atomic save
  - [x] Test JSON serialization/deserialization round-trip
  - [x] 37 unit tests passing
  - [x] 18 doc tests passing

## Phase 2: Graphics & Reporting (Weeks 5-8)

**Goal**: Generate professional PDF reports and render basic graphics in the GUI.

### Week 5: PDF Generation Foundation ✅ COMPLETE

- [x] **Core: Typst Integration**
  - [x] Add `typst` dependency (v0.14)
  - [x] Add `typst-pdf` dependency
  - [x] Add `typst-assets` for fonts
  - [x] Create `pdf.rs` module
  - [x] Implement `render_beam_pdf()` function

- [x] **Core: Template System**
  - [x] Create template for beam calculation report
  - [x] Professional layout (title block, tables, equations)
  - [x] Data injection via template replacement
  - [x] Pass/fail status box with color
  - [x] PDF generation tested and working (56KB output)

- [x] **Core: Asset Embedding**
  - [x] Fonts embedded via `typst-assets`
  - [ ] Custom font loading - *planned*
  - [ ] Logo embedding - *planned*

### Week 6: PDF Customization 🔲 NOT STARTED

- [ ] **Core: Custom Fonts**
  - [ ] Implement font loading from user files
  - [ ] Allow user-provided font files (TTF/OTF)
  - [ ] Pass font data to Typst compiler

- [ ] **Core: Logo & Seal Placement**
  - [ ] Define logo placement options (header, footer, custom position)
  - [ ] Implement seal image embedding
  - [ ] Create template variables for logo/seal positioning

- [ ] **Core: Report Layout Enhancements**
  - [ ] Add company letterhead support
  - [ ] Create table templates for loads and results
  - [ ] Implement multi-page support for multiple calculations
  - [ ] Add page numbering and headers/footers

### Week 7: GUI Foundation 🔲 NOT STARTED (NEXT PRIORITY)

- [ ] **GUI: Iced Setup**
  - [ ] Add `iced` dependency
  - [ ] Create basic application shell
  - [ ] Set up window creation and event loop
  - [ ] Test native compilation (Windows)

- [ ] **GUI: Layout Structure**
  - [ ] Create "Project Explorer" sidebar (list of calculations)
  - [ ] Create "Properties" panel (input fields)
  - [ ] Create "Results" panel
  - [ ] Implement resizable panels

- [ ] **GUI: Basic Widgets**
  - [ ] Create text input widgets for beam parameters
  - [ ] Create dropdown for material selection
  - [ ] Create button widgets (Calculate, Save, Export PDF)
  - [ ] Implement basic styling

### Week 8: Graphics Rendering 🔲 NOT STARTED

- [ ] **GUI: Diagram Rendering**
  - [ ] Create custom canvas for beam diagrams
  - [ ] Implement shear diagram rendering
  - [ ] Implement moment diagram rendering
  - [ ] Implement deflection diagram rendering

- [ ] **GUI: File Operations**
  - [ ] Implement file menu (New, Open, Save, Save As, Export PDF)
  - [ ] Integrate with `calc_core` file I/O functions
  - [ ] Display lock status in status bar

## Phase 3: Engineering Library (Months 3-6)

**Goal**: Implement comprehensive structural engineering calculations and code compliance.

### Month 3: Code Compliance & Load Combinations 🔲 NOT STARTED

- [ ] **Core: IBC Code Support**
  - [ ] Define `IBCVersion` enum (2012, 2015, 2018, 2021, 2024)
  - [ ] Create code-specific constants (wind speeds, seismic factors)
  - [ ] Implement code selection in `GlobalSettings`

- [ ] **Core: Load Combinations**
  - [ ] Define `LoadType` enum (Dead, Live, Wind, Seismic, Snow, etc.)
  - [ ] Implement ASCE 7 load combination generator (ASD)
  - [ ] Implement ASCE 7 load combination generator (LRFD)

### Month 4: Wood Design (NDS) ⚠️ PARTIAL

- [x] **Core: Basic NDS Implementation**
  - [x] Bending stress calculations (fb vs Fb)
  - [x] Shear stress calculations (fv vs Fv)
  - [x] Deflection calculations (L/240, L/360)
  - [x] Unity check functions

- [ ] **Core: Full NDS Adjustment Factors**
  - [ ] Implement C_D (load duration factor)
  - [ ] Implement C_M (wet service factor)
  - [ ] Implement C_t (temperature factor)
  - [ ] Implement C_L (beam stability factor)
  - [ ] Implement C_F (size factor)
  - [ ] Implement C_fu (flat use factor)
  - [ ] Implement C_i (incising factor)
  - [ ] Implement C_r (repetitive member factor)
  - [ ] Implement C_P (column stability factor - full NDS 3.7)
  - [ ] Implement C_b (bearing area factor)

- [ ] **Core: Wood Column Design (Full)**
  - [ ] Full column stability factor (C_P) per NDS 3.7
  - [ ] Combined axial + bending checks
  - [ ] Euler buckling calculations

- [ ] **Core: Engineered Wood**
  - [ ] Add Glulam properties and design
  - [ ] Add LVL properties and design
  - [ ] Add PSL properties and design

### Month 5: Steel Design (AISC 360) 🔲 NOT STARTED

- [ ] **Core: Steel Section Properties**
  - [ ] Complete AISC database (all W-shapes, HSS, Channels, Angles)
  - [ ] Implement section property lookup

- [ ] **Core: Steel Beam Design**
  - [ ] Implement LRFD flexural strength
  - [ ] Implement LRFD shear strength
  - [ ] Implement lateral-torsional buckling
  - [ ] Implement deflection checks

- [ ] **Core: Steel Column Design**
  - [ ] Implement column strength calculations
  - [ ] Calculate effective length factors (K)
  - [ ] Implement combined axial + bending

### Month 6: Concrete Design & Advanced Calculations 🔲 NOT STARTED

- [ ] **Core: ACI 318 Implementation**
  - [ ] Concrete material properties
  - [ ] Basic flexural design (beams)
  - [ ] Shear design (stirrups)

- [ ] **Core: Footing Design**
  - [ ] Spread footing design
  - [ ] Continuous footing design

- [ ] **Core: Retaining Wall Design**
  - [ ] Earth pressure calculations
  - [ ] Stability checks

- [ ] **Core: Simpson Catalog Integration**
  - [ ] Parse Simpson XML/CSV catalog data
  - [ ] Create connector database
  - [ ] Implement connector selection logic

## Phase 4: Production & Polish (Month 7+)

### Month 7: WebAssembly Deployment 🔲 NOT STARTED

- [ ] Set up WASM target compilation
- [ ] Configure Iced for WASM
- [ ] Test in browsers (Chrome, Firefox, Safari)
- [ ] Deploy to web hosting

### Month 8: UI/UX Polish 🔲 NOT STARTED

- [ ] Dark/Light mode
- [ ] Keyboard shortcuts
- [ ] Undo/redo
- [ ] Input validation and error messages

### Month 9: Documentation & Testing 🔲 NOT STARTED

- [ ] User guide
- [ ] Developer guide
- [ ] Comprehensive test coverage

### Month 10+: Production Readiness 🔲 NOT STARTED

- [ ] Installer packages
- [ ] Performance optimization
- [ ] Security review

## Future Enhancements (Post-1.0)

- [ ] 3D visualization of structures
- [ ] Finite element analysis (FEA) integration
- [ ] Cloud sync
- [ ] Collaborative editing
- [ ] Mobile app
- [ ] API for programmatic access
- [ ] Plugin system for custom calculations
- [ ] CAD integration (IFC, Revit)

## Notes

- **Dependency Minimization**: All dependencies should be carefully evaluated. Prefer well-maintained, industry-standard libraries over "magic" frameworks.
- **Testing**: Each phase should include testing milestones. Don't move to the next phase until current phase tests pass.
- **Documentation**: Update README and documentation as features are added, not at the end.
- **Git Workflow**: Use feature branches for major features. Main branch should always be stable.
