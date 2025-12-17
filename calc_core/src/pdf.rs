//! # PDF Generation Module
//!
//! Generates professional PDF reports from structural calculations using Typst.
//!
//! ## Architecture
//!
//! - Typst templates are embedded as string constants
//! - Data is injected via string formatting before compilation
//! - Output is raw PDF bytes (`Vec<u8>`)
//!
//! ## Example
//!
//! ```rust,no_run
//! use calc_core::pdf::render_beam_pdf;
//! use calc_core::calculations::beam::{BeamInput, calculate};
//! use calc_core::materials::{WoodSpecies, WoodGrade, WoodMaterial};
//!
//! let input = BeamInput {
//!     label: "B-1".to_string(),
//!     span_ft: 12.0,
//!     uniform_load_plf: 150.0,
//!     material: WoodMaterial::new(WoodSpecies::DouglasFirLarch, WoodGrade::No2),
//!     width_in: 1.5,
//!     depth_in: 9.25,
//! };
//!
//! let result = calculate(&input).unwrap();
//! let pdf_bytes = render_beam_pdf(&input, &result, "John Engineer", "25-001").unwrap();
//! std::fs::write("beam_report.pdf", pdf_bytes).unwrap();
//! ```

use chrono::Utc;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_pdf::PdfOptions;

use crate::calculations::beam::{BeamInput, BeamResult};
use crate::errors::{CalcError, CalcResult};

// ============================================================================
// Typst World Implementation
// ============================================================================

/// A minimal Typst world for compiling documents without external files.
struct PdfWorld {
    /// The main source document
    main: Source,
    /// Font book
    book: LazyHash<FontBook>,
    /// Available fonts
    fonts: Vec<Font>,
    /// Library (standard functions)
    library: LazyHash<Library>,
}

impl PdfWorld {
    fn new(source: String) -> Self {
        let fonts = Self::load_fonts();
        let book = FontBook::from_fonts(&fonts);

        PdfWorld {
            main: Source::detached(source),
            book: LazyHash::new(book),
            fonts,
            library: LazyHash::new(Library::default()),
        }
    }

    fn load_fonts() -> Vec<Font> {
        let mut fonts = Vec::new();

        // Load BerkeleyMono fonts (embedded at compile time)
        const BERKELEY_MONO_REGULAR: &[u8] =
            include_bytes!("../../assets/fonts/BerkleyMono/BerkeleyMono-Regular.otf");
        const BERKELEY_MONO_BOLD: &[u8] =
            include_bytes!("../../assets/fonts/BerkleyMono/BerkeleyMono-Bold.otf");
        const BERKELEY_MONO_OBLIQUE: &[u8] =
            include_bytes!("../../assets/fonts/BerkleyMono/BerkeleyMono-Oblique.otf");
        const BERKELEY_MONO_BOLD_OBLIQUE: &[u8] =
            include_bytes!("../../assets/fonts/BerkleyMono/BerkeleyMono-Bold-Oblique.otf");

        // Load our custom fonts first (higher priority)
        for font_bytes in [
            BERKELEY_MONO_REGULAR,
            BERKELEY_MONO_BOLD,
            BERKELEY_MONO_OBLIQUE,
            BERKELEY_MONO_BOLD_OBLIQUE,
        ] {
            let buffer = Bytes::new(font_bytes.to_vec());
            for font in Font::iter(buffer) {
                fonts.push(font);
            }
        }

        // Load bundled fonts from typst-assets (fallback for math symbols, etc.)
        for font_bytes in typst_assets::fonts() {
            let buffer = Bytes::new(font_bytes.to_vec());
            for font in Font::iter(buffer) {
                fonts.push(font);
            }
        }

        fonts
    }
}

impl World for PdfWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        let now = Utc::now();
        Datetime::from_ymd(
            now.format("%Y").to_string().parse().ok()?,
            now.format("%m").to_string().parse().ok()?,
            now.format("%d").to_string().parse().ok()?,
        )
    }
}

