//! ASCE 7-22 Load Combinations
//!
//! This module provides standard load combinations per ASCE 7-22 for both
//! Allowable Stress Design (ASD) and Load and Resistance Factor Design (LRFD).

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::load_types::LoadType;
use super::LoadCase;

/// A load combination with factors for each load type
///
/// Load combinations define how different load types are combined and factored
/// to determine design loads. Each combination has a name, human-readable equation,
/// and a set of factors to apply to each load type.
///
/// # Example
/// ```
/// use calc_core::loads::{LoadCombination, LoadCase, LoadType};
/// use std::collections::HashMap;
///
/// let combo = LoadCombination {
///     name: "ASD-2".to_string(),
///     equation: "D + L".to_string(),
///     factors: vec![
///         (LoadType::Dead, 1.0),
///         (LoadType::Live, 1.0),
///     ].into_iter().collect(),
/// };
///
/// let case = LoadCase::new("Floor")
///     .with_load(LoadType::Dead, 20.0)
///     .with_load(LoadType::Live, 50.0);
///
/// assert_eq!(combo.apply(&case), 70.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadCombination {
    /// Combination identifier (e.g., "ASD-1", "LRFD-2a")
    pub name: String,

    /// Human-readable equation for display (e.g., "D + L", "1.2D + 1.6L")
    pub equation: String,

    /// Load factors keyed by load type
    pub factors: HashMap<LoadType, f64>,
}

impl LoadCombination {
    /// Create a new load combination
    pub fn new(name: impl Into<String>, equation: impl Into<String>) -> Self {
        LoadCombination {
            name: name.into(),
            equation: equation.into(),
            factors: HashMap::new(),
        }
    }

    /// Add a load factor (builder pattern)
    pub fn with_factor(mut self, load_type: LoadType, factor: f64) -> Self {
        self.factors.insert(load_type, factor);
        self
    }

    /// Apply this combination to a LoadCase, returning the total factored load
    ///
    /// Load types not in the combination are treated as having factor 0.
    /// Load types in the combination but not in the case are treated as 0 load.
    pub fn apply(&self, case: &LoadCase) -> f64 {
        self.factors
            .iter()
            .map(|(load_type, factor)| factor * case.get(*load_type))
            .sum()
    }

    /// Get the factor for a specific load type (0.0 if not in combination)
    pub fn get_factor(&self, load_type: LoadType) -> f64 {
        self.factors.get(&load_type).copied().unwrap_or(0.0)
    }
}

/// Generate ASCE 7-22 ASD load combinations (Section 2.4.1)
///
/// Returns all standard ASD combinations including alternates for roof loads.
/// The combinations account for:
/// - Basic gravity combinations (D, D+L, D+Lr/S/R)
/// - Combined gravity and lateral (D+0.75L+0.75Lr/S/R)
/// - Lateral load combinations (D+W, D+E)
/// - Uplift/overturning combinations (0.6D+W, 0.6D+E)
///
/// # Example
/// ```
/// use calc_core::loads::asce7_asd_combinations;
///
/// let combos = asce7_asd_combinations();
/// assert!(combos.len() >= 9);
/// ```
pub fn asce7_asd_combinations() -> Vec<LoadCombination> {
    vec![
        // 1. D
        LoadCombination::new("ASD-1", "D")
            .with_factor(LoadType::Dead, 1.0),

        // 2. D + L
        LoadCombination::new("ASD-2", "D + L")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 1.0),

        // 3a. D + Lr
        LoadCombination::new("ASD-3a", "D + Lr")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::LiveRoof, 1.0),

        // 3b. D + S
        LoadCombination::new("ASD-3b", "D + S")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Snow, 1.0),

        // 3c. D + R
        LoadCombination::new("ASD-3c", "D + R")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Rain, 1.0),

        // 4a. D + 0.75L + 0.75Lr
        LoadCombination::new("ASD-4a", "D + 0.75L + 0.75Lr")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 0.75)
            .with_factor(LoadType::LiveRoof, 0.75),

        // 4b. D + 0.75L + 0.75S
        LoadCombination::new("ASD-4b", "D + 0.75L + 0.75S")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 0.75)
            .with_factor(LoadType::Snow, 0.75),

        // 4c. D + 0.75L + 0.75R
        LoadCombination::new("ASD-4c", "D + 0.75L + 0.75R")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 0.75)
            .with_factor(LoadType::Rain, 0.75),

        // 5a. D + 0.6W
        LoadCombination::new("ASD-5a", "D + 0.6W")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Wind, 0.6),

        // 5b. D + 0.7E
        LoadCombination::new("ASD-5b", "D + 0.7E")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Seismic, 0.7),

        // 6a. D + 0.75L + 0.75(0.6W) + 0.75Lr
        LoadCombination::new("ASD-6a", "D + 0.75L + 0.45W + 0.75Lr")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 0.75)
            .with_factor(LoadType::Wind, 0.45) // 0.75 * 0.6
            .with_factor(LoadType::LiveRoof, 0.75),

        // 6b. D + 0.75L + 0.75(0.6W) + 0.75S
        LoadCombination::new("ASD-6b", "D + 0.75L + 0.45W + 0.75S")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 0.75)
            .with_factor(LoadType::Wind, 0.45)
            .with_factor(LoadType::Snow, 0.75),

        // 6c. D + 0.75L + 0.75(0.6W) + 0.75R
        LoadCombination::new("ASD-6c", "D + 0.75L + 0.45W + 0.75R")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 0.75)
            .with_factor(LoadType::Wind, 0.45)
            .with_factor(LoadType::Rain, 0.75),

        // 7. D + 0.75L + 0.75(0.7E) + 0.75S
        LoadCombination::new("ASD-7", "D + 0.75L + 0.525E + 0.75S")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 0.75)
            .with_factor(LoadType::Seismic, 0.525) // 0.75 * 0.7
            .with_factor(LoadType::Snow, 0.75),

        // 8. 0.6D + 0.6W
        LoadCombination::new("ASD-8", "0.6D + 0.6W")
            .with_factor(LoadType::Dead, 0.6)
            .with_factor(LoadType::Wind, 0.6),

        // 9. 0.6D + 0.7E
        LoadCombination::new("ASD-9", "0.6D + 0.7E")
            .with_factor(LoadType::Dead, 0.6)
            .with_factor(LoadType::Seismic, 0.7),
    ]
}

