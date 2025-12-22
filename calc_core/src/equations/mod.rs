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
//! - [`registry`] - Equation metadata and tracking for PDF appendix generation
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
pub mod registry;
pub mod section;

// Re-export commonly used items
pub use beam::{
    // Simply-supported formulas
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
    // Fixed-end moments (for moment distribution)
    fem_uniform_full,
    fem_point_load,
    fem_partial_uniform,
    // Fixed-fixed beam formulas
    fixed_fixed_uniform_reactions,
    fixed_fixed_uniform_end_moments,
    fixed_fixed_uniform_max_positive_moment,
    fixed_fixed_uniform_moment,
    fixed_fixed_uniform_shear,
    fixed_fixed_uniform_deflection,
    fixed_fixed_uniform_max_deflection,
    fixed_fixed_point_reactions,
    fixed_fixed_point_end_moments,
    // Cantilever formulas
    cantilever_uniform_reactions,
    cantilever_uniform_shear,
    cantilever_uniform_moment,
    cantilever_uniform_deflection,
    cantilever_uniform_max_deflection,
    cantilever_point_reactions,
    cantilever_point_moment,
    cantilever_point_deflection,
    // Propped cantilever (fixed-pinned)
    fixed_pinned_uniform_reactions,
    fixed_pinned_uniform_moment,
    fixed_pinned_uniform_max_positive_moment,
};

pub use section::{
    rectangular_area,
    rectangular_section_modulus,
    rectangular_moment_of_inertia,
    rectangular_radius_of_gyration,
    rectangular_shear_area,
    nominal_to_actual_dimensions,
};

pub use registry::{
    CodeReference,
    Equation,
    EquationCategory,
    EquationMetadata,
    EquationTracker,
    EquationUsage,
    Variable,
    ALL_EQUATIONS,
    beam_calculation_equations,
    generate_equations_markdown,
    generate_static_equations_appendix_typst,
};