// ============================================================================
// PDF Templates
// ============================================================================

/// Typst template for beam calculation report
const BEAM_TEMPLATE: &str = r##"
#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  header: align(right)[
    #text(size: 9pt, fill: gray)[Stratify Structural Calculations]
  ],
  footer: context [
    #line(length: 100%, stroke: 0.5pt + gray)
    #v(4pt)
    #grid(
      columns: (1fr, 1fr, 1fr),
      align(left)[#text(size: 9pt)[Job: {{JOB_ID}}]],
      align(center)[#text(size: 9pt)[Page #counter(page).display()]],
      align(right)[#text(size: 9pt)[{{DATE}}]],
    )
  ]
)

#set text(font: "Berkeley Mono", size: 11pt)

// Title Block
#align(center)[
  #block(width: 100%, fill: rgb("#f0f0f0"), inset: 12pt, radius: 4pt)[
    #text(size: 18pt, weight: "bold")[Simply-Supported Beam Analysis]
    #v(4pt)
    #text(size: 14pt)[{{BEAM_LABEL}}]
  ]
]

#v(12pt)

#grid(
  columns: (1fr, 1fr),
  gutter: 20pt,
  [
    *Project Information*
    #v(4pt)
    #table(
      columns: (auto, 1fr),
      stroke: none,
      row-gutter: 4pt,
      [Engineer:], [{{ENGINEER}}],
      [Job ID:], [{{JOB_ID}}],
      [Date:], [{{DATE}}],
    )
  ],
  [
    *Code Reference*
    #v(4pt)
    NDS 2018 (National Design Specification for Wood Construction)
  ]
)

#v(16pt)
#line(length: 100%, stroke: 0.5pt)
#v(8pt)

== Input Parameters

#table(
  columns: (1fr, auto, auto),
  inset: 8pt,
  stroke: 0.5pt,
  align: (left, right, left),
  table.header([*Parameter*], [*Value*], [*Unit*]),
  [Clear Span], [{{SPAN_FT}}], [ft],
  [Uniform Load], [{{LOAD_PLF}}], [plf],
  [Beam Width], [{{WIDTH_IN}}], [in],
  [Beam Depth], [{{DEPTH_IN}}], [in],
  [Material], [{{MATERIAL}}], [],
)

#v(12pt)

== Section Properties

#table(
  columns: (1fr, auto, auto),
  inset: 8pt,
  stroke: 0.5pt,
  align: (left, right, left),
  table.header([*Property*], [*Value*], [*Unit*]),
  [Section Modulus (S)], [{{SECTION_MODULUS}}], [in#super[3]],
  [Moment of Inertia (I)], [{{MOMENT_INERTIA}}], [in#super[4]],
)

#v(12pt)

== Material Properties (Reference Values)

#table(
  columns: (1fr, auto, auto),
  inset: 8pt,
  stroke: 0.5pt,
  align: (left, right, left),
  table.header([*Property*], [*Value*], [*Unit*]),
  [Bending Stress (F#sub[b])], [{{FB_REF}}], [psi],
  [Shear Stress (F#sub[v])], [{{FV_REF}}], [psi],
  [Modulus of Elasticity (E)], [{{E_REF}}], [psi],
)

#v(16pt)
#line(length: 100%, stroke: 0.5pt)
#v(8pt)

== Analysis Results

=== Applied Forces (Demand)

For a simply-supported beam with uniform load:

$ M_"max" = (w L^2) / 8 = {{MOMENT_FTLB}} "ft-lb" $

$ V_"max" = (w L) / 2 = {{SHEAR_LB}} "lb" $

$ delta_"max" = (5 w L^4) / (384 E I) = {{DEFLECTION_IN}} "in" $

#v(12pt)

=== Stress Checks

#table(
  columns: (1fr, auto, auto, auto, auto),
  inset: 8pt,
  stroke: 0.5pt,
  align: (left, right, right, right, center),
  table.header([*Check*], [*Actual*], [*Allowable*], [*Unity*], [*Status*]),
  [Bending], [{{FB_ACTUAL}} psi], [{{FB_ALLOW}} psi], [{{BENDING_UNITY}}], [{{BENDING_STATUS}}],
  [Shear], [{{FV_ACTUAL}} psi], [{{FV_ALLOW}} psi], [{{SHEAR_UNITY}}], [{{SHEAR_STATUS}}],
  [Deflection], [L/{{DEFL_RATIO}}], [L/{{DEFL_LIMIT}}], [{{DEFL_UNITY}}], [{{DEFL_STATUS}}],
)

#v(16pt)

#let pass_status = "{{OVERALL_PASS}}"
#align(center)[
  #block(
    width: auto,
    fill: if pass_status == "PASS" { rgb("#d4edda") } else { rgb("#f8d7da") },
    inset: 16pt,
    radius: 4pt
  )[
    #text(size: 16pt, weight: "bold")[
      #if pass_status == "PASS" [
        DESIGN ADEQUATE
      ] else [
        DESIGN INADEQUATE
      ]
    ]
    #v(4pt)
    #text(size: 12pt)[Governing condition: {{GOVERNING}}]
  ]
]

