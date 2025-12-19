# Next Steps

## WASM Browser Support: COMPLETE ✓

WebAssembly build now runs successfully in browsers with WebGPU support.

### WASM Implementation
- [x] Add wasm32-unknown-unknown target
- [x] Upgraded to **Iced 0.14** (uses wgpu 27.0 with proper WebGPU browser support)
- [x] Fixed dependencies for WASM:
  - `uuid` - added `js` feature for random number generation
  - `fs2` - made native-only with conditional compilation (FileLock stubbed for WASM)
  - `rfd` - file dialogs are native-only (stubbed for WASM with browser messages)
- [x] Added `console_error_panic_hook` for browser debugging
- [x] Created `index.html` for Trunk bundler with loading spinner
- [x] Configured WASM build to use only wgpu (no tiny-skia fallback)
- [x] All native tests still pass

### Build & Run WASM
```bash
# Install dependencies
rustup target add wasm32-unknown-unknown
cargo install trunk

# Build and serve
cd calc_gui
trunk serve --open
```

Opens at http://[::1]:8080 with full GUI functionality.

### Previous Issue (RESOLVED)
The canvas context conflict between wgpu and tiny_skia has been fixed by:
1. Upgrading to Iced 0.14 with wgpu 27.0 (proper WebGPU API compatibility)
2. Disabling tiny_skia fallback for WASM builds (wgpu-only configuration)

---

## Current Priority: Complete Wood Beams (Full Depth)

We're taking a **depth-first approach**: fully implement one element type before moving to others. This will:
1. Iron out data structures and patterns
2. Establish how to handle material databases (NDS species, grades, adjustment factors)
3. Create reusable patterns that carry over to columns, steel, concrete, etc.

### Wood Beam Feature Parity Checklist

**NDS Adjustment Factors:** ✓ Core module complete (`calc_core/src/nds_factors.rs`)
- [x] C_D - Load duration factor (with UI selection enum)
- [x] C_M - Wet service factor
- [x] C_t - Temperature factor
- [x] C_L - Beam stability factor (calculated from slenderness)
- [x] C_F - Size factor (depth-based per NDS Table 4A)
- [x] C_fu - Flat use factor
- [x] C_i - Incising factor
- [x] C_r - Repetitive member factor
- [ ] C_b - Bearing area factor (not needed for flexure)

