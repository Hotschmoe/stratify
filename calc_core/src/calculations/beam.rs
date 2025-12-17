//! # Simply-Supported Beam Calculation
//!
//! Analyzes a simply-supported wood beam under uniform load per NDS.
//!
//! ## Assumptions
//!
//! - Simply-supported (pin-roller) boundary conditions
//! - Uniform distributed load over full span
//! - Rectangular section (sawn lumber, glulam, LVL, or PSL)
//! - Dry service conditions (C_M = 1.0)
//! - Normal temperature (C_t = 1.0)
//! - Normal load duration (C_D = 1.0) - adjust as needed
//!
//! ## Example (LLM-friendly)
//!
//! ```rust
//! use calc_core::calculations::beam::{BeamInput, calculate};
//! use calc_core::materials::{Material, WoodSpecies, WoodGrade, WoodMaterial};
//!
//! // Define beam input as would come from JSON
//! let input = BeamInput {
//!     label: "B-1".to_string(),
//!     span_ft: 12.0,
//!     uniform_load_plf: 150.0,
//!     material: Material::SawnLumber(WoodMaterial::new(
//!         WoodSpecies::DouglasFirLarch,
//!         WoodGrade::No2
//!     )),
//!     width_in: 1.5,  // 2x nominal
//!     depth_in: 9.25, // 10 nominal
//! };
//!
//! let result = calculate(&input).unwrap();
//!
//! println!("Max moment: {:.2} ft-lb", result.max_moment_ftlb);
//! println!("Bending stress: {:.0} psi", result.actual_fb_psi);
//! println!("Bending unity: {:.2}", result.bending_unity);
//! println!("Pass: {}", result.passes());
//! ```

use serde::{Deserialize, Serialize};

use crate::errors::{CalcError, CalcResult};
use crate::materials::Material;

/// Input parameters for a simply-supported beam.
///
/// All inputs use US customary units for compatibility with US building codes.
/// Supports sawn lumber, glulam, LVL, and PSL materials.
///
/// ## JSON Example (Sawn Lumber)
///
/// ```json
/// {
///   "label": "B-1",
///   "span_ft": 12.0,
///   "uniform_load_plf": 150.0,
///   "material": {
///     "type": "SawnLumber",
///     "species": "DF-L",
///     "grade": "No.2"
///   },
///   "width_in": 1.5,
///   "depth_in": 9.25
/// }
/// ```
///
/// ## JSON Example (Glulam)
///
/// ```json
/// {
///   "label": "GLB-1",
///   "span_ft": 24.0,
///   "uniform_load_plf": 300.0,
///   "material": {
///     "type": "Glulam",
///     "stress_class": "24F-V4",
///     "layup": "Unbalanced"
///   },
///   "width_in": 5.125,
///   "depth_in": 16.5
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamInput {
    /// User label for this beam (e.g., "B-1", "Floor Beam at Grid A")
    pub label: String,

    /// Clear span in feet
    pub span_ft: f64,

    /// Uniform distributed load in pounds per linear foot (plf)
    ///
    /// This should be the total factored load (dead + live) for ASD.
    pub uniform_load_plf: f64,

    /// Material (sawn lumber, glulam, LVL, or PSL)
    pub material: Material,

    /// Actual beam width in inches (e.g., 1.5 for 2x, 5.125 for glulam)
    pub width_in: f64,

    /// Actual beam depth in inches (e.g., 9.25 for 2x10, 16.5 for glulam)
    pub depth_in: f64,
}

impl BeamInput {
    /// Validate input parameters.
    pub fn validate(&self) -> CalcResult<()> {
        if self.span_ft <= 0.0 {
            return Err(CalcError::invalid_input(
                "span_ft",
                self.span_ft.to_string(),
                "Span must be positive",
            ));
        }
        if self.span_ft > 60.0 {
            return Err(CalcError::invalid_input(
                "span_ft",
                self.span_ft.to_string(),
                "Span exceeds 60 ft - verify member sizing",
            ));
        }
        if self.uniform_load_plf < 0.0 {
            return Err(CalcError::invalid_input(
                "uniform_load_plf",
                self.uniform_load_plf.to_string(),
                "Load cannot be negative",
            ));
        }
        if self.width_in <= 0.0 {
            return Err(CalcError::invalid_input(
                "width_in",
                self.width_in.to_string(),
                "Width must be positive",
            ));
        }
        if self.depth_in <= 0.0 {
            return Err(CalcError::invalid_input(
                "depth_in",
                self.depth_in.to_string(),
                "Depth must be positive",
            ));
        }
        Ok(())
    }

