//! Shared UI components reusable across input/result modules
//!
//! Contains:
//! - `diagrams` - Canvas drawing utilities for beam diagrams
//! - `divider` - Resizable panel divider

pub mod diagrams;
pub mod divider;

// Re-exports accessed via shared::diagrams::{BeamDiagram, BeamDiagramData}
// Re-exports accessed via shared::divider::view_divider