**Adjustment Factor Integration:**
- [x] AdjustmentFactors struct with all factors
- [x] BeamInput includes adjustment_factors field
- [x] BeamResult includes AdjustmentSummary with all applied factors
- [x] Calculate function uses factors for Fb', Fv', E' adjustments
- [x] GUI UI to expose adjustment factor settings
  - Load Duration (C_D): Permanent, Normal, Snow, Construction, Wind/Seismic, Impact
  - Wet Service (C_M): Dry, Wet
  - Temperature (C_t): Normal, Elevated, High
  - Incising (C_i): Not Incised, Incised
  - Repetitive Member (C_r): Single, Repetitive (3+ @ ≤24" OC)
  - Flat Use (C_fu): Normal, Flat
  - Compression Edge Braced (C_L = 1.0): checkbox

**Load Handling:**
- [x] Point loads at specific locations (superposition analysis)
- [ ] Partial uniform loads (start/end positions) - formulas ready, UI pending
- [ ] Moment loads
- [ ] Multiple load cases with envelope results

**Span Conditions:** ✓ Core architecture complete
- [x] Simply supported (pin-pin)
- [x] Cantilever (fixed-free)
- [x] Fixed-fixed
- [x] Propped cantilever (fixed-pinned)
- [x] Continuous spans (2-span, 3-span, N-span) - Hardy Cross moment distribution
- [ ] GUI controls for multi-span configuration (pending)

**Section/Material:**
- [ ] Standard lumber sizes dropdown (2x4, 2x6, 2x8, 2x10, 2x12, etc.)
- [ ] Multiple-ply beams (2-2x10, 3-2x12, etc.)
- [x] NDS species/grade database (5 species, 8 grades)
- [ ] Notch and hole deductions

**Output/Reporting:**
- [x] Governing load combination display
- [x] Adjustment factor breakdown in results (AdjustmentSummary)
- [ ] Code reference citations (NDS section numbers)
- [x] Diagrams for shear, moment, deflection (single-span)
- [ ] Diagrams for continuous beams (multi-span)

---

## Multi-Span Continuous Beam Support: CORE COMPLETE ✓

Full support for multi-span continuous beams with configurable support conditions.

### New Data Model (`calc_core/src/calculations/continuous_beam.rs`)
- [x] `SupportType` enum: Free, Pinned, Roller, Fixed
- [x] `SpanSegment` struct: per-span length, width, depth, material
- [x] `ContinuousBeamInput`: N spans with N+1 support conditions
- [x] `ContinuousBeamResult`: span-by-span results + global extrema
- [x] Convenience constructors: `simple_span()`, `fixed_fixed()`, `cantilever()`
- [x] Migrated `CalculationItem::Beam` to use `ContinuousBeamInput`

### Analysis Engine
- [x] **Single-span closed-form solutions** (Roark's formulas):
  - Pin-pin (simply supported)
  - Fixed-fixed
  - Fixed-pinned (propped cantilever)
  - Fixed-free (cantilever)
- [x] **Multi-span Hardy Cross moment distribution** (`moment_distribution.rs`):
  - Distribution factor calculation based on span stiffnesses
  - Fixed-end moment calculation for uniform/point loads
  - Iterative distribution with 50% carryover
  - Convergence check (tolerance-based)
  - Exterior pinned support release

### Fixed-End Formulas (`calc_core/src/equations/beam.rs`)
- [x] `fem_uniform_full()` - FEM for full uniform load
- [x] `fem_point_load()` - FEM for point load at any position
- [x] `fem_partial_uniform()` - FEM for partial uniform load
- [x] Fixed-fixed beam: reactions, moments, shear, deflection
- [x] Cantilever beam: reactions, moments, shear, deflection
- [x] Propped cantilever: reactions, moments

### Integration
- [x] PDF generation updated for `ContinuousBeamInput`/`ContinuousBeamResult`
- [x] GUI updated to use new types (single-span mode)
- [x] All 216 tests pass (179 unit + 37 doc)

### Remaining for Multi-Span
- [ ] GUI: Add span list with +/- buttons
- [ ] GUI: Support type dropdown at each node
- [ ] GUI: Multi-span beam schematic
- [ ] GUI: Continuous M/V/δ diagrams across spans
- [ ] Show reactions and moments at each support

---

## What's Been Completed

### Phase 1 & 2 (Foundation)
- [x] Rust workspace structure (calc_core, calc_gui, calc_cli)
- [x] Core data structures (Project, ProjectMetadata, GlobalSettings)
- [x] Unit types (Feet, Inches, Kips, Psi, etc.)
- [x] Wood materials database (NDS values for DF-L, SP, HF, SPF, DF-S)
- [x] Engineered wood (Glulam, LVL, PSL) with stress classes
- [x] BeamInput/BeamResult with calculation logic
- [x] Simply-supported beam calculation (moment, shear, deflection, unity checks)
- [x] ASCE 7 load combinations (ASD and LRFD)
- [x] Discrete load system (point, uniform, partial - UI ready)
- [x] Atomic file I/O with locking
- [x] PDF generation with Typst
- [x] Full Iced GUI with live preview
- [x] Diagram rendering (beam schematic, shear, moment, deflection)
- [x] Comprehensive unit tests (216 passing: 179 unit + 37 doc)

### Recent Additions
- [x] Live preview - results update as you type
- [x] Immediate beam creation from Items Panel
- [x] Auto-save on every keystroke
- [x] GUI layout documentation (GUI_LAYOUT.md)
- [x] WASM compilation support (sanity check - runtime issue documented)
- [x] **NDS Adjustment Factors module** (`calc_core/src/nds_factors.rs`)
  - All 8 primary factors: C_D, C_M, C_t, C_L, C_F, C_fu, C_i, C_r
  - AdjustmentFactors struct with builder pattern
  - BeamStability calculator for C_L from slenderness
  - SizeFactor calculator for C_F from NDS Table 4A
  - AdjustmentSummary for detailed reporting
  - Integrated into beam calculation (Fb', Fv', E')
- [x] **Adjustment Factors GUI** (`calc_gui/src/main.rs`)
  - UI controls for all adjustment factors in beam editor
  - Load Duration, Wet Service, Repetitive Member (common)
  - Temperature, Incising, Flat Use (less common)
  - Compression edge braced checkbox for C_L
  - Factors saved/loaded with beam data
- [x] **Equations module** (`calc_core/src/equations/`)
  - Documented statics formulas with Roark's references
  - Sign conventions documented
  - Simply-supported, fixed-fixed, cantilever, propped cantilever formulas
- [x] **Multi-span continuous beam architecture**
  - ContinuousBeamInput/ContinuousBeamResult data model
  - Hardy Cross moment distribution solver
  - All boundary condition combinations supported
  - Integrated with load combinations and NDS factors