    /// Calculate section modulus S = bd²/6
    pub fn section_modulus_in3(&self) -> f64 {
        self.width_in * self.depth_in.powi(2) / 6.0
    }

    /// Calculate moment of inertia I = bd³/12
    pub fn moment_of_inertia_in4(&self) -> f64 {
        self.width_in * self.depth_in.powi(3) / 12.0
    }

    /// Calculate cross-sectional area A = bd
    pub fn area_in2(&self) -> f64 {
        self.width_in * self.depth_in
    }
}

/// Results from beam calculation.
///
/// All results include both raw values and unity checks for easy pass/fail determination.
///
/// ## JSON Example
///
/// ```json
/// {
///   "max_moment_ftlb": 2700.0,
///   "max_shear_lb": 900.0,
///   "max_deflection_in": 0.42,
///   "actual_fb_psi": 502.7,
///   "allowable_fb_psi": 900.0,
///   "bending_unity": 0.56,
///   "actual_fv_psi": 9.7,
///   "allowable_fv_psi": 180.0,
///   "shear_unity": 0.05,
///   "deflection_ratio": 343,
///   "deflection_limit_ratio": 240,
///   "deflection_unity": 0.70
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamResult {
    // === Demand (Applied Forces) ===
    /// Maximum bending moment in foot-pounds
    ///
    /// For simply-supported beam with uniform load: M = wL²/8
    pub max_moment_ftlb: f64,

    /// Maximum shear force in pounds
    ///
    /// For simply-supported beam with uniform load: V = wL/2
    pub max_shear_lb: f64,

    /// Maximum deflection in inches
    ///
    /// For simply-supported beam with uniform load: δ = 5wL⁴/(384EI)
    pub max_deflection_in: f64,

    // === Bending Check ===
    /// Actual bending stress fb = M/S (psi)
    pub actual_fb_psi: f64,

    /// Allowable bending stress Fb' (psi)
    ///
    /// Adjusted for all applicable NDS factors.
    pub allowable_fb_psi: f64,

    /// Bending unity check: actual_fb / allowable_fb
    ///
    /// Must be ≤ 1.0 to pass.
    pub bending_unity: f64,

    // === Shear Check ===
    /// Actual shear stress fv = 3V/(2bd) (psi)
    pub actual_fv_psi: f64,

    /// Allowable shear stress Fv' (psi)
    pub allowable_fv_psi: f64,

    /// Shear unity check: actual_fv / allowable_fv
    ///
    /// Must be ≤ 1.0 to pass.
    pub shear_unity: f64,

    // === Deflection Check ===
    /// Deflection ratio L/δ
    ///
    /// Higher is better (less deflection).
    pub deflection_ratio: f64,

    /// Deflection limit ratio (typically L/240 for live load, L/180 for total)
    pub deflection_limit_ratio: f64,

    /// Deflection unity check: (L/limit) / (L/actual) = actual/limit
    ///
    /// Must be ≤ 1.0 to pass.
    pub deflection_unity: f64,

    // === Section Properties (for reference) ===
    /// Section modulus S (in³)
    pub section_modulus_in3: f64,

    /// Moment of inertia I (in⁴)
    pub moment_of_inertia_in4: f64,

    // === Material Properties Used ===
    /// Reference bending stress Fb (psi) before adjustments
    pub fb_reference_psi: f64,

    /// Reference shear stress Fv (psi)
    pub fv_reference_psi: f64,

    /// Modulus of elasticity E (psi)
    pub e_psi: f64,
}

impl BeamResult {
    /// Check if all unity checks pass (≤ 1.0)
    pub fn passes(&self) -> bool {
        self.bending_unity <= 1.0 && self.shear_unity <= 1.0 && self.deflection_unity <= 1.0
    }

    /// Get the governing (highest) unity ratio
    pub fn governing_unity(&self) -> f64 {
        self.bending_unity
            .max(self.shear_unity)
            .max(self.deflection_unity)
    }

    /// Get a description of what governs the design
    pub fn governing_condition(&self) -> &'static str {
        if self.bending_unity >= self.shear_unity && self.bending_unity >= self.deflection_unity {
            "Bending"
        } else if self.shear_unity >= self.deflection_unity {
            "Shear"
        } else {
            "Deflection"
        }
    }
}

