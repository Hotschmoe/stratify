//! Sawn Lumber Materials (NDS 2018 Table 4A)
//!
//! Reference design values for visually graded dimension lumber.
//! All values are for 2"-4" thick members (2x10 and wider).

use serde::{Deserialize, Serialize};

use crate::errors::{CalcError, CalcResult};
use crate::units::Psi;

/// Wood species groups per NDS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum WoodSpecies {
    /// Douglas Fir-Larch
    #[serde(rename = "DF-L")]
    DouglasFirLarch,
    /// Southern Pine
    #[serde(rename = "SP")]
    SouthernPine,
    /// Hem-Fir
    #[serde(rename = "HF")]
    HemFir,
    /// Spruce-Pine-Fir
    #[serde(rename = "SPF")]
    SprucePineFir,
    /// Douglas Fir-South
    #[serde(rename = "DF-S")]
    DouglasFirSouth,
}

impl WoodSpecies {
    /// All wood species variants for UI selection
    pub const ALL: [WoodSpecies; 5] = [
        WoodSpecies::DouglasFirLarch,
        WoodSpecies::SouthernPine,
        WoodSpecies::HemFir,
        WoodSpecies::SprucePineFir,
        WoodSpecies::DouglasFirSouth,
    ];

    /// Parse from common string representations
    pub fn from_str_flexible(s: &str) -> CalcResult<Self> {
        match s.to_uppercase().replace([' ', '_'], "-").as_str() {
            "DF-L" | "DOUGLAS-FIR-LARCH" | "DFL" => Ok(WoodSpecies::DouglasFirLarch),
            "SP" | "SOUTHERN-PINE" => Ok(WoodSpecies::SouthernPine),
            "HF" | "HEM-FIR" => Ok(WoodSpecies::HemFir),
            "SPF" | "SPRUCE-PINE-FIR" => Ok(WoodSpecies::SprucePineFir),
            "DF-S" | "DOUGLAS-FIR-SOUTH" | "DFS" => Ok(WoodSpecies::DouglasFirSouth),
            _ => Err(CalcError::material_not_found(s)),
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            WoodSpecies::DouglasFirLarch => "Douglas Fir-Larch",
            WoodSpecies::SouthernPine => "Southern Pine",
            WoodSpecies::HemFir => "Hem-Fir",
            WoodSpecies::SprucePineFir => "Spruce-Pine-Fir",
            WoodSpecies::DouglasFirSouth => "Douglas Fir-South",
        }
    }
}

impl std::fmt::Display for WoodSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Wood grades per NDS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WoodGrade {
    /// Select Structural
    #[serde(rename = "SS")]
    SelectStructural,
    /// No. 1
    #[serde(rename = "No.1")]
    No1,
    /// No. 2
    #[serde(rename = "No.2")]
    No2,
    /// No. 3
    #[serde(rename = "No.3")]
    No3,
    /// Stud
    Stud,
    /// Construction
    Construction,
    /// Standard
    Standard,
    /// Utility
    Utility,
}

impl WoodGrade {
    /// All wood grade variants for UI selection
    pub const ALL: [WoodGrade; 8] = [
        WoodGrade::SelectStructural,
        WoodGrade::No1,
        WoodGrade::No2,
        WoodGrade::No3,
        WoodGrade::Stud,
        WoodGrade::Construction,
        WoodGrade::Standard,
        WoodGrade::Utility,
    ];

    /// Parse from common string representations
    pub fn from_str_flexible(s: &str) -> CalcResult<Self> {
        match s.to_uppercase().replace([' ', '.', '#'], "").as_str() {
            "SS" | "SELECTSTRUCTURAL" | "SELSTR" => Ok(WoodGrade::SelectStructural),
            "NO1" | "1" | "N1" => Ok(WoodGrade::No1),
            "NO2" | "2" | "N2" => Ok(WoodGrade::No2),
            "NO3" | "3" | "N3" => Ok(WoodGrade::No3),
            "STUD" => Ok(WoodGrade::Stud),
            "CONST" | "CONSTRUCTION" => Ok(WoodGrade::Construction),
            "STANDARD" | "STD" => Ok(WoodGrade::Standard),
            "UTILITY" | "UTIL" => Ok(WoodGrade::Utility),
            _ => Err(CalcError::material_not_found(s)),
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            WoodGrade::SelectStructural => "Select Structural",
            WoodGrade::No1 => "No. 1",
            WoodGrade::No2 => "No. 2",
            WoodGrade::No3 => "No. 3",
            WoodGrade::Stud => "Stud",
            WoodGrade::Construction => "Construction",
            WoodGrade::Standard => "Standard",
            WoodGrade::Utility => "Utility",
        }
    }
}