#v(24pt)
#line(length: 100%, stroke: 0.5pt)
#v(8pt)

#text(size: 9pt, fill: gray)[
  Generated by Stratify Structural Engineering Suite \
  Calculations should be verified by a licensed professional engineer.
]
"##;

// ============================================================================
// PDF Rendering Functions
// ============================================================================

/// Render a beam calculation to PDF.
///
/// # Arguments
///
/// * `input` - The beam input parameters
/// * `result` - The calculation results
/// * `engineer` - Engineer name for the report
/// * `job_id` - Job/project ID
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - PDF file as bytes
/// * `Err(CalcError)` - If rendering fails
///
/// # Example
///
/// ```rust,no_run
/// use calc_core::pdf::render_beam_pdf;
/// use calc_core::calculations::beam::{BeamInput, calculate};
/// use calc_core::materials::{WoodSpecies, WoodGrade, WoodMaterial};
///
/// let input = BeamInput {
///     label: "B-1".to_string(),
///     span_ft: 12.0,
///     uniform_load_plf: 150.0,
///     material: WoodMaterial::new(WoodSpecies::DouglasFirLarch, WoodGrade::No2),
///     width_in: 1.5,
///     depth_in: 9.25,
/// };
///
/// let result = calculate(&input).unwrap();
/// let pdf = render_beam_pdf(&input, &result, "John Engineer", "25-001").unwrap();
/// ```
pub fn render_beam_pdf(
    input: &BeamInput,
    result: &BeamResult,
    engineer: &str,
    job_id: &str,
) -> CalcResult<Vec<u8>> {
    // Format the template with calculation data
    let source = BEAM_TEMPLATE
        .replace("{{BEAM_LABEL}}", &input.label)
        .replace("{{ENGINEER}}", engineer)
        .replace("{{JOB_ID}}", job_id)
        .replace("{{DATE}}", &Utc::now().format("%Y-%m-%d").to_string())
        .replace("{{SPAN_FT}}", &format!("{:.1}", input.span_ft))
        .replace("{{LOAD_PLF}}", &format!("{:.0}", input.uniform_load_plf))
        .replace("{{WIDTH_IN}}", &format!("{:.2}", input.width_in))
        .replace("{{DEPTH_IN}}", &format!("{:.2}", input.depth_in))
        .replace(
            "{{MATERIAL}}",
            &format!(
                "{} {}",
                input.material.species.display_name(),
                input.material.grade.display_name()
            ),
        )
        .replace("{{SECTION_MODULUS}}", &format!("{:.2}", result.section_modulus_in3))
        .replace("{{MOMENT_INERTIA}}", &format!("{:.2}", result.moment_of_inertia_in4))
        .replace("{{FB_REF}}", &format!("{:.0}", result.fb_reference_psi))
        .replace("{{FV_REF}}", &format!("{:.0}", result.fv_reference_psi))
        .replace("{{E_REF}}", &format!("{:.0}", result.e_psi))
        .replace("{{MOMENT_FTLB}}", &format!("{:.0}", result.max_moment_ftlb))
        .replace("{{SHEAR_LB}}", &format!("{:.0}", result.max_shear_lb))
        .replace("{{DEFLECTION_IN}}", &format!("{:.3}", result.max_deflection_in))
        .replace("{{FB_ACTUAL}}", &format!("{:.0}", result.actual_fb_psi))
        .replace("{{FB_ALLOW}}", &format!("{:.0}", result.allowable_fb_psi))
        .replace("{{BENDING_UNITY}}", &format!("{:.2}", result.bending_unity))
        .replace(
            "{{BENDING_STATUS}}",
            if result.bending_unity <= 1.0 { "OK" } else { "FAIL" },
        )
        .replace("{{FV_ACTUAL}}", &format!("{:.0}", result.actual_fv_psi))
        .replace("{{FV_ALLOW}}", &format!("{:.0}", result.allowable_fv_psi))
        .replace("{{SHEAR_UNITY}}", &format!("{:.2}", result.shear_unity))
        .replace(
            "{{SHEAR_STATUS}}",
            if result.shear_unity <= 1.0 { "OK" } else { "FAIL" },
        )
        .replace("{{DEFL_RATIO}}", &format!("{:.0}", result.deflection_ratio))
        .replace("{{DEFL_LIMIT}}", &format!("{:.0}", result.deflection_limit_ratio))
        .replace("{{DEFL_UNITY}}", &format!("{:.2}", result.deflection_unity))
        .replace(
            "{{DEFL_STATUS}}",
            if result.deflection_unity <= 1.0 { "OK" } else { "FAIL" },
        )
        .replace(
            "{{OVERALL_PASS}}",
            if result.passes() { "PASS" } else { "FAIL" },
        )
        .replace("{{GOVERNING}}", result.governing_condition());

    // Compile the Typst document
    let world = PdfWorld::new(source);

    let warned = typst::compile(&world);

    let document = warned.output.map_err(|errors| {
        let error_msgs: Vec<String> = errors
            .iter()
            .map(|e| e.message.to_string())
            .collect();
        CalcError::Internal {
            message: format!("Typst compilation failed: {}", error_msgs.join("; ")),
        }
    })?;

    // Render to PDF
    let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default()).map_err(|errors| {
        let error_msgs: Vec<String> = errors.iter().map(|e| e.message.to_string()).collect();
        CalcError::Internal {
            message: format!("PDF rendering failed: {}", error_msgs.join("; ")),
        }
    })?;

    Ok(pdf_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculations::beam::calculate;
    use crate::materials::{WoodGrade, WoodMaterial, WoodSpecies};

    #[test]
    fn test_pdf_generation() {
        let input = BeamInput {
            label: "B-1 Test Beam".to_string(),
            span_ft: 12.0,
            uniform_load_plf: 100.0,
            material: WoodMaterial::new(WoodSpecies::DouglasFirLarch, WoodGrade::No2),
            width_in: 1.5,
            depth_in: 9.25,
        };

        let result = calculate(&input).unwrap();
        let pdf = render_beam_pdf(&input, &result, "Test Engineer", "TEST-001");

        // Should succeed
        assert!(pdf.is_ok(), "PDF generation failed: {:?}", pdf.err());

        let pdf_bytes = pdf.unwrap();
        // PDF should start with %PDF
        assert!(pdf_bytes.starts_with(b"%PDF"), "Output is not a valid PDF");
        // Should be a reasonable size (at least 1KB)
        assert!(pdf_bytes.len() > 1000, "PDF seems too small");
    }
}