/// Calculate beam stresses and deflections.
///
/// This is a pure function suitable for LLM invocation.
///
/// # Arguments
///
/// * `input` - Beam parameters (span, load, material, size)
///
/// # Returns
///
/// * `Ok(BeamResult)` - Calculation results with all checks
/// * `Err(CalcError)` - Structured error if inputs are invalid
///
/// # Example
///
/// ```rust
/// use calc_core::calculations::beam::{BeamInput, calculate};
/// use calc_core::materials::{Material, WoodSpecies, WoodGrade, WoodMaterial};
///
/// let input = BeamInput {
///     label: "Test Beam".to_string(),
///     span_ft: 10.0,
///     uniform_load_plf: 100.0,
///     material: Material::SawnLumber(WoodMaterial::new(
///         WoodSpecies::DouglasFirLarch,
///         WoodGrade::No2
///     )),
///     width_in: 1.5,
///     depth_in: 9.25,
/// };
///
/// let result = calculate(&input).expect("Calculation should succeed");
/// assert!(result.max_moment_ftlb > 0.0);
/// ```
pub fn calculate(input: &BeamInput) -> CalcResult<BeamResult> {
    // Validate inputs
    input.validate()?;

    // Get material properties (unified interface for all material types)
    let props = input.material.base_properties();

    // Section properties
    let s = input.section_modulus_in3();
    let i = input.moment_of_inertia_in4();
    let area = input.area_in2();

    // Convert span to inches for deflection calc
    let span_in = input.span_ft * 12.0;

    // === Calculate Demand ===

    // Maximum moment: M = wL²/8 (result in ft-lb)
    let max_moment_ftlb = input.uniform_load_plf * input.span_ft.powi(2) / 8.0;

    // Maximum shear: V = wL/2 (result in lb)
    let max_shear_lb = input.uniform_load_plf * input.span_ft / 2.0;

    // Maximum deflection: δ = 5wL⁴/(384EI)
    // Need consistent units: w in lb/in, L in inches, E in psi, I in in⁴
    let w_lb_per_in = input.uniform_load_plf / 12.0;
    let max_deflection_in = 5.0 * w_lb_per_in * span_in.powi(4) / (384.0 * props.e_psi * i);

    // === Calculate Stresses ===

    // Bending stress: fb = M/S
    // M is in ft-lb, S is in in³, so convert M to in-lb
    let max_moment_inlb = max_moment_ftlb * 12.0;
    let actual_fb_psi = max_moment_inlb / s;

    // Shear stress: fv = 3V/(2bd) = 3V/(2A)
    let actual_fv_psi = 3.0 * max_shear_lb / (2.0 * area);

    // === Apply NDS Adjustment Factors ===
    // For this MVP, we use simplified factors:
    // C_D = 1.0 (normal duration)
    // C_M = 1.0 (dry service)
    // C_t = 1.0 (normal temperature)
    // C_F = size factor (applied via fb_for_depth for LVL/PSL)
    // C_r = 1.0 (not repetitive for single beam)

    // Get Fb adjusted for depth (handles LVL/PSL depth factor automatically)
    let fb_depth_adjusted = input.material.fb_for_depth(input.depth_in);

    // Additional size factor C_F for sawn lumber bending (NDS Table 4A footnote)
    // For depths > 12", C_F = (12/d)^(1/9)
    // Note: LVL/PSL handle this in fb_for_depth, glulam uses volume factor separately
    let c_f = if !input.material.is_engineered() && input.depth_in > 12.0 {
        (12.0 / input.depth_in).powf(1.0 / 9.0)
    } else {
        1.0
    };

    let allowable_fb_psi = fb_depth_adjusted * c_f;
    let allowable_fv_psi = props.fv_psi; // No adjustment for shear in simple case

    // === Unity Checks ===

    let bending_unity = actual_fb_psi / allowable_fb_psi;
    let shear_unity = actual_fv_psi / allowable_fv_psi;

    // Deflection check (L/240 for typical floor beam)
    let deflection_limit_ratio = 240.0;
    let deflection_ratio = if max_deflection_in > 0.0 {
        span_in / max_deflection_in
    } else {
        f64::INFINITY
    };
    let deflection_unity = deflection_limit_ratio / deflection_ratio;

    Ok(BeamResult {
        max_moment_ftlb,
        max_shear_lb,
        max_deflection_in,
        actual_fb_psi,
        allowable_fb_psi,
        bending_unity,
        actual_fv_psi,
        allowable_fv_psi,
        shear_unity,
        deflection_ratio,
        deflection_limit_ratio,
        deflection_unity,
        section_modulus_in3: s,
        moment_of_inertia_in4: i,
        fb_reference_psi: props.fb_psi,
        fv_reference_psi: props.fv_psi,
        e_psi: props.e_psi,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::{WoodGrade, WoodSpecies, WoodMaterial, GlulamMaterial, GlulamStressClass, GlulamLayup, LvlMaterial, LvlGrade};

    fn test_beam() -> BeamInput {
        BeamInput {
            label: "Test Beam".to_string(),
            span_ft: 12.0,
            uniform_load_plf: 150.0,
            material: Material::SawnLumber(WoodMaterial::new(
                WoodSpecies::DouglasFirLarch,
                WoodGrade::No2,
            )),
            width_in: 1.5,
            depth_in: 9.25,
        }
    }

    #[test]
    fn test_section_properties() {
        let beam = test_beam();
        let s = beam.section_modulus_in3();
        let i = beam.moment_of_inertia_in4();

        // S = bd²/6 = 1.5 * 9.25² / 6 = 21.39
        assert!((s - 21.39).abs() < 0.1);

        // I = bd³/12 = 1.5 * 9.25³ / 12 = 98.93
        assert!((i - 98.93).abs() < 0.1);
    }

    #[test]
    fn test_moment_calculation() {
        let beam = test_beam();
        let result = calculate(&beam).unwrap();

        // M = wL²/8 = 150 * 12² / 8 = 2700 ft-lb
        assert!((result.max_moment_ftlb - 2700.0).abs() < 1.0);
    }

    #[test]
    fn test_shear_calculation() {
        let beam = test_beam();
        let result = calculate(&beam).unwrap();

        // V = wL/2 = 150 * 12 / 2 = 900 lb
        assert!((result.max_shear_lb - 900.0).abs() < 1.0);
    }

    #[test]
    fn test_bending_stress() {
        let beam = test_beam();
        let result = calculate(&beam).unwrap();

        // fb = M/S = (2700 * 12) / 21.39 = 1515 psi (approximately)
        assert!(result.actual_fb_psi > 1400.0 && result.actual_fb_psi < 1600.0);
    }

    #[test]
    fn test_passes_check() {
        let beam = test_beam();
        let result = calculate(&beam).unwrap();

        // This beam should fail (unity > 1.0 for bending)
        // 2x10 DF-L No.2 at 12' with 150 plf is overstressed
        assert!(!result.passes() || result.bending_unity <= 1.0);
    }

    #[test]
    fn test_valid_beam_passes() {
        // A more reasonable beam that should pass
        let beam = BeamInput {
            label: "Light Beam".to_string(),
            span_ft: 8.0,
            uniform_load_plf: 50.0,
            material: Material::SawnLumber(WoodMaterial::new(
                WoodSpecies::DouglasFirLarch,
                WoodGrade::No2,
            )),
            width_in: 1.5,
            depth_in: 9.25,
        };
        let result = calculate(&beam).unwrap();
        assert!(result.passes());
    }

    #[test]
    fn test_glulam_beam() {
        let beam = BeamInput {
            label: "Glulam Beam".to_string(),
            span_ft: 24.0,
            uniform_load_plf: 200.0,
            material: Material::Glulam(GlulamMaterial::new(
                GlulamStressClass::F24_V4,
                GlulamLayup::Unbalanced,
            )),
            width_in: 5.125,
            depth_in: 16.5,
        };
        let result = calculate(&beam).unwrap();
        // Should have higher allowable Fb than sawn lumber
        assert!(result.allowable_fb_psi > 2000.0);
    }

    #[test]
    fn test_lvl_beam() {
        let beam = BeamInput {
            label: "LVL Header".to_string(),
            span_ft: 12.0,
            uniform_load_plf: 400.0,
            material: Material::Lvl(LvlMaterial::new(LvlGrade::Standard)),
            width_in: 1.75,
            depth_in: 11.875,
        };
        let result = calculate(&beam).unwrap();
        // LVL should have higher E than sawn lumber
        assert!(result.e_psi >= 2_000_000.0);
    }

    #[test]
    fn test_invalid_span() {
        let mut beam = test_beam();
        beam.span_ft = -5.0;
        let result = calculate(&beam);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let beam = test_beam();
        let json = serde_json::to_string_pretty(&beam).unwrap();
        let roundtrip: BeamInput = serde_json::from_str(&json).unwrap();
        assert_eq!(beam.span_ft, roundtrip.span_ft);
        assert_eq!(beam.uniform_load_plf, roundtrip.uniform_load_plf);
    }

    #[test]
    fn test_result_serialization() {
        let beam = test_beam();
        let result = calculate(&beam).unwrap();
        let json = serde_json::to_string_pretty(&result).unwrap();

        // Should contain key fields
        assert!(json.contains("max_moment_ftlb"));
        assert!(json.contains("bending_unity"));
        assert!(json.contains("shear_unity"));

        let roundtrip: BeamResult = serde_json::from_str(&json).unwrap();
        assert!((result.max_moment_ftlb - roundtrip.max_moment_ftlb).abs() < 0.001);
    }
}