impl std::fmt::Display for WoodGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Reference design values for wood (NDS Table 4A)
///
/// All values are in psi unless otherwise noted.
/// These are base values before adjustment factors.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WoodProperties {
    /// Species
    pub species: WoodSpecies,
    /// Grade
    pub grade: WoodGrade,
    /// Bending stress Fb (psi)
    pub fb_psi: f64,
    /// Tension parallel to grain Ft (psi)
    pub ft_psi: f64,
    /// Shear parallel to grain Fv (psi)
    pub fv_psi: f64,
    /// Compression perpendicular to grain Fc_perp (psi)
    pub fc_perp_psi: f64,
    /// Compression parallel to grain Fc (psi)
    pub fc_psi: f64,
    /// Modulus of elasticity E (psi)
    pub e_psi: f64,
    /// Minimum modulus of elasticity Emin (psi)
    pub e_min_psi: f64,
    /// Specific gravity G
    pub specific_gravity: f64,
}

impl WoodProperties {
    /// Look up wood properties by species and grade.
    ///
    /// # Example
    ///
    /// ```rust
    /// use calc_core::materials::{WoodSpecies, WoodGrade, WoodProperties};
    ///
    /// let props = WoodProperties::lookup(WoodSpecies::DouglasFirLarch, WoodGrade::No2);
    /// assert!(props.fb_psi > 0.0);
    /// ```
    pub fn lookup(species: WoodSpecies, grade: WoodGrade) -> Self {
        // NDS 2018 Table 4A - Visually Graded Dimension Lumber (2" - 4" thick)
        // Values for 2x10 and wider
        match (species, grade) {
            // Douglas Fir-Larch
            (WoodSpecies::DouglasFirLarch, WoodGrade::SelectStructural) => WoodProperties {
                species,
                grade,
                fb_psi: 1500.0,
                ft_psi: 1000.0,
                fv_psi: 180.0,
                fc_perp_psi: 625.0,
                fc_psi: 1700.0,
                e_psi: 1_900_000.0,
                e_min_psi: 690_000.0,
                specific_gravity: 0.50,
            },
            (WoodSpecies::DouglasFirLarch, WoodGrade::No1) => WoodProperties {
                species,
                grade,
                fb_psi: 1200.0,
                ft_psi: 800.0,
                fv_psi: 180.0,
                fc_perp_psi: 625.0,
                fc_psi: 1550.0,
                e_psi: 1_700_000.0,
                e_min_psi: 620_000.0,
                specific_gravity: 0.50,
            },
            (WoodSpecies::DouglasFirLarch, WoodGrade::No2) => WoodProperties {
                species,
                grade,
                fb_psi: 900.0,
                ft_psi: 575.0,
                fv_psi: 180.0,
                fc_perp_psi: 625.0,
                fc_psi: 1350.0,
                e_psi: 1_600_000.0,
                e_min_psi: 580_000.0,
                specific_gravity: 0.50,
            },
            (WoodSpecies::DouglasFirLarch, WoodGrade::No3) => WoodProperties {
                species,
                grade,
                fb_psi: 525.0,
                ft_psi: 325.0,
                fv_psi: 180.0,
                fc_perp_psi: 625.0,
                fc_psi: 775.0,
                e_psi: 1_400_000.0,
                e_min_psi: 510_000.0,
                specific_gravity: 0.50,
            },

            // Southern Pine
            (WoodSpecies::SouthernPine, WoodGrade::SelectStructural) => WoodProperties {
                species,
                grade,
                fb_psi: 1500.0,
                ft_psi: 1000.0,
                fv_psi: 175.0,
                fc_perp_psi: 565.0,
                fc_psi: 1800.0,
                e_psi: 1_800_000.0,
                e_min_psi: 660_000.0,
                specific_gravity: 0.55,
            },
            (WoodSpecies::SouthernPine, WoodGrade::No1) => WoodProperties {
                species,
                grade,
                fb_psi: 1250.0,
                ft_psi: 825.0,
                fv_psi: 175.0,
                fc_perp_psi: 565.0,
                fc_psi: 1650.0,
                e_psi: 1_700_000.0,
                e_min_psi: 620_000.0,
                specific_gravity: 0.55,
            },
            (WoodSpecies::SouthernPine, WoodGrade::No2) => WoodProperties {
                species,
                grade,
                fb_psi: 850.0,
                ft_psi: 550.0,
                fv_psi: 175.0,
                fc_perp_psi: 565.0,
                fc_psi: 1450.0,
                e_psi: 1_400_000.0,
                e_min_psi: 510_000.0,
                specific_gravity: 0.55,
            },
            (WoodSpecies::SouthernPine, WoodGrade::No3) => WoodProperties {
                species,
                grade,
                fb_psi: 500.0,
                ft_psi: 300.0,
                fv_psi: 175.0,
                fc_perp_psi: 565.0,
                fc_psi: 825.0,
                e_psi: 1_200_000.0,
                e_min_psi: 440_000.0,
                specific_gravity: 0.55,
            },

            // Hem-Fir
            (WoodSpecies::HemFir, WoodGrade::SelectStructural) => WoodProperties {
                species,
                grade,
                fb_psi: 1400.0,
                ft_psi: 925.0,
                fv_psi: 150.0,
                fc_perp_psi: 405.0,
                fc_psi: 1500.0,
                e_psi: 1_600_000.0,
                e_min_psi: 580_000.0,
                specific_gravity: 0.43,
            },
            (WoodSpecies::HemFir, WoodGrade::No1) => WoodProperties {
                species,
                grade,
                fb_psi: 1100.0,
                ft_psi: 725.0,
                fv_psi: 150.0,
                fc_perp_psi: 405.0,
                fc_psi: 1350.0,
                e_psi: 1_500_000.0,
                e_min_psi: 550_000.0,
                specific_gravity: 0.43,
            },
            (WoodSpecies::HemFir, WoodGrade::No2) => WoodProperties {
                species,
                grade,
                fb_psi: 850.0,
                ft_psi: 525.0,
                fv_psi: 150.0,
                fc_perp_psi: 405.0,
                fc_psi: 1300.0,
                e_psi: 1_300_000.0,
                e_min_psi: 470_000.0,
                specific_gravity: 0.43,
            },
            (WoodSpecies::HemFir, WoodGrade::No3) => WoodProperties {
                species,
                grade,
                fb_psi: 500.0,
                ft_psi: 300.0,
                fv_psi: 150.0,
                fc_perp_psi: 405.0,
                fc_psi: 750.0,
                e_psi: 1_200_000.0,
                e_min_psi: 440_000.0,
                specific_gravity: 0.43,
            },

            // SPF (Spruce-Pine-Fir)
            (WoodSpecies::SprucePineFir, WoodGrade::SelectStructural) => WoodProperties {
                species,
                grade,
                fb_psi: 1250.0,
                ft_psi: 825.0,
                fv_psi: 135.0,
                fc_perp_psi: 425.0,
                fc_psi: 1400.0,
                e_psi: 1_500_000.0,
                e_min_psi: 550_000.0,
                specific_gravity: 0.42,
            },
            (WoodSpecies::SprucePineFir, WoodGrade::No1) => WoodProperties {
                species,
                grade,
                fb_psi: 1000.0,
                ft_psi: 650.0,
                fv_psi: 135.0,
                fc_perp_psi: 425.0,
                fc_psi: 1250.0,
                e_psi: 1_400_000.0,
                e_min_psi: 510_000.0,
                specific_gravity: 0.42,
            },
            (WoodSpecies::SprucePineFir, WoodGrade::No2) => WoodProperties {
                species,
                grade,
                fb_psi: 875.0,
                ft_psi: 450.0,
                fv_psi: 135.0,
                fc_perp_psi: 425.0,
                fc_psi: 1150.0,
                e_psi: 1_400_000.0,
                e_min_psi: 510_000.0,
                specific_gravity: 0.42,
            },
            (WoodSpecies::SprucePineFir, WoodGrade::No3) => WoodProperties {
                species,
                grade,
                fb_psi: 500.0,
                ft_psi: 250.0,
                fv_psi: 135.0,
                fc_perp_psi: 425.0,
                fc_psi: 650.0,
                e_psi: 1_200_000.0,
                e_min_psi: 440_000.0,
                specific_gravity: 0.42,
            },

            // Douglas Fir-South
            (WoodSpecies::DouglasFirSouth, WoodGrade::SelectStructural) => WoodProperties {
                species,
                grade,
                fb_psi: 1350.0,
                ft_psi: 900.0,
                fv_psi: 180.0,
                fc_perp_psi: 520.0,
                fc_psi: 1600.0,
                e_psi: 1_400_000.0,
                e_min_psi: 510_000.0,
                specific_gravity: 0.46,
            },
            (WoodSpecies::DouglasFirSouth, WoodGrade::No1) => WoodProperties {
                species,
                grade,
                fb_psi: 1050.0,
                ft_psi: 700.0,
                fv_psi: 180.0,
                fc_perp_psi: 520.0,
                fc_psi: 1450.0,
                e_psi: 1_200_000.0,
                e_min_psi: 440_000.0,
                specific_gravity: 0.46,
            },
            (WoodSpecies::DouglasFirSouth, WoodGrade::No2) => WoodProperties {
                species,
                grade,
                fb_psi: 875.0,
                ft_psi: 525.0,
                fv_psi: 180.0,
                fc_perp_psi: 520.0,
                fc_psi: 1350.0,
                e_psi: 1_100_000.0,
                e_min_psi: 400_000.0,
                specific_gravity: 0.46,
            },
            (WoodSpecies::DouglasFirSouth, WoodGrade::No3) => WoodProperties {
                species,
                grade,
                fb_psi: 500.0,
                ft_psi: 300.0,
                fv_psi: 180.0,
                fc_perp_psi: 520.0,
                fc_psi: 775.0,
                e_psi: 1_000_000.0,
                e_min_psi: 370_000.0,
                specific_gravity: 0.46,
            },

            // Stud grade - use similar to No.3 for structural calcs
            (species, WoodGrade::Stud) => {
                let base = Self::lookup(species, WoodGrade::No3);
                WoodProperties {
                    grade: WoodGrade::Stud,
                    ..base
                }
            }

            // Construction, Standard, Utility - less common, approximate
            (species, WoodGrade::Construction) => {
                let base = Self::lookup(species, WoodGrade::No2);
                WoodProperties {
                    grade: WoodGrade::Construction,
                    fb_psi: base.fb_psi * 1.15,
                    ..base
                }
            }
            (species, WoodGrade::Standard) => {
                let base = Self::lookup(species, WoodGrade::No3);
                WoodProperties {
                    grade: WoodGrade::Standard,
                    ..base
                }
            }
            (species, WoodGrade::Utility) => {
                let base = Self::lookup(species, WoodGrade::No3);
                WoodProperties {
                    grade: WoodGrade::Utility,
                    fb_psi: base.fb_psi * 0.6,
                    fc_psi: base.fc_psi * 0.6,
                    ..base
                }
            }
        }
    }

