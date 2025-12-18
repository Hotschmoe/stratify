//! # Structural Engineering Equations
//!
//! This module contains all fundamental structural mechanics equations used in calculations.
//! Having equations in one place enables:
//! - Easy verification against code references (NDS, AISC, ACI)
//! - Documentation of assumptions and sign conventions
//! - Consistent implementation across calculation types
//!
//! ## Modules
//!
//! - [`beam`] - Simply-supported beam formulas (moment, shear, deflection)
//! - [`section`] - Cross-section properties (S, I, A)
//!
//! ## Sign Conventions
//!
//! - **Loads**: Positive downward (gravity direction)
//! - **Moment**: Positive causes tension on bottom fiber (sagging)
//! - **Shear**: Positive when left side moves up relative to right
//! - **Deflection**: Positive downward
//! - **Reactions**: Positive upward (resisting gravity)
//!
//! ## References
//!
//! - NDS 2018: National Design Specification for Wood Construction
//! - AISC 360-22: Specification for Structural Steel Buildings
//! - Roark's Formulas for Stress and Strain, 8th Edition
//! - ASCE 7-22: Minimum Design Loads for Buildings

pub mod beam;
pub mod section;

// Re-export commonly used items
pub use beam::{
    point_load_moment,
    point_load_shear,
    point_load_deflection,
    point_load_reactions,
    uniform_load_moment,
    uniform_load_shear,
    uniform_load_deflection,
    uniform_load_reactions,
    partial_uniform_reactions,
    partial_uniform_moment,
    partial_uniform_shear,
};

pub use section::{
    rectangular_area,
    rectangular_section_modulus,
    rectangular_moment_of_inertia,
    rectangular_radius_of_gyration,
    rectangular_shear_area,
    nominal_to_actual_dimensions,
};
