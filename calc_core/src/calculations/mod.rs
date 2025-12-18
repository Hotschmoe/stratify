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
//! - [`continuous_beam`] - Multi-span beam analysis with configurable supports
//! - [`beam_analysis`] - Detailed beam analysis with superposition
//! - [`column`] - Axial compression member analysis (wood)

pub mod beam;
pub mod beam_analysis;
pub mod column;
pub mod continuous_beam;
pub mod moment_distribution;

use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use beam::{BeamInput, BeamResult};
pub use beam_analysis::{AnalysisResults, BeamAnalysis, SingleLoad};
pub use column::{ColumnInput, ColumnResult};
pub use continuous_beam::{
    calculate_continuous, ContinuousBeamInput, ContinuousBeamResult, SpanResult, SpanSegment,
    SupportType,
};

/// Enum wrapper for all calculation types.
///
/// This allows storing heterogeneous calculations in a single collection
/// while maintaining type safety and clean serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CalculationItem {
    /// Multi-span continuous beam calculation
    ///
    /// Supports single-span simply-supported, cantilever, fixed-fixed,
    /// propped cantilever, and multi-span continuous beams with any
    /// combination of support conditions.
    Beam(ContinuousBeamInput),
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