/// Generate ASCE 7-22 LRFD load combinations (Section 2.3.1)
///
/// Returns all standard LRFD combinations including alternates.
/// LRFD uses factored loads to account for uncertainty in both
/// loads and resistance.
///
/// # Example
/// ```
/// use calc_core::loads::asce7_lrfd_combinations;
///
/// let combos = asce7_lrfd_combinations();
/// let lrfd1 = combos.iter().find(|c| c.name == "LRFD-1").unwrap();
/// assert_eq!(lrfd1.get_factor(calc_core::loads::LoadType::Dead), 1.4);
/// ```
pub fn asce7_lrfd_combinations() -> Vec<LoadCombination> {
    vec![
        // 1. 1.4D
        LoadCombination::new("LRFD-1", "1.4D")
            .with_factor(LoadType::Dead, 1.4),

        // 2a. 1.2D + 1.6L + 0.5Lr
        LoadCombination::new("LRFD-2a", "1.2D + 1.6L + 0.5Lr")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Live, 1.6)
            .with_factor(LoadType::LiveRoof, 0.5),

        // 2b. 1.2D + 1.6L + 0.5S
        LoadCombination::new("LRFD-2b", "1.2D + 1.6L + 0.5S")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Live, 1.6)
            .with_factor(LoadType::Snow, 0.5),

        // 2c. 1.2D + 1.6L + 0.5R
        LoadCombination::new("LRFD-2c", "1.2D + 1.6L + 0.5R")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Live, 1.6)
            .with_factor(LoadType::Rain, 0.5),

        // 3a. 1.2D + 1.6Lr + L
        LoadCombination::new("LRFD-3a", "1.2D + 1.6Lr + L")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::LiveRoof, 1.6)
            .with_factor(LoadType::Live, 1.0),

        // 3b. 1.2D + 1.6Lr + 0.5W
        LoadCombination::new("LRFD-3b", "1.2D + 1.6Lr + 0.5W")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::LiveRoof, 1.6)
            .with_factor(LoadType::Wind, 0.5),

        // 3c. 1.2D + 1.6S + L
        LoadCombination::new("LRFD-3c", "1.2D + 1.6S + L")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Snow, 1.6)
            .with_factor(LoadType::Live, 1.0),

        // 3d. 1.2D + 1.6S + 0.5W
        LoadCombination::new("LRFD-3d", "1.2D + 1.6S + 0.5W")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Snow, 1.6)
            .with_factor(LoadType::Wind, 0.5),

        // 3e. 1.2D + 1.6R + L
        LoadCombination::new("LRFD-3e", "1.2D + 1.6R + L")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Rain, 1.6)
            .with_factor(LoadType::Live, 1.0),

        // 3f. 1.2D + 1.6R + 0.5W
        LoadCombination::new("LRFD-3f", "1.2D + 1.6R + 0.5W")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Rain, 1.6)
            .with_factor(LoadType::Wind, 0.5),

        // 4a. 1.2D + 1.0W + L + 0.5Lr
        LoadCombination::new("LRFD-4a", "1.2D + 1.0W + L + 0.5Lr")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Wind, 1.0)
            .with_factor(LoadType::Live, 1.0)
            .with_factor(LoadType::LiveRoof, 0.5),

        // 4b. 1.2D + 1.0W + L + 0.5S
        LoadCombination::new("LRFD-4b", "1.2D + 1.0W + L + 0.5S")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Wind, 1.0)
            .with_factor(LoadType::Live, 1.0)
            .with_factor(LoadType::Snow, 0.5),

        // 4c. 1.2D + 1.0W + L + 0.5R
        LoadCombination::new("LRFD-4c", "1.2D + 1.0W + L + 0.5R")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Wind, 1.0)
            .with_factor(LoadType::Live, 1.0)
            .with_factor(LoadType::Rain, 0.5),

        // 5. 1.2D + 1.0E + L + 0.2S
        LoadCombination::new("LRFD-5", "1.2D + 1.0E + L + 0.2S")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Seismic, 1.0)
            .with_factor(LoadType::Live, 1.0)
            .with_factor(LoadType::Snow, 0.2),

        // 6. 0.9D + 1.0W
        LoadCombination::new("LRFD-6", "0.9D + 1.0W")
            .with_factor(LoadType::Dead, 0.9)
            .with_factor(LoadType::Wind, 1.0),

        // 7. 0.9D + 1.0E
        LoadCombination::new("LRFD-7", "0.9D + 1.0E")
            .with_factor(LoadType::Dead, 0.9)
            .with_factor(LoadType::Seismic, 1.0),
    ]
}

