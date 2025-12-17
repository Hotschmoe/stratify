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
//! - [`column`] - Axial compression member analysis (wood)

pub mod beam;
pub mod column;

use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use beam::{BeamInput, BeamResult};
pub use column::{ColumnInput, ColumnResult};

/// Enum wrapper for all calculation types.
///
/// This allows storing heterogeneous calculations in a single collection
/// while maintaining type safety and clean serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CalculationItem {
    /// Simply-supported beam calculation
    Beam(BeamInput),
    /// Axial compression column calculation
    Column(ColumnInput),
    // Future: ShearWall(ShearWallInput),
    // etc.
}

impl CalculationItem {
    /// Get the user-provided label for this calculation
    pub fn label(&self) -> &str {
        match self {
            CalculationItem::Beam(b) => &b.label,
            CalculationItem::Column(c) => &c.label,
        }
    }

    /// Get the calculation type as a string
    pub fn calc_type(&self) -> &'static str {
        match self {
            CalculationItem::Beam(_) => "Beam",
            CalculationItem::Column(_) => "Column",
        }
    }
}
