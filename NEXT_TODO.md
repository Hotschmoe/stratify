# Next Steps

## WASM Sanity Check: COMPLETE ✓

We validated that our architecture compiles to WebAssembly. The build works but there's a runtime issue to address later.

### WASM Test Results
- [x] Add wasm32-unknown-unknown target
- [x] `cargo build --target wasm32-unknown-unknown --bin calc_gui` - SUCCESS
- [x] Fixed dependencies for WASM:
  - `uuid` - added `js` feature for random number generation
  - `fs2` - made native-only with conditional compilation (FileLock stubbed for WASM)
  - `rfd` - file dialogs are native-only (stubbed for WASM with browser messages)
- [x] Added `console_error_panic_hook` for browser debugging
- [x] Created `index.html` for Trunk bundler
- [x] All 101 native tests still pass

### Known WASM Runtime Issue
**Status:** Compiles but crashes at runtime in browser

**Error:** Iced's fallback renderer (tiny_skia) conflicts with wgpu canvas context
```
panicked at iced_tiny_skia: "A canvas context other than `CanvasRenderingContext2d` was already created"
```

**Root Cause:** When wgpu's `request_adapter()` returns None (WebGPU not fully available), Iced falls back to tiny_skia which tries to create a 2D canvas context on the same canvas element that wgpu touched.

**Solutions (for later):**
1. Add `webgl` feature to Iced (enables WebGL2 fallback within wgpu, avoids tiny_skia)
2. Investigate why `request_adapter()` returns None on browsers that support WebGPU
3. Configure Iced to disable tiny_skia fallback for WebGPU-only builds

**Decision:** WASM is not priority. Compiles successfully. Will address runtime issue when WASM becomes focus.

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
- [ ] Point loads at specific locations (calculation, not just UI)
- [ ] Partial uniform loads (start/end positions)
- [ ] Moment loads
- [ ] Multiple load cases with envelope results

**Span Conditions:**
- [x] Simply supported (current)
- [ ] Cantilever
- [ ] Continuous spans (2-span, 3-span)
- [ ] Fixed ends

**Section/Material:**
- [ ] Standard lumber sizes dropdown (2x4, 2x6, 2x8, 2x10, 2x12, etc.)
- [ ] Multiple-ply beams (2-2x10, 3-2x12, etc.)
- [x] NDS species/grade database (5 species, 8 grades)
- [ ] Notch and hole deductions

**Output/Reporting:**
- [x] Governing load combination display
- [x] Adjustment factor breakdown in results (AdjustmentSummary)
- [ ] Code reference citations (NDS section numbers)
- [ ] Diagrams for non-uniform loads

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
- [x] Comprehensive unit tests (140 passing: 112 unit + 28 doc)

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