    /// Get Fb as a typed unit
    pub fn fb(&self) -> Psi {
        Psi(self.fb_psi)
    }

    /// Get Fv as a typed unit
    pub fn fv(&self) -> Psi {
        Psi(self.fv_psi)
    }

    /// Get E as a typed unit
    pub fn e(&self) -> Psi {
        Psi(self.e_psi)
    }
}

/// Combined wood material identifier for serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WoodMaterial {
    pub species: WoodSpecies,
    pub grade: WoodGrade,
}

impl WoodMaterial {
    pub fn new(species: WoodSpecies, grade: WoodGrade) -> Self {
        Self { species, grade }
    }

    /// Parse from string like "DF-L No.2"
    pub fn from_str_flexible(s: &str) -> CalcResult<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(CalcError::invalid_input(
                "material",
                s,
                "Expected format: 'SPECIES GRADE' (e.g., 'DF-L No.2')",
            ));
        }
        Ok(Self {
            species: WoodSpecies::from_str_flexible(parts[0])?,
            grade: WoodGrade::from_str_flexible(parts[1])?,
        })
    }

    /// Get properties for this material
    pub fn properties(&self) -> WoodProperties {
        WoodProperties::lookup(self.species, self.grade)
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        format!("{} {}", self.species.display_name(), self.grade.display_name())
    }
}

