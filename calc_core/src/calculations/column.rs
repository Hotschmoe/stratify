//! # Column Calculation
//!
//! Analyzes axial compression members (columns) per NDS.
//!
//! ## Current Status: Placeholder
//!
//! This module provides the data structures for column calculations.
//! Full calculation logic will be implemented in a future phase.
//!
//! ## Assumptions (when implemented)
//!
//! - Pin-pin boundary conditions (K = 1.0)
//! - Sawn lumber rectangular section
//! - Axial load only (combined loading to be added later)
//!
//! ## Example
//!
//! ```rust
//! use calc_core::calculations::column::ColumnInput;
//! use calc_core::materials::{WoodSpecies, WoodGrade, WoodMaterial};
//!
//! let input = ColumnInput {
//!     label: "C-1".to_string(),
//!     height_ft: 10.0,
//!     axial_load_lb: 5000.0,
//!     material: WoodMaterial::new(WoodSpecies::DouglasFirLarch, WoodGrade::No2),
//!     width_in: 3.5,
//!     depth_in: 3.5,
//!     k_factor: 1.0,
//! };
//! ```

use serde::{Deserialize, Serialize};

use crate::errors::{CalcError, CalcResult};
use crate::materials::WoodMaterial;

/// Input parameters for a wood column.
///
/// ## JSON Example
///
/// ```json
/// {
///   "label": "C-1",
///   "height_ft": 10.0,
///   "axial_load_lb": 5000.0,
///   "material": {
///     "species": "DF-L",
///     "grade": "No.2"
///   },
///   "width_in": 3.5,
///   "depth_in": 3.5,
///   "k_factor": 1.0
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInput {
    /// User label for this column (e.g., "C-1", "Interior Column")
    pub label: String,

    /// Unbraced length in feet
    pub height_ft: f64,

    /// Axial compression load in pounds
    pub axial_load_lb: f64,

    /// Wood material (species and grade)
    pub material: WoodMaterial,

    /// Actual column width in inches (e.g., 3.5 for 4x4)
    pub width_in: f64,

    /// Actual column depth in inches (e.g., 3.5 for 4x4)
    pub depth_in: f64,

    /// Effective length factor K (typically 1.0 for pin-pin)
    pub k_factor: f64,
}

