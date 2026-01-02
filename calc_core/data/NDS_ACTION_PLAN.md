# NDS Data Architecture Action Plan

> Central tracking document for TOML-based materials data migration.
> Last updated: 2026-01-02

## Architecture Overview

```
TOML Source Files (human-readable)
        |
        v
    build.rs (compile time)
        |
        v
  Generated Rust Code (zero runtime overhead)
        |
        v
  calc_core::generated module
```

## Phase Status

| Phase | Bead ID | Title | Status | Blocked By |
|-------|---------|-------|--------|------------|
| 1 | Stratify-cey | Infrastructure setup | CLOSED | - |
| 2 | Stratify-idi | Migrate existing data | OPEN | Phase 1 (done) |
| 3 | Stratify-g0o | Expand NDS coverage | OPEN | Stratify-q38 (licensing) |
| 4 | Stratify-4zy | Manufacturer data | OPEN | - |

## Related Beads

| Bead ID | Title | Status | Notes |
|---------|-------|--------|-------|
| Stratify-040 | Research and integrate authoritative wood design values | IN_PROGRESS | Parent epic |
| Stratify-q38 | Contact AWC about NDS data licensing | OPEN | Blocks Phase 3 |

---

## Phase 1: Infrastructure [COMPLETE]

**Bead:** `Stratify-cey` (CLOSED)

### Deliverables
- [x] Directory structure: `calc_core/data/wood/`
- [x] `build.rs` - compiles TOML to Rust at build time
- [x] `sources.toml` - provenance tracking
- [x] TOML schema templates (sawn lumber, LVL, PSL)
- [x] `TRANSCRIPTION_GUIDE.md`
- [x] `src/generated/mod.rs` - includes generated code
- [x] `pdf-extractor` binary tool
- [x] Added `once_cell` and `pdf-extract` dependencies

### Files Created
```
calc_core/
  build.rs
  data/
    TRANSCRIPTION_GUIDE.md
    NDS_ACTION_PLAN.md (this file)
    wood/
      sources.toml
      sawn_lumber/
        nds_2018_table4a.toml
      engineered/
        lvl_generic.toml
        psl_generic.toml
        manufacturer/.gitkeep
  src/
    bin/pdf_extractor.rs
    generated/mod.rs
```

---

## Phase 2: Migrate Existing Data [TODO]

**Bead:** `Stratify-idi` (OPEN)

### Goal
Replace hardcoded Rust match statements with TOML-generated code.

### Tasks
- [ ] Verify `nds_2018_table4a.toml` values match `sawn_lumber.rs`
- [ ] Export glulam data to `engineered/glulam_nds.toml`
- [ ] Verify LVL/PSL TOML matches `engineered_wood.rs`
- [ ] Export lumber sizes to `lumber_sizes.toml`
- [ ] Update `materials/` modules to use generated code
- [ ] Remove hardcoded match statements
- [ ] Run full test suite to verify equivalence

### Current Hardcoded Locations
| File | Data | Lines (approx) |
|------|------|----------------|
| `materials/sawn_lumber.rs` | NDS 2018 Table 4A | 180-350 |
| `materials/engineered_wood.rs` | Glulam, LVL, PSL | 130-480 |
| `materials/lumber_sizes.rs` | Nominal to actual | 50-150 |

### Verification Command
```bash
cargo test -p calc_core
# All 40 tests must pass after migration
```

---

## Phase 3: Expand NDS Coverage [BLOCKED]

**Bead:** `Stratify-g0o` (OPEN)
**Blocked by:** `Stratify-q38` (AWC licensing contact)

### Goal
Transcribe complete NDS Supplement tables after licensing.

### Tables to Transcribe
- [ ] Table 4A - Full sawn lumber (all species/grades)
- [ ] Table 4B - Timbers (5x5+)
- [ ] Table 4C - Decking
- [ ] Table 4D - Machine Stress Rated (MSR)
- [ ] Table 4E - Machine Evaluated Lumber (MEL)
- [ ] Table 5A/5B - Glulam (all combinations)
- [ ] Adjustment factor tables

### Licensing Status
- **Contact:** info@awc.org
- **Questions to ask:**
  1. Data licensing options for software developers?
  2. Available formats (JSON, CSV, database)?
  3. Costs and terms?
  4. Updates for future NDS editions?
- **Reference:** Other software (SkyCiv, BeamChek, ASDIP) has NDS values

### Action Required
```bash
# Check/update licensing bead status
bd show Stratify-q38
```

---

## Phase 4: Manufacturer Data [TODO]

**Bead:** `Stratify-4zy` (OPEN)

### Goal
Add manufacturer-specific LVL/PSL/LSL design values.

### Available Sources

| Source | Status | Notes |
|--------|--------|-------|
| NDS 2015 Supplement PDF | ENCRYPTED | Cannot extract programmatically |
| Weyerhaeuser Trus Joist | Need to obtain | ESR-1153 (verify current) |
| LP SolidStart | Need to obtain | ESR-2403 (verify current) |

### PDF Extraction Tool
```bash
# Attempt extraction (will fail on encrypted PDFs)
cargo run --bin pdf-extractor -- "codes/2015/AWC NDS 2015 Supplement.pdf"
```

### Tasks
- [ ] Obtain Weyerhaeuser Specifier's Guide
- [ ] Obtain LP SolidStart Specifier's Guide
- [ ] Transcribe Microllam LVL values
- [ ] Transcribe Parallam PSL values
- [ ] Transcribe LP LVL values
- [ ] Transcribe LSL values (if available)
- [ ] Add I-joist depth tables (TJI series)

### Output Files
```
data/wood/engineered/manufacturer/
  weyerhaeuser_lvl.toml
  weyerhaeuser_psl.toml
  lp_lvl.toml
  lp_lsl.toml
```

---

## Quick Reference Commands

```bash
# Check ready beads
bd ready

# Show specific bead
bd show Stratify-idi

# Claim a bead
bd update Stratify-idi --status=in_progress

# Close a bead
bd close Stratify-idi --reason="Migration complete, all tests pass"

# Build and verify TOML compilation
cargo build -p calc_core

# Run tests
cargo test -p calc_core

# Sync beads
bd sync
```

---

## Data Quality Checklist

When transcribing new data:

- [ ] Source added to `sources.toml`
- [ ] Metadata complete in TOML file
- [ ] All required fields present
- [ ] Units are in psi (not ksi)
- [ ] E and Emin in psi (not million psi)
- [ ] Spot-check 3+ values against source document
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] Second person review (if available)

---

## Notes

### NDS 2015 Supplement PDF Issue
The PDF at `codes/2015/AWC NDS 2015 Supplement.pdf` is encrypted/protected.
This is common for copyrighted AWC documents. Options:
1. Contact AWC for digital licensing
2. Manually transcribe from print copy
3. Use manufacturer-published catalogs instead

### Manufacturer Data Licensing
Published design values in specifier guides are generally available for use
in structural engineering software. For commercial embedding, verify with
manufacturer. ICC-ES evaluation reports are public documents.
