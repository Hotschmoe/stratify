# Obtaining the Official AISC Shapes Database

This document explains how to obtain, verify, and integrate the official AISC (American Institute of Steel Construction) shapes database into Stratify.

## Why Official Source Matters

The AISC Shapes Database contains dimensional and structural properties for all standard steel shapes used in US structural engineering. Using the official source ensures:

- **Accuracy**: Values match the AISC Steel Construction Manual
- **Liability**: Professional engineers can reference official documentation
- **Updates**: New shapes and corrections are properly tracked

## Step 1: Download from AISC

### Current Version: v16.0 (August 2023)

1. Visit the official AISC download page:
   **https://www.aisc.org/publications/steel-construction-manual-resources/16th-ed-steel-construction-manual/aisc-shapes-database-v16.0/**

2. Download the Excel file: `aisc-shapes-database-v16.0.xlsx`

3. The file is **free** and does not require AISC membership

### What's Included in v16.0

- **W-shapes**: Wide flange beams (W4x13 through W44x335)
- **M-shapes**: Miscellaneous shapes
- **S-shapes**: American Standard beams
- **HP-shapes**: H-piles
- **C-shapes**: American Standard channels
- **MC-shapes**: Miscellaneous channels
- **L-shapes**: Single angles
- **WT, MT, ST**: Structural tees
- **2L-shapes**: Double angles
- **HSS**: Hollow structural sections (rectangular, square, round)
- **PIPE**: Standard and extra-strong pipe

**v16.0 includes 222 new shapes** not in v15.0.

### Historic Shapes Database (Optional)

For renovation projects requiring historic steel sections (1873-2016):
**https://www.aisc.org/publications/steel-construction-manual-resources/16th-ed-steel-construction-manual/**

Download: `aisc-shapes-database-v16.0h.xlsx`

## Step 2: Verify Version and Integrity

### Check You Have the Latest Version

1. Open the Excel file
2. Go to the **"Readme"** sheet
3. Verify the version number matches: **v16.0 (August 2023)**

### Version History

| Version | Release Date | Manual Edition | Notes |
|---------|--------------|----------------|-------|
| v16.0   | August 2023  | 16th Edition   | Current - 222 new shapes |
| v15.0   | November 2017| 15th Edition   | Previous |
| v14.1   | October 2013 | 14th Edition   | Legacy |

### Check for Updates

AISC releases companion resources at:
**https://www.aisc.org/publications/steel-construction-manual-resources/**

Check this page periodically or when a new Manual edition is released.

## Step 3: Convert to CSV Format

Stratify reads steel shapes from CSV files for portability and version control.

### Using Excel

1. Open `aisc-shapes-database-v16.0.xlsx`
2. Select the **"Database"** sheet (contains all shapes)
3. File > Save As > CSV (Comma delimited) (*.csv)
4. Save as `aisc-shapes-v16.csv` in this folder (`assets/steel/`)

### Using LibreOffice Calc

1. Open the .xlsx file
2. Select the Database sheet
3. File > Save As > Text CSV (.csv)
4. Options: UTF-8 encoding, comma separator, quote text cells
5. Save as `aisc-shapes-v16.csv`

### Column Format

The CSV should have these columns (in order):

```
Type,EDI_Std_Nomenclature,AISC_Manual_Label,T_F,W,A,d,ddet,Ht,h,OD,bf,bfdet,B,b,ID,tw,twdet,twdet/2,tf,tfdet,t,tnom,tdes,kdes,kdet,k1,x,y,eo,xp,yp,bf/2tf,b/t,b/tdes,h/tw,h/tdes,D/t,Ix,Zx,Sx,rx,Iy,Zy,Sy,ry,Iz,rz,Sz,J,Cw,C,Wno,Sw1,Sw2,Sw3,Qf,Qw,ro,H,tan(a),Iw,zA,zB,zC,wA,wB,wC,SzA,SzB,SzC,rts,ho,PA,PA2,PB,PC,PD,T,WGi,WGo
```

Key properties:
- **Type**: Shape type (W, M, S, HP, C, MC, L, WT, HSS, PIPE)
- **AISC_Manual_Label**: Human-readable name (e.g., "W14X90")
- **W**: Weight per linear foot (lb/ft)
- **A**: Cross-sectional area (in²)
- **d**: Depth (in)
- **bf**: Flange width (in)
- **tf**: Flange thickness (in)
- **tw**: Web thickness (in)
- **Ix, Iy**: Moments of inertia (in⁴)
- **Sx, Sy**: Elastic section moduli (in³)
- **Zx, Zy**: Plastic section moduli (in³)
- **rx, ry**: Radii of gyration (in)
- **J**: Torsional constant (in⁴)
- **Cw**: Warping constant (in⁶)

## Step 4: Place Files in Stratify

Copy your CSV file to:
```
stratify/
└── assets/
    └── steel/
        ├── OBTAINING_AISC_DATA.md  (this file)
        └── aisc-shapes-v16.csv     (your converted file)
```

## Step 5: Verify Integration

Run the test suite to verify the shapes load correctly:

```bash
cargo test steel
```

Or test a specific shape lookup:

```bash
cargo run --bin calc_cli -- lookup-shape W14X90
```

## Licensing Notes

- The AISC Shapes Database is provided **free** by AISC
- Data may be used in software per AISC's terms of use
- Always cite: "AISC Shapes Database v16.0, American Institute of Steel Construction"
- The database carries AISC's standard disclaimer about professional verification

## Troubleshooting

### "Shape not found" errors

1. Verify the CSV file exists in `assets/steel/`
2. Check the shape name format (e.g., "W14X90" not "W14x90")
3. Ensure the CSV was exported correctly (no missing columns)

### Numeric parsing errors

1. Check for non-numeric values in numeric columns
2. Replace any dashes (`—`) with empty cells or zero
3. Ensure decimal points, not commas, for numbers

### Missing properties

Some properties are only applicable to certain shape types:
- `bf`, `tf`: Only for shapes with flanges (W, C, etc.)
- `OD`, `ID`: Only for round shapes (HSS round, PIPE)
- `B`, `Ht`: Only for rectangular HSS

## References

- [AISC Shapes Database v16.0](https://www.aisc.org/publications/steel-construction-manual-resources/16th-ed-steel-construction-manual/aisc-shapes-database-v16.0/)
- [AISC Steel Construction Manual, 16th Edition](https://www.aisc.org/publications/steel-construction-manual-resources/)
- [AISC Shape Availability Tool](https://www.aisc.org/steelavailability/)
- [EDI Naming Convention for Steel Shapes](https://www.aisc.org/publications/steel-construction-manual-resources/)