impl ColumnInput {
    /// Validate input parameters.
    pub fn validate(&self) -> CalcResult<()> {
        if self.height_ft <= 0.0 {
            return Err(CalcError::invalid_input(
                "height_ft",
                self.height_ft.to_string(),
                "Height must be positive",
            ));
        }
        if self.height_ft > 20.0 {
            return Err(CalcError::invalid_input(
                "height_ft",
                self.height_ft.to_string(),
                "Height exceeds 20 ft - consider steel or engineered lumber",
            ));
        }
        if self.axial_load_lb < 0.0 {
            return Err(CalcError::invalid_input(
                "axial_load_lb",
                self.axial_load_lb.to_string(),
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
        if self.k_factor <= 0.0 || self.k_factor > 2.5 {
            return Err(CalcError::invalid_input(
                "k_factor",
                self.k_factor.to_string(),
                "K factor must be between 0 and 2.5",
            ));
        }
        Ok(())
    }

    /// Calculate cross-sectional area A = bd
    pub fn area_in2(&self) -> f64 {
        self.width_in * self.depth_in
    }

    /// Calculate minimum dimension for slenderness
    pub fn min_dimension_in(&self) -> f64 {
        self.width_in.min(self.depth_in)
    }

    /// Calculate slenderness ratio le/d
    pub fn slenderness_ratio(&self) -> f64 {
        let le_in = self.height_ft * 12.0 * self.k_factor;
        le_in / self.min_dimension_in()
    }
}

/// Results from column calculation.
///
/// ## JSON Example
///
/// ```json
/// {
///   "actual_fc_psi": 408.2,
///   "allowable_fc_psi": 1350.0,
///   "axial_unity": 0.30,
///   "slenderness_ratio": 34.3,
///   "cp_factor": 0.85
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnResult {
    /// Actual compression stress fc = P/A (psi)
    pub actual_fc_psi: f64,

    /// Allowable compression stress Fc' (psi) with all adjustments
    pub allowable_fc_psi: f64,

    /// Axial unity check: actual_fc / allowable_fc
    pub axial_unity: f64,

    /// Slenderness ratio le/d
    pub slenderness_ratio: f64,

    /// Column stability factor Cp (NDS 3.7)
    pub cp_factor: f64,

    /// Reference compression stress Fc (psi) before adjustments
    pub fc_reference_psi: f64,

    /// Cross-sectional area (in²)
    pub area_in2: f64,
}

impl ColumnResult {
    /// Check if the column passes (unity ≤ 1.0)
    pub fn passes(&self) -> bool {
        self.axial_unity <= 1.0
    }
}

/// Calculate column capacity.
///
/// **Note**: This is a simplified placeholder implementation.
/// Full NDS column design with Cp factor calculation will be added later.
///
/// # Arguments
///
/// * `input` - Column parameters
///
/// # Returns
///
/// * `Ok(ColumnResult)` - Calculation results
/// * `Err(CalcError)` - If inputs are invalid
pub fn calculate(input: &ColumnInput) -> CalcResult<ColumnResult> {
    input.validate()?;

    let props = input.material.properties();
    let area = input.area_in2();
    let slenderness = input.slenderness_ratio();

    // Actual compression stress
    let actual_fc_psi = input.axial_load_lb / area;

    // Simplified Cp calculation (placeholder - full NDS 3.7 to be implemented)
    // For slenderness < 50, use simplified approach
    let cp_factor = if slenderness <= 50.0 {
        // Simplified: linear reduction from 1.0 at le/d=0 to 0.5 at le/d=50
        1.0 - 0.5 * (slenderness / 50.0)
    } else {
        // Very slender - heavily penalized
        0.2
    };

    // Allowable stress with Cp factor applied
    let allowable_fc_psi = props.fc_psi * cp_factor;

    // Unity check
    let axial_unity = actual_fc_psi / allowable_fc_psi;

    Ok(ColumnResult {
        actual_fc_psi,
        allowable_fc_psi,
        axial_unity,
        slenderness_ratio: slenderness,
        cp_factor,
        fc_reference_psi: props.fc_psi,
        area_in2: area,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::{WoodGrade, WoodSpecies};

    fn test_column() -> ColumnInput {
        ColumnInput {
            label: "Test Column".to_string(),
            height_ft: 10.0,
            axial_load_lb: 5000.0,
            material: WoodMaterial::new(WoodSpecies::DouglasFirLarch, WoodGrade::No2),
            width_in: 3.5,
            depth_in: 3.5,
            k_factor: 1.0,
        }
    }

    #[test]
    fn test_column_area() {
        let col = test_column();
        assert!((col.area_in2() - 12.25).abs() < 0.01);
    }

    #[test]
    fn test_slenderness_ratio() {
        let col = test_column();
        // le/d = (10 * 12 * 1.0) / 3.5 = 34.29
        let slenderness = col.slenderness_ratio();
        assert!((slenderness - 34.29).abs() < 0.1);
    }

    #[test]
    fn test_column_calculation() {
        let col = test_column();
        let result = calculate(&col).unwrap();

        // fc = 5000 / 12.25 = 408.16 psi
        assert!((result.actual_fc_psi - 408.16).abs() < 1.0);

        // Should pass with this light load
        assert!(result.passes());
    }

    #[test]
    fn test_invalid_height() {
        let mut col = test_column();
        col.height_ft = -5.0;
        assert!(calculate(&col).is_err());
    }

    #[test]
    fn test_serialization() {
        let col = test_column();
        let json = serde_json::to_string_pretty(&col).unwrap();
        let roundtrip: ColumnInput = serde_json::from_str(&json).unwrap();
        assert_eq!(col.height_ft, roundtrip.height_ft);
        assert_eq!(col.axial_load_lb, roundtrip.axial_load_lb);
    }
}