impl std::fmt::Display for WoodMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wood_lookup() {
        let props = WoodProperties::lookup(WoodSpecies::DouglasFirLarch, WoodGrade::No2);
        assert_eq!(props.fb_psi, 900.0);
        assert_eq!(props.e_psi, 1_600_000.0);
    }

    #[test]
    fn test_species_parsing() {
        assert_eq!(
            WoodSpecies::from_str_flexible("DF-L").unwrap(),
            WoodSpecies::DouglasFirLarch
        );
        assert_eq!(
            WoodSpecies::from_str_flexible("southern pine").unwrap(),
            WoodSpecies::SouthernPine
        );
    }

    #[test]
    fn test_grade_parsing() {
        assert_eq!(WoodGrade::from_str_flexible("No.2").unwrap(), WoodGrade::No2);
        assert_eq!(WoodGrade::from_str_flexible("#2").unwrap(), WoodGrade::No2);
        assert_eq!(WoodGrade::from_str_flexible("SS").unwrap(), WoodGrade::SelectStructural);
    }

    #[test]
    fn test_material_parsing() {
        let mat = WoodMaterial::from_str_flexible("DF-L No.2").unwrap();
        assert_eq!(mat.species, WoodSpecies::DouglasFirLarch);
        assert_eq!(mat.grade, WoodGrade::No2);
    }

    #[test]
    fn test_serialization() {
        let props = WoodProperties::lookup(WoodSpecies::SouthernPine, WoodGrade::No1);
        let json = serde_json::to_string(&props).unwrap();
        let roundtrip: WoodProperties = serde_json::from_str(&json).unwrap();
        assert_eq!(props.fb_psi, roundtrip.fb_psi);
    }

    #[test]
    fn test_material_display() {
        let mat = WoodMaterial::new(WoodSpecies::DouglasFirLarch, WoodGrade::No2);
        assert_eq!(mat.display_name(), "Douglas Fir-Larch No. 2");
    }
}