/// Find the governing (maximum) load combination result
///
/// Applies all combinations to the given load case and returns the maximum
/// factored load along with the governing combination name.
///
/// # Example
/// ```
/// use calc_core::loads::{LoadCase, LoadType, asce7_asd_combinations, find_governing_combination};
///
/// let case = LoadCase::new("Floor")
///     .with_load(LoadType::Dead, 20.0)
///     .with_load(LoadType::Live, 50.0);
///
/// let (max_load, combo_name) = find_governing_combination(&case, &asce7_asd_combinations());
/// assert!(max_load >= 70.0); // At least D + L
/// ```
pub fn find_governing_combination(
    case: &LoadCase,
    combinations: &[LoadCombination],
) -> (f64, String) {
    combinations
        .iter()
        .map(|combo| (combo.apply(case), combo.name.clone()))
        .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((0.0, String::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asd_combination_count() {
        let combos = asce7_asd_combinations();
        // 16 combinations (9 basic + alternates for Lr/S/R)
        assert_eq!(combos.len(), 16);
    }

    #[test]
    fn test_lrfd_combination_count() {
        let combos = asce7_lrfd_combinations();
        // 16 combinations (7 basic + alternates)
        assert_eq!(combos.len(), 16);
    }

    #[test]
    fn test_lrfd_dead_only_factor() {
        let combos = asce7_lrfd_combinations();
        let lrfd1 = combos.iter().find(|c| c.name == "LRFD-1").unwrap();
        assert_eq!(lrfd1.get_factor(LoadType::Dead), 1.4);
        assert_eq!(lrfd1.get_factor(LoadType::Live), 0.0);
    }

    #[test]
    fn test_apply_combination() {
        let case = LoadCase::new("Test")
            .with_load(LoadType::Dead, 20.0)
            .with_load(LoadType::Live, 40.0);

        // D + L = 20 + 40 = 60
        let combo = LoadCombination::new("Test", "D + L")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 1.0);
        assert_eq!(combo.apply(&case), 60.0);

        // 1.2D + 1.6L = 24 + 64 = 88
        let combo_lrfd = LoadCombination::new("Test", "1.2D + 1.6L")
            .with_factor(LoadType::Dead, 1.2)
            .with_factor(LoadType::Live, 1.6);
        assert!((combo_lrfd.apply(&case) - 88.0).abs() < 0.001);
    }

    #[test]
    fn test_find_governing_asd() {
        let case = LoadCase::new("Floor")
            .with_load(LoadType::Dead, 20.0)
            .with_load(LoadType::Live, 50.0);

        let combos = asce7_asd_combinations();
        let (max_load, _name) = find_governing_combination(&case, &combos);

        // D + L should be 70, which should govern for these loads
        assert!((max_load - 70.0).abs() < 0.001);
    }

    #[test]
    fn test_find_governing_lrfd() {
        let case = LoadCase::new("Floor")
            .with_load(LoadType::Dead, 20.0)
            .with_load(LoadType::Live, 50.0);

        let combos = asce7_lrfd_combinations();
        let (max_load, name) = find_governing_combination(&case, &combos);

        // 1.2D + 1.6L = 24 + 80 = 104 should govern
        assert!((max_load - 104.0).abs() < 0.001);
        assert!(name.starts_with("LRFD-2"));
    }

    #[test]
    fn test_combination_serialization() {
        let combo = LoadCombination::new("ASD-1", "D")
            .with_factor(LoadType::Dead, 1.0);

        let json = serde_json::to_string(&combo).unwrap();
        let parsed: LoadCombination = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "ASD-1");
        assert_eq!(parsed.get_factor(LoadType::Dead), 1.0);
    }

    #[test]
    fn test_zero_load_handling() {
        let case = LoadCase::new("Dead only")
            .with_load(LoadType::Dead, 100.0);
        // Live load not set (defaults to 0)

        let combo = LoadCombination::new("Test", "D + L")
            .with_factor(LoadType::Dead, 1.0)
            .with_factor(LoadType::Live, 1.0);

        // Should be 100 + 0 = 100
        assert_eq!(combo.apply(&case), 100.0);
    }
}
