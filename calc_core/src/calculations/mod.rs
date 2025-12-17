//! # Structural Calculations
//!
//! This module contains all structural calculation types. Each calculation
//! follows the pattern:
//!
//! - `*Input` - Input parameters (JSON-serializable)
//! - `*Result` - Calculation results (JSON-serializable)
//! - `calculate(input) -> Result<*Result, CalcError>` - Pure calculation function
//!
//! ## LLM Integration
//!
//! All types are designed for LLM consumption:
//! - Comprehensive rustdoc with examples
//! - Clean JSON serialization
//! - Structured error responses
//!
//! ## Available Calculations
//!
//! - [`beam`] - Simply-supported beam analysis (wood)

pub mod beam;

use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use beam::{BeamInput, BeamResult};

/// Enum wrapper for all calculation types.
///
/// This allows storing heterogeneous calculations in a single collection
/// while maintaining type safety and clean serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CalculationItem {
    /// Simply-supported beam calculation
    Beam(BeamInput),
    // Future: Column(ColumnInput),
    // Future: ShearWall(ShearWallInput),
    // etc.
}

impl CalculationItem {
    /// Get the user-provided label for this calculation
    pub fn label(&self) -> &str {
        match self {
            CalculationItem::Beam(b) => &b.label,
        }
    }

    /// Get the calculation type as a string
    pub fn calc_type(&self) -> &'static str {
        match self {
            CalculationItem::Beam(_) => "Beam",
        }
    }
}
