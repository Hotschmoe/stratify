# Stratify Development Roadmap

This document outlines the phased development approach for Stratify, from initial foundation through production deployment.

## Phase 1: Foundation & CLI (Weeks 1-4)

**Goal**: Establish the core architecture and create a working CLI that can calculate a simple beam and save/load safely.

### Week 1: Workspace Setup & Core Data Structures

- [ ] **Initialize Rust Workspace**
  - [ ] Create `Cargo.toml` workspace file
  - [ ] Create `calc_core/` library crate
  - [ ] Create `calc_gui/` binary crate
  - [ ] Create `calc_cli/` binary crate
  - [ ] Set up basic directory structure (`assets/`, `calc_core/src/`, etc.)

- [ ] **Core: Project Data Structure**
  - [ ] Define `Project` struct with `meta`, `settings`, `items` fields
  - [ ] Implement `Serialize` and `Deserialize` for `Project`
  - [ ] Define `ProjectMetadata` struct (version, engineer, job_id, client, timestamps)
  - [ ] Define `GlobalSettings` struct (IBC year, risk category, default materials)
  - [ ] Create `CalculationEnum` enum with variants for each calculation type
  - [ ] Implement UUID generation for calculation items

- [ ] **Core: Basic Calculation Types**
  - [ ] Define `BeamCalc` struct (span, loads, material, supports)
  - [ ] Define `ColumnCalc` struct (height, loads, material)
  - [ ] Create placeholder calculation logic (return dummy results)

### Week 2: File I/O & Locking

- [ ] **Core: File Locking**
  - [ ] Add `fs2` dependency
  - [ ] Implement `LockFile` struct to manage `.lock` files
  - [ ] Create `acquire_lock()` function
  - [ ] Create `release_lock()` function
  - [ ] Create `check_lock()` function for read-only detection
  - [ ] Handle lock file format (JSON with user_id, timestamp, pid)

- [ ] **Core: Atomic Save Logic**
  - [ ] Implement `save_project()` function
  - [ ] Write to `.tmp` file first
  - [ ] Verify write integrity (checksum or size check)
  - [ ] Flush to disk
  - [ ] Rename `.tmp` to `.stf`
  - [ ] Handle errors gracefully (rollback on failure)

- [ ] **Core: Load Logic**
  - [ ] Implement `load_project()` function
  - [ ] Check for lock file before loading
  - [ ] Parse JSON and deserialize to `Project`
  - [ ] Validate schema version compatibility
  - [ ] Return read-only flag if lock detected

### Week 3: Material Database Foundation

