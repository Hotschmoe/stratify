# Stratify Data Transcription Guide

This guide documents the process for transcribing structural engineering design values from published sources into TOML format for use in Stratify.

## Directory Structure

```
calc_core/data/
  wood/
    sources.toml              # Provenance tracking for all data
    sawn_lumber/
      nds_2018_table4a.toml   # NDS Table 4A - Dimension lumber
      nds_2018_table4b.toml   # NDS Table 4B - Timbers (future)
    engineered/
      lvl_generic.toml        # Generic LVL values
      psl_generic.toml        # Generic PSL values
      glulam_nds.toml         # NDS-S glulam values (future)
      manufacturer/
        weyerhaeuser_lvl.toml # Weyerhaeuser-specific LVL
        lp_lvl.toml           # LP-specific LVL
```

## Transcription Workflow

### 1. Register the Source

Before transcribing any data, add an entry to `sources.toml`:

```toml
[source_id]
name = "Human-readable source name"
publisher = "Publisher name"
document = "Document title"
edition = "Year or version"
license_status = "pending|purchased|public|public_domain"
notes = """
Any relevant notes about the source.
"""
```

### 2. Create the TOML File

Use the appropriate schema template from existing files. Each file must have:

- `[metadata]` section with provenance information
- Data entries following the schema for that file type
- Comments explaining any ambiguities or decisions

### 3. Verify the Data

After transcription:

1. Run `cargo build` to compile the TOML to Rust
2. Run `cargo test` to verify existing tests pass
3. Spot-check at least 3 values against the source document
4. Document the verification in the metadata

### 4. Submit for Review

Create a PR with:
- The new TOML file(s)
- Updated `sources.toml` entry
- Test file if adding new lookup functionality

## TOML Schemas

### Sawn Lumber (NDS Table 4A/4B)

```toml
[metadata]
source = "NDS Supplement"
table = "4A"
edition_year = 2018
transcribed_by = "Your Name"
transcribed_date = "YYYY-MM-DD"
notes = "Optional notes"

[[species]]
name = "Species Name"
code = "XX"  # 2-4 letter code

  [[species.grades]]
  name = "Grade Name"
  code = "GradeCode"
  Fb = 1000.0        # Bending (psi)
  Ft = 600.0         # Tension parallel (psi)
  Fv = 180.0         # Shear (psi)
  Fc_perp = 625.0    # Compression perpendicular (psi)
  Fc = 1350.0        # Compression parallel (psi)
  E = 1600000.0      # Modulus of elasticity (psi)
  Emin = 580000.0    # Minimum E for stability (psi)
  SG = 0.50          # Specific gravity
```

### Engineered Wood (LVL, PSL, LSL)

```toml
[metadata]
source = "Source document"
product_type = "LVL"  # LVL, PSL, LSL, Glulam
manufacturer = "Manufacturer"  # Optional
evaluation_report = "ESR-XXXX"  # Optional ICC-ES report
transcribed_date = "YYYY-MM-DD"

[[products]]
name = "Product Name"
code = "PRODUCT-CODE"
Fb = 2600.0
Ft = 1950.0
Fv = 285.0
Fc_perp = 750.0
Fc = 2510.0
E = 2000000.0
Emin = 1030000.0
SG = 0.50
depth_factor_exponent = 0.111  # For depth adjustment
```

## Data Sources

### NDS (National Design Specification)

- **Publisher**: American Wood Council (AWC)
- **Tables**: 4A (dimension), 4B (timbers), 4C (decking), 4D (MSR), 4E (MEL)
- **Licensing**: Contact info@awc.org for data licensing
- **Notes**: Values are reference design values; apply adjustment factors per NDS

### Manufacturer Data

- **Weyerhaeuser Trus Joist**: ESR-1153 (verify current)
  - Products: Microllam LVL, Parallam PSL, TJI joists

- **LP SolidStart**: ESR-2403 (verify current)
  - Products: LVL, LSL, I-joists

- **Notes**: Published design values in specifier guides are generally available
  for use in structural engineering software. Contact manufacturer for
  commercial embedding rights.

## Using the PDF Extraction Tool

For extracting tables from PDF sources:

```bash
# From the Stratify root directory
cargo run --bin pdf-extractor -- codes/2015/AWC\ NDS\ 2015\ Supplement.pdf

# Output will be in data/extracted/
```

The tool extracts tables and outputs them in a format that can be manually
reviewed and converted to TOML.

## Quality Assurance

### Verification Checklist

- [ ] Source added to sources.toml
- [ ] Metadata complete in TOML file
- [ ] All required fields present
- [ ] Units are in psi (not ksi)
- [ ] E and Emin in psi (not million psi)
- [ ] Spot-check 3+ values against source
- [ ] cargo build succeeds
- [ ] cargo test passes
- [ ] Review by second person

### Common Errors

1. **Wrong units**: NDS tables use psi, some sources use ksi
2. **Missing Emin**: Often listed separately from E
3. **Species codes**: Use consistent codes (DF-L not DFL)
4. **Grade codes**: Match existing conventions (No2 not #2)

## Version Control

- Never commit generated code (target/build/...)
- Always commit TOML source files
- Include sources.toml updates with data changes
- Use descriptive commit messages: "Add Southern Pine grades to NDS Table 4A"
