# Next Steps

## Current Priority: WASM Sanity Check

Before going deeper on features, we need to validate that our architecture works on WebAssembly. This is a quick sanity check to ensure we're not building on a foundation that won't compile to web.

### WASM Test Checklist
- [ ] Add wasm32-unknown-unknown target
- [ ] Attempt `cargo build --target wasm32-unknown-unknown --bin calc_gui`
- [ ] Identify any dependencies that don't compile to WASM
- [ ] Test with Trunk if basic build works
- [ ] Document any blockers or required changes

## After WASM Verified: Complete Wood Beams (Full Depth)

We're taking a **depth-first approach**: fully implement one element type before moving to others. This will:
1. Iron out data structures and patterns
2. Establish how to handle material databases (NDS species, grades, adjustment factors)
3. Create reusable patterns that carry over to columns, steel, concrete, etc.

### Wood Beam Feature Parity Checklist

**NDS Adjustment Factors:**
- [ ] C_D - Load duration factor (with UI selection)
- [ ] C_M - Wet service factor
- [ ] C_t - Temperature factor
- [ ] C_L - Beam stability factor
- [ ] C_F - Size factor (depth-based)
- [ ] C_fu - Flat use factor
- [ ] C_i - Incising factor
- [ ] C_r - Repetitive member factor
- [ ] C_b - Bearing area factor

**Load Handling:**
- [ ] Point loads at specific locations (calculation, not just UI)
- [ ] Partial uniform loads (start/end positions)
- [ ] Moment loads
- [ ] Multiple load cases with envelope results

**Span Conditions:**
- [ ] Simply supported (current)
- [ ] Cantilever
- [ ] Continuous spans (2-span, 3-span)
- [ ] Fixed ends

**Section/Material:**
- [ ] Standard lumber sizes dropdown (2x4, 2x6, 2x8, 2x10, 2x12, etc.)
- [ ] Multiple-ply beams (2-2x10, 3-2x12, etc.)
- [ ] Full NDS species/grade database import
- [ ] Notch and hole deductions

**Output/Reporting:**
- [ ] Governing load combination display
- [ ] Adjustment factor breakdown in results
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
- [x] Comprehensive unit tests (101 passing)

### Recent Additions
- [x] Live preview - results update as you type
- [x] Immediate beam creation from Items Panel
- [x] Auto-save on every keystroke
- [x] GUI layout documentation (GUI_LAYOUT.md)