- [ ] **Core: Material Database Structure**
  - [ ] Create `MaterialDatabase` trait/struct
  - [ ] Define `WoodSpecies` enum (DF-L, SP, HF, etc.)
  - [ ] Define `WoodGrade` enum (Sel Str, No.1, No.2, etc.)
  - [ ] Define `SteelGrade` enum (A992, A36, A500, etc.)
  - [ ] Define `ConcreteStrength` struct (f'c values)

- [ ] **Core: Static Material Tables**
  - [ ] Create CSV files for AISC steel sections (W-shapes, HSS, etc.)
  - [ ] Create CSV files for NDS wood properties
  - [ ] Use `build.rs` or `phf` to compile tables into static HashMaps at compile time
  - [ ] Implement lookup functions: `get_steel_section()`, `get_wood_properties()`
  - [ ] Ensure zero-runtime dependencies (no database files needed)

- [ ] **Core: Material Selection Logic**
  - [ ] Implement material selection for calculations
  - [ ] Store material references in calculation structs
  - [ ] Validate material compatibility with calculation type

### Week 4: CLI Interface

- [ ] **CLI: Ratatui Setup**
  - [ ] Add `ratatui` dependency
  - [ ] Create basic TUI layout (file list, calculation list, properties panel)
  - [ ] Implement event loop and key bindings

- [ ] **CLI: File Operations**
  - [ ] Implement "New Project" command
  - [ ] Implement "Open Project" command (with lock detection)
  - [ ] Implement "Save Project" command (with atomic save)
  - [ ] Display lock status in UI

- [ ] **CLI: Basic Beam Calculation**
  - [ ] Create form to input beam parameters (span, loads, material)
  - [ ] Call calculation logic from `calc_core`
  - [ ] Display results in TUI
  - [ ] Save beam to project file

- [ ] **CLI: Testing**
  - [ ] Test file locking with multiple instances
  - [ ] Test atomic save on simulated network interruption
  - [ ] Test JSON serialization/deserialization round-trip

## Phase 2: Graphics & Reporting (Weeks 5-8)

**Goal**: Generate professional PDF reports and render basic graphics in the GUI.

### Week 5: PDF Generation Foundation

- [ ] **Core: Typst Integration**
  - [ ] Add `typst` dependency (library version)
  - [ ] Create `typst_templates.rs` module
  - [ ] Define basic report template string (title, content, footer)
  - [ ] Implement `render_pdf()` function that takes `Project` and returns `Vec<u8>`

- [ ] **Core: Template System**
  - [ ] Create template for beam calculation report
  - [ ] Define template variables (beam_label, span, loads, results)
  - [ ] Implement data injection logic (map `BeamCalc` to template variables)
  - [ ] Test PDF generation for simple beam

- [ ] **Core: Asset Embedding**
  - [ ] Add `rust-embed` dependency
  - [ ] Create `assets/fonts/` directory
  - [ ] Embed default font (Inter or Roboto Mono) using `rust-embed`
  - [ ] Create placeholder logo image
  - [ ] Embed logo in binary
  - [ ] Implement font/logo access functions

### Week 6: PDF Customization

- [ ] **Core: Custom Fonts**
  - [ ] Implement font loading from embedded assets
  - [ ] Allow user-provided font files (TTF/OTF)
  - [ ] Pass font data to Typst compiler
  - [ ] Test custom font rendering

- [ ] **Core: Logo & Seal Placement**
  - [ ] Define logo placement options (header, footer, custom position)
  - [ ] Implement seal image embedding
  - [ ] Create template variables for logo/seal positioning
  - [ ] Test logo rendering in PDF

- [ ] **Core: Report Layout**
  - [ ] Design title block template
  - [ ] Create table templates for loads and results
  - [ ] Implement multi-page support
  - [ ] Add page numbering and headers/footers

### Week 7: GUI Foundation

- [ ] **GUI: Iced Setup**
  - [ ] Add `iced` dependency
  - [ ] Create basic application shell
  - [ ] Set up window creation and event loop
  - [ ] Test native compilation (Windows, macOS, Linux)

- [ ] **GUI: Layout Structure**
  - [ ] Create "Project Explorer" sidebar (list of calculations)
  - [ ] Create "Properties" panel (input fields)
  - [ ] Create "Canvas" area (for diagrams)
  - [ ] Implement resizable panels

- [ ] **GUI: Basic Widgets**
  - [ ] Create text input widgets for beam parameters
  - [ ] Create dropdown for material selection
  - [ ] Create button widgets (Calculate, Save, Export PDF)
  - [ ] Implement basic styling (colors, fonts)

### Week 8: Graphics Rendering

- [ ] **GUI: Canvas Widget**
  - [ ] Create custom `Canvas` widget using Iced's canvas API
  - [ ] Implement coordinate system transformation
  - [ ] Draw basic beam diagram (line with supports)

- [ ] **GUI: Diagram Rendering**
  - [ ] Implement shear diagram rendering (from calculation results)
  - [ ] Implement moment diagram rendering
  - [ ] Implement deflection diagram rendering
  - [ ] Add axis labels and grid lines
  - [ ] Add zoom/pan functionality

- [ ] **GUI: File Operations**
  - [ ] Implement file menu (New, Open, Save, Save As)
  - [ ] Integrate with `calc_core` file I/O functions
  - [ ] Display lock status in status bar
  - [ ] Handle read-only mode UI indication

## Phase 3: Engineering Library (Months 3-6)

**Goal**: Implement comprehensive structural engineering calculations and code compliance.

### Month 3: Code Compliance & Load Combinations

- [ ] **Core: IBC Code Support**
  - [ ] Define `IBCVersion` enum (2012, 2015, 2018, 2021, 2024)
  - [ ] Create code-specific constants (wind speeds, seismic factors)
  - [ ] Implement code selection in `GlobalSettings`
  - [ ] Create code lookup functions

- [ ] **Core: Load Combinations**
  - [ ] Define `LoadType` enum (Dead, Live, Wind, Seismic, Snow, etc.)
  - [ ] Define `LoadCase` struct (magnitude, type, location)
  - [ ] Implement ASCE 7 load combination generator (ASD)
  - [ ] Implement ASCE 7 load combination generator (LRFD)
  - [ ] Create `apply_load_combinations()` function

- [ ] **Core: Wind Load Calculations**
  - [ ] Implement ASCE 7 Chapter 30 wind pressure calculations
  - [ ] Calculate wind loads on walls and roofs
  - [ ] Handle exposure categories (B, C, D)
  - [ ] Calculate gust effect factors

### Month 4: Wood Design (NDS)

- [ ] **Core: NDS Adjustment Factors**
  - [ ] Implement C_D (load duration factor)
  - [ ] Implement C_M (wet service factor)
  - [ ] Implement C_t (temperature factor)
  - [ ] Implement C_F (size factor)
  - [ ] Implement C_r (repetitive member factor)
  - [ ] Implement C_i (incising factor)
  - [ ] Calculate adjusted design values

- [ ] **Core: Wood Beam Design**
  - [ ] Implement bending stress calculations (F_b)
  - [ ] Implement shear stress calculations (F_v)
  - [ ] Implement deflection calculations
  - [ ] Implement bearing calculations
  - [ ] Create unity check functions

- [ ] **Core: Wood Column Design**
  - [ ] Implement column stability factor (C_P)
  - [ ] Calculate slenderness ratios
  - [ ] Implement axial capacity calculations
  - [ ] Implement combined axial + bending checks

- [ ] **Core: Engineered Wood**
  - [ ] Add Glulam properties and design
  - [ ] Add LVL properties and design
  - [ ] Add PSL properties and design

### Month 5: Steel Design (AISC 360)

- [ ] **Core: Steel Section Properties**
  - [ ] Complete AISC database (all W-shapes, HSS, Channels, Angles)
  - [ ] Implement section property lookup
  - [ ] Calculate section moduli, moments of inertia

- [ ] **Core: Steel Beam Design**
  - [ ] Implement LRFD flexural strength (phi*M_n)
  - [ ] Implement LRFD shear strength (phi*V_n)
  - [ ] Implement lateral-torsional buckling calculations
  - [ ] Implement deflection checks
  - [ ] Create unity check functions

- [ ] **Core: Steel Column Design**
  - [ ] Implement column strength calculations (phi*P_n)
  - [ ] Calculate effective length factors (K)
  - [ ] Implement slenderness ratio checks
  - [ ] Implement combined axial + bending (interaction equations)

- [ ] **Core: Steel Connection Design**
  - [ ] Basic weld capacity calculations
  - [ ] Basic bolt capacity calculations
  - [ ] Connection design helpers

### Month 6: Concrete Design & Advanced Calculations

- [ ] **Core: ACI 318 Implementation**
  - [ ] Define concrete material properties (f'c, reinforcement grades)
  - [ ] Implement basic flexural design (beams)
  - [ ] Implement shear design (stirrups)

- [ ] **Core: Spread Footing Design**
  - [ ] Calculate bearing capacity
  - [ ] Calculate required footing size
  - [ ] Calculate required reinforcement
  - [ ] Check one-way and two-way shear

- [ ] **Core: Continuous Footing Design**
  - [ ] Calculate required width
  - [ ] Calculate required reinforcement (longitudinal and transverse)
  - [ ] Check bearing and shear

- [ ] **Core: Retaining Wall Design**
  - [ ] Calculate earth pressures (active, passive)
  - [ ] Check overturning stability
  - [ ] Check sliding stability
  - [ ] Check bearing capacity
  - [ ] Calculate required reinforcement

- [ ] **Core: Frame Analysis**
  - [ ] Implement matrix stiffness method (basic)
  - [ ] Calculate member forces in frames
  - [ ] Support for moment frames and braced frames

- [ ] **Core: Shear Wall Design**
  - [ ] Calculate overturning moments
  - [ ] Calculate sliding forces
  - [ ] Design anchorage
  - [ ] Check boundary element requirements

- [ ] **Core: Diaphragm Design**
  - [ ] Calculate chord forces
  - [ ] Calculate drag strut forces
  - [ ] Design connections

- [ ] **Core: Simpson Catalog Integration**
  - [ ] Parse Simpson XML/CSV catalog data
  - [ ] Create connector database
  - [ ] Implement connector selection logic
  - [ ] Add connector capacity checks

## Phase 4: Production & Polish (Month 7+)

**Goal**: Deploy to web, polish UI/UX, and prepare for production use.

### Month 7: WebAssembly Deployment

- [ ] **WASM: File I/O Refactoring**
  - [ ] Create platform-specific file I/O abstraction trait
  - [ ] Implement native file I/O (using `std::fs`)
  - [ ] Implement WASM file I/O (using File System Access API)
  - [ ] Add fallback for browsers without File System Access API (download blob, file input)

- [ ] **WASM: Build Configuration**
  - [ ] Set up `wasm32-unknown-unknown` target compilation
  - [ ] Configure `wasm-bindgen` for Iced
  - [ ] Create HTML shell for WASM app
  - [ ] Test WASM build locally

- [ ] **WASM: Browser Testing**
  - [ ] Test in Chrome/Edge (WebGPU support)
  - [ ] Test in Firefox (WebGL2 fallback)
  - [ ] Test in Safari (WebGL2 fallback)
  - [ ] Verify file save/load in browser
  - [ ] Test performance with large projects

- [ ] **WASM: Deployment**
  - [ ] Set up hosting (GitHub Pages, Netlify, or similar)
  - [ ] Configure build pipeline
  - [ ] Test production deployment

### Month 8: UI/UX Polish

- [ ] **GUI: Styling Pass**
  - [ ] Implement dark mode
  - [ ] Implement light mode
  - [ ] Create consistent color scheme
  - [ ] Improve typography
  - [ ] Add icons for common actions

- [ ] **GUI: User Experience**
  - [ ] Add keyboard shortcuts (Ctrl+S, Ctrl+O, etc.)
  - [ ] Implement undo/redo functionality
  - [ ] Add calculation validation and error messages
  - [ ] Improve form layouts and input validation
  - [ ] Add tooltips and help text

- [ ] **GUI: Advanced Features**
  - [ ] Implement calculation templates
  - [ ] Add copy/paste for calculations
  - [ ] Implement calculation dependencies (beam on column)
  - [ ] Add calculation grouping/folders

- [ ] **CLI: Enhancements**
  - [ ] Add batch processing mode
  - [ ] Add export to CSV/JSON
  - [ ] Improve TUI layout and navigation

### Month 9: Documentation & Testing

- [ ] **Documentation: User Guide**
  - [ ] Write "Getting Started" guide
  - [ ] Document file format specification
  - [ ] Create "Engineering with Git" workflow guide
  - [ ] Write calculation type documentation
  - [ ] Create video tutorials (optional)

- [ ] **Documentation: Developer Guide**
  - [ ] Document architecture decisions
  - [ ] Document adding new calculation types
  - [ ] Document material database format
  - [ ] Document PDF template system

- [ ] **Testing: Unit Tests**
  - [ ] Write tests for calculation logic
  - [ ] Write tests for file I/O
  - [ ] Write tests for material lookups
  - [ ] Achieve >80% code coverage

- [ ] **Testing: Integration Tests**
  - [ ] Test full calculation workflows
  - [ ] Test file locking scenarios
  - [ ] Test PDF generation for all calculation types
  - [ ] Test cross-platform compatibility

- [ ] **Testing: Google Drive Integration**
  - [ ] Test offline file behavior
  - [ ] Test lock file syncing
  - [ ] Test atomic saves on network interruption
  - [ ] Document Google Drive best practices

### Month 10+: Production Readiness

- [ ] **Assets: Finalization**
  - [ ] Finalize font licensing
  - [ ] Create default logo placeholders
  - [ ] Verify all embedded assets load correctly

- [ ] **Performance: Optimization**
  - [ ] Profile calculation performance
  - [ ] Optimize PDF generation speed
  - [ ] Optimize GUI rendering (large projects)
  - [ ] Minimize WASM binary size

- [ ] **Security: Review**
  - [ ] Review file I/O for path traversal vulnerabilities
  - [ ] Sanitize user inputs
  - [ ] Review JSON deserialization for malicious input
  - [ ] Add file format version validation

- [ ] **Release: Preparation**
  - [ ] Create installer packages (Windows, macOS, Linux)
  - [ ] Set up auto-update mechanism (optional)
  - [ ] Create release notes template
  - [ ] Prepare marketing materials (if needed)

## Future Enhancements (Post-1.0)

- [ ] 3D visualization of structures
- [ ] Finite element analysis (FEA) integration
- [ ] Cloud sync (beyond Google Drive)
- [ ] Collaborative editing (multiple engineers)
- [ ] Mobile app (iOS/Android)
- [ ] API for programmatic access
- [ ] Plugin system for custom calculations
- [ ] Integration with CAD software
- [ ] Automated code checking and suggestions

## Notes

- **Dependency Minimization**: All dependencies should be carefully evaluated. Prefer well-maintained, industry-standard libraries over "magic" frameworks.
- **Testing**: Each phase should include testing milestones. Don't move to the next phase until current phase tests pass.
- **Documentation**: Update README and documentation as features are added, not at the end.
- **Git Workflow**: Use feature branches for major features. Main branch should always be stable.
- **Code Review**: All code should be reviewed before merging to main (even for solo developers, self-review is valuable).

