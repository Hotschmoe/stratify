//! # Equation Registry
//!
//! Central registry of all structural engineering equations used in calculations.
//! Each equation has metadata including code references, formulas, and variable definitions.
//!
//! ## Architecture
//!
//! The registry provides:
//! - Type-safe equation identification via the `Equation` enum
//! - Full metadata for PDF generation and audit trails
//! - Serialization support for JSON export
//!
//! ## Usage
//!
//! ```rust
//! use calc_core::equations::registry::{Equation, EquationUsage};
//!
//! // Track equation usage during calculation
//! let usage = EquationUsage::new(Equation::UniformLoadMaxMoment, "Span 1");
//!
//! // Get metadata for PDF appendix
//! let meta = Equation::UniformLoadMaxMoment.metadata();
//! println!("Formula: {}", meta.formula_typst);
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Code References
// ============================================================================

/// Reference to a structural engineering code or standard.
///
/// All equations should cite their source for auditability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodeReference {
    /// Roark's Formulas for Stress and Strain
    Roarks {
        edition: u8,
        table: &'static str,
        case: &'static str,
    },
    /// National Design Specification for Wood Construction
    NDS {
        year: u16,
        section: &'static str,
    },
    /// AISC 360 - Specification for Structural Steel Buildings
    AISC360 {
        year: u16,
        chapter: &'static str,
    },
    /// ASCE 7 - Minimum Design Loads for Buildings
    ASCE7 {
        year: u16,
        section: &'static str,
    },
    /// ACI 318 - Building Code Requirements for Structural Concrete
    ACI318 {
        year: u16,
        section: &'static str,
    },
    /// Structural Analysis by R.C. Hibbeler
    Hibbeler {
        edition: u8,
        chapter: u8,
    },
    /// Fundamental mechanics (no specific code reference needed)
    Mechanics,
}

impl CodeReference {
    /// Format the reference for display in PDF reports
    pub fn citation(&self) -> String {
        match self {
            CodeReference::Roarks { edition, table, case } => {
                format!("Roark's {}ed, {}, Case {}", edition, table, case)
            }
            CodeReference::NDS { year, section } => {
                format!("NDS {} Section {}", year, section)
            }
            CodeReference::AISC360 { year, chapter } => {
                format!("AISC 360-{} Chapter {}", year % 100, chapter)
            }
            CodeReference::ASCE7 { year, section } => {
                format!("ASCE 7-{} Section {}", year % 100, section)
            }
            CodeReference::ACI318 { year, section } => {
                format!("ACI 318-{} Section {}", year % 100, section)
            }
            CodeReference::Hibbeler { edition, chapter } => {
                format!("Hibbeler {}ed, Ch. {}", edition, chapter)
            }
            CodeReference::Mechanics => "Fundamental Mechanics".to_string(),
        }
    }

    /// Short form for inline references
    pub fn short_form(&self) -> &'static str {
        match self {
            CodeReference::Roarks { .. } => "Roark's",
            CodeReference::NDS { .. } => "NDS",
            CodeReference::AISC360 { .. } => "AISC 360",
            CodeReference::ASCE7 { .. } => "ASCE 7",
            CodeReference::ACI318 { .. } => "ACI 318",
            CodeReference::Hibbeler { .. } => "Hibbeler",
            CodeReference::Mechanics => "Mechanics",
        }
    }
}

// ============================================================================
// Equation Categories
// ============================================================================

/// Categories for organizing equations in the PDF appendix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquationCategory {
    /// Support reactions (R1, R2)
    Reactions,
    /// Internal forces (moment, shear)
    InternalForces,
    /// Stress calculations (bending, shear, bearing)
    Stresses,
    /// Deflection calculations
    Deflections,
    /// Code-specific design checks (unity ratios)
    DesignChecks,
    /// Material adjustment factors (NDS C_D, C_M, etc.)
    AdjustmentFactors,
    /// Section properties (I, S, A, etc.)
    SectionProperties,
    /// Fixed-end moments for indeterminate analysis
    FixedEndMoments,
}

impl EquationCategory {
    /// Display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            EquationCategory::Reactions => "Reactions",
            EquationCategory::InternalForces => "Internal Forces",
            EquationCategory::Stresses => "Stresses",
            EquationCategory::Deflections => "Deflections",
            EquationCategory::DesignChecks => "Design Checks",
            EquationCategory::AdjustmentFactors => "Adjustment Factors",
            EquationCategory::SectionProperties => "Section Properties",
            EquationCategory::FixedEndMoments => "Fixed-End Moments",
        }
    }

    /// Sort order for PDF appendix (lower = earlier)
    pub fn sort_order(&self) -> u8 {
        match self {
            EquationCategory::SectionProperties => 1,
            EquationCategory::Reactions => 2,
            EquationCategory::InternalForces => 3,
            EquationCategory::Stresses => 4,
            EquationCategory::Deflections => 5,
            EquationCategory::FixedEndMoments => 6,
            EquationCategory::AdjustmentFactors => 7,
            EquationCategory::DesignChecks => 8,
        }
    }
}

// ============================================================================
// Variable Definition
// ============================================================================

/// Definition of a variable used in an equation.
#[derive(Debug, Clone)]
pub struct Variable {
    /// Symbol (e.g., "M", "L", "w")
    pub symbol: &'static str,
    /// Description
    pub description: &'static str,
    /// Units (e.g., "ft-lb", "in", "plf")
    pub units: &'static str,
}

impl Variable {
    pub const fn new(symbol: &'static str, description: &'static str, units: &'static str) -> Self {
        Self { symbol, description, units }
    }
}

// ============================================================================
// Equation Metadata
// ============================================================================

/// Complete metadata for a structural engineering equation.
///
/// This struct contains everything needed to:
/// - Display the equation in a PDF report
/// - Document its source for audit purposes
/// - Explain its variables and assumptions
#[derive(Debug, Clone)]
pub struct EquationMetadata {
    /// Human-readable name (e.g., "Maximum Moment for Uniform Load")
    pub name: &'static str,
    /// Brief description of what this equation calculates
    pub description: &'static str,
    /// The formula in Typst math notation for PDF rendering
    pub formula_typst: &'static str,
    /// Code/standard reference
    pub reference: CodeReference,
    /// Variable definitions (owned for flexibility)
    pub variables: Vec<Variable>,
    /// Assumptions or limitations
    pub assumptions: Vec<&'static str>,
    /// Category for grouping in appendix
    pub category: EquationCategory,
}

// ============================================================================
// Equation Enum
// ============================================================================

/// All structural engineering equations used in Stratify.
///
/// Each variant maps to a specific formula with full metadata.
/// This enum is the primary interface for equation tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Equation {
    // -------------------------------------------------------------------------
    // Simply-Supported Beam: Point Load
    // -------------------------------------------------------------------------
    /// R1 = P(L-a)/L, R2 = Pa/L
    PointLoadReactions,
    /// V(x) for point load
    PointLoadShear,
    /// M(x) for point load, M_max = Pa(L-a)/L
    PointLoadMoment,
    /// Deflection at any point for point load
    PointLoadDeflection,

    // -------------------------------------------------------------------------
    // Simply-Supported Beam: Uniform Load
    // -------------------------------------------------------------------------
    /// R1 = R2 = wL/2
    UniformLoadReactions,
    /// V(x) = w(L/2 - x)
    UniformLoadShear,
    /// M(x) = wx(L-x)/2
    UniformLoadMoment,
    /// M_max = wL^2/8
    UniformLoadMaxMoment,
    /// Deflection at any point
    UniformLoadDeflection,
    /// Max deflection = 5wL^4/(384EI)
    UniformLoadMaxDeflection,

    // -------------------------------------------------------------------------
    // Simply-Supported Beam: Partial Uniform Load
    // -------------------------------------------------------------------------
    /// Reactions for partial uniform load from a to b
    PartialUniformReactions,
    /// Moment for partial uniform load
    PartialUniformMoment,
    /// Shear for partial uniform load
    PartialUniformShear,

    // -------------------------------------------------------------------------
    // Fixed-End Moments (for Moment Distribution)
    // -------------------------------------------------------------------------
    /// FEM = wL^2/12 for uniform load
    FEMUniformFull,
    /// FEM for point load: -Pab^2/L^2, Pa^2b/L^2
    FEMPointLoad,
    /// FEM for partial uniform load (numerical)
    FEMPartialUniform,

    // -------------------------------------------------------------------------
    // Fixed-Fixed Beam
    // -------------------------------------------------------------------------
    /// End moments for fixed-fixed with uniform load
    FixedFixedUniformEndMoments,
    /// Max positive moment at midspan
    FixedFixedUniformMaxPositiveMoment,
    /// Max deflection = wL^4/(384EI)
    FixedFixedUniformMaxDeflection,

    // -------------------------------------------------------------------------
    // Cantilever Beam
    // -------------------------------------------------------------------------
    /// Reaction and fixed-end moment for uniform load
    CantileverUniformReactions,
    /// Max deflection = wL^4/(8EI)
    CantileverUniformMaxDeflection,
    /// Reaction and moment for point load
    CantileverPointReactions,

    // -------------------------------------------------------------------------
    // Propped Cantilever (Fixed-Pinned)
    // -------------------------------------------------------------------------
    /// R_A = 5wL/8, R_B = 3wL/8, M_A = wL^2/8
    FixedPinnedUniformReactions,
    /// Max positive moment = 9wL^2/128
    FixedPinnedUniformMaxPositiveMoment,

    // -------------------------------------------------------------------------
    // Section Properties
    // -------------------------------------------------------------------------
    /// A = bd for rectangular section
    RectangularArea,
    /// S = bd^2/6 for rectangular section
    RectangularSectionModulus,
    /// I = bd^3/12 for rectangular section
    RectangularMomentOfInertia,

    // -------------------------------------------------------------------------
    // Stress Calculations
    // -------------------------------------------------------------------------
    /// f_b = M/S bending stress
    BendingStress,
    /// f_v = 3V/(2bd) shear stress for rectangular section
    ShearStressRectangular,

    // -------------------------------------------------------------------------
    // NDS Wood Design
    // -------------------------------------------------------------------------
    /// F_b' = F_b * C_D * C_M * C_t * C_L * C_F * C_fu * C_i * C_r
    NDSAdjustedBendingStrength,
    /// F_v' = F_v * C_D * C_M * C_t * C_i
    NDSAdjustedShearStrength,
    /// E' = E * C_M * C_t * C_i
    NDSAdjustedModulusOfElasticity,
    /// Unity ratio: f_b / F_b' <= 1.0
    NDSBendingUnityRatio,
    /// Unity ratio: f_v / F_v' <= 1.0
    NDSShearUnityRatio,
    /// Deflection limit: L/240, L/360, etc.
    DeflectionLimit,
}

impl Equation {
    /// Get the full metadata for this equation
    pub fn metadata(&self) -> EquationMetadata {
        match self {
            // Simply-Supported: Point Load
            Equation::PointLoadReactions => EquationMetadata {
                name: "Point Load Reactions",
                description: "Support reactions for concentrated load at distance a from left support",
                formula_typst: r#"$R_1 = P(L - a) / L$, $R_2 = P a / L$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "1a" },
                variables: vec![
                    Variable::new("P", "Point load magnitude", "lb"),
                    Variable::new("a", "Distance from left support to load", "ft"),
                    Variable::new("L", "Span length", "ft"),
                    Variable::new("R_1", "Left reaction", "lb"),
                    Variable::new("R_2", "Right reaction", "lb"),
                ],
                assumptions: vec!["Simply-supported (pin-roller)", "Load is perpendicular to beam axis"],
                category: EquationCategory::Reactions,
            },

            Equation::PointLoadShear => EquationMetadata {
                name: "Point Load Shear",
                description: "Shear force at position x for concentrated load",
                formula_typst: r#"$V(x) = R_1$ for $x < a$, $V(x) = R_1 - P$ for $x >= a$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "1a" },
                variables: vec![
                    Variable::new("V", "Shear force", "lb"),
                    Variable::new("x", "Position along beam", "ft"),
                ],
                assumptions: vec!["Simply-supported", "Positive shear: left side up"],
                category: EquationCategory::InternalForces,
            },

            Equation::PointLoadMoment => EquationMetadata {
                name: "Point Load Moment",
                description: "Bending moment at position x for concentrated load",
                formula_typst: r#"$M(x) = R_1 x$ for $x <= a$, $M_"max" = P a (L - a) / L$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "1a" },
                variables: vec![
                    Variable::new("M", "Bending moment", "ft-lb"),
                    Variable::new("x", "Position along beam", "ft"),
                ],
                assumptions: vec!["Simply-supported", "Positive moment: tension on bottom"],
                category: EquationCategory::InternalForces,
            },

            Equation::PointLoadDeflection => EquationMetadata {
                name: "Point Load Deflection",
                description: "Deflection at position x for concentrated load",
                formula_typst: r#"$delta(x) = (P b x (L^2 - b^2 - x^2)) / (6 E I L)$ for $x <= a$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "1a" },
                variables: vec![
                    Variable::new("delta", "Deflection", "in"),
                    Variable::new("E", "Modulus of elasticity", "psi"),
                    Variable::new("I", "Moment of inertia", "in^4"),
                    Variable::new("b", "L - a", "ft"),
                ],
                assumptions: vec!["Linear elastic material", "Small deflections"],
                category: EquationCategory::Deflections,
            },

            // Simply-Supported: Uniform Load
            Equation::UniformLoadReactions => EquationMetadata {
                name: "Uniform Load Reactions",
                description: "Support reactions for uniformly distributed load over full span",
                formula_typst: r#"$R_1 = R_2 = w L / 2$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2a" },
                variables: vec![
                    Variable::new("w", "Uniform load intensity", "plf"),
                    Variable::new("L", "Span length", "ft"),
                ],
                assumptions: vec!["Simply-supported", "Symmetric loading"],
                category: EquationCategory::Reactions,
            },

            Equation::UniformLoadShear => EquationMetadata {
                name: "Uniform Load Shear",
                description: "Shear force at position x for uniform load",
                formula_typst: r#"$V(x) = w (L / 2 - x)$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2a" },
                variables: vec![
                    Variable::new("V", "Shear force", "lb"),
                    Variable::new("x", "Position along beam", "ft"),
                ],
                assumptions: vec!["Simply-supported", "Linear variation"],
                category: EquationCategory::InternalForces,
            },

            Equation::UniformLoadMoment => EquationMetadata {
                name: "Uniform Load Moment",
                description: "Bending moment at position x for uniform load",
                formula_typst: r#"$M(x) = w x (L - x) / 2$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2a" },
                variables: vec![
                    Variable::new("M", "Bending moment", "ft-lb"),
                    Variable::new("x", "Position along beam", "ft"),
                ],
                assumptions: vec!["Simply-supported", "Parabolic distribution"],
                category: EquationCategory::InternalForces,
            },

            Equation::UniformLoadMaxMoment => EquationMetadata {
                name: "Maximum Moment for Uniform Load",
                description: "Maximum bending moment at midspan for uniform load",
                formula_typst: r#"$M_"max" = w L^2 / 8$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2a" },
                variables: vec![
                    Variable::new("M_max", "Maximum moment", "ft-lb"),
                    Variable::new("w", "Uniform load", "plf"),
                    Variable::new("L", "Span length", "ft"),
                ],
                assumptions: vec!["Simply-supported", "Occurs at midspan"],
                category: EquationCategory::InternalForces,
            },

            Equation::UniformLoadDeflection => EquationMetadata {
                name: "Uniform Load Deflection",
                description: "Deflection at position x for uniform load",
                formula_typst: r#"$delta(x) = (w x (L^3 - 2 L x^2 + x^3)) / (24 E I)$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2a" },
                variables: vec![
                    Variable::new("delta", "Deflection", "in"),
                    Variable::new("E", "Modulus of elasticity", "psi"),
                    Variable::new("I", "Moment of inertia", "in^4"),
                ],
                assumptions: vec!["Linear elastic material", "Small deflections"],
                category: EquationCategory::Deflections,
            },

            Equation::UniformLoadMaxDeflection => EquationMetadata {
                name: "Maximum Deflection for Uniform Load",
                description: "Maximum deflection at midspan for uniform load",
                formula_typst: r#"$delta_"max" = (5 w L^4) / (384 E I)$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2a" },
                variables: vec![
                    Variable::new("delta_max", "Maximum deflection", "in"),
                    Variable::new("w", "Uniform load", "lb/in"),
                    Variable::new("L", "Span length", "in"),
                    Variable::new("E", "Modulus of elasticity", "psi"),
                    Variable::new("I", "Moment of inertia", "in^4"),
                ],
                assumptions: vec!["Linear elastic", "Occurs at midspan", "Small deflections"],
                category: EquationCategory::Deflections,
            },

            // Partial Uniform Load
            Equation::PartialUniformReactions => EquationMetadata {
                name: "Partial Uniform Load Reactions",
                description: "Reactions for uniform load from position a to b",
                formula_typst: r#"$R_1 = W (L - c) / L$, $R_2 = W c / L$ where $W = w (b - a)$, $c = (a + b) / 2$"#,
                reference: CodeReference::Mechanics,
                variables: vec![
                    Variable::new("W", "Total load", "lb"),
                    Variable::new("c", "Centroid position", "ft"),
                    Variable::new("a", "Load start position", "ft"),
                    Variable::new("b", "Load end position", "ft"),
                ],
                assumptions: vec!["Simply-supported", "Load treated as resultant at centroid for reactions"],
                category: EquationCategory::Reactions,
            },

            Equation::PartialUniformMoment => EquationMetadata {
                name: "Partial Uniform Load Moment",
                description: "Moment at position x for partial uniform load",
                formula_typst: r#"$M(x) = R_1 x - w (x - a)^2 / 2$ for $a < x < b$"#,
                reference: CodeReference::Mechanics,
                variables: vec![
                    Variable::new("M", "Bending moment", "ft-lb"),
                    Variable::new("x", "Position along beam", "ft"),
                ],
                assumptions: vec!["Simply-supported", "Superposition of uniform load segment"],
                category: EquationCategory::InternalForces,
            },

            Equation::PartialUniformShear => EquationMetadata {
                name: "Partial Uniform Load Shear",
                description: "Shear at position x for partial uniform load",
                formula_typst: r#"$V(x) = R_1 - w (x - a)$ for $a < x < b$"#,
                reference: CodeReference::Mechanics,
                variables: vec![
                    Variable::new("V", "Shear force", "lb"),
                    Variable::new("x", "Position along beam", "ft"),
                ],
                assumptions: vec!["Simply-supported"],
                category: EquationCategory::InternalForces,
            },

            // Fixed-End Moments
            Equation::FEMUniformFull => EquationMetadata {
                name: "FEM for Uniform Load",
                description: "Fixed-end moments for uniform load over entire span",
                formula_typst: r#"$"FEM"_A = -w L^2 / 12$, $"FEM"_B = +w L^2 / 12$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2e" },
                variables: vec![
                    Variable::new("FEM_A", "Fixed-end moment at A", "ft-lb"),
                    Variable::new("FEM_B", "Fixed-end moment at B", "ft-lb"),
                ],
                assumptions: vec!["Fully fixed ends", "Used in moment distribution method"],
                category: EquationCategory::FixedEndMoments,
            },

            Equation::FEMPointLoad => EquationMetadata {
                name: "FEM for Point Load",
                description: "Fixed-end moments for point load at distance a",
                formula_typst: r#"$"FEM"_A = -P a b^2 / L^2$, $"FEM"_B = +P a^2 b / L^2$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "1e" },
                variables: vec![
                    Variable::new("FEM_A", "Fixed-end moment at A", "ft-lb"),
                    Variable::new("FEM_B", "Fixed-end moment at B", "ft-lb"),
                    Variable::new("b", "L - a", "ft"),
                ],
                assumptions: vec!["Fully fixed ends", "Used in moment distribution method"],
                category: EquationCategory::FixedEndMoments,
            },

            Equation::FEMPartialUniform => EquationMetadata {
                name: "FEM for Partial Uniform Load",
                description: "Fixed-end moments for partial uniform load (numerical integration)",
                formula_typst: r#"$"FEM" = sum P_i "FEM"_i$ (discrete approximation)"#,
                reference: CodeReference::Mechanics,
                variables: vec![
                    Variable::new("FEM", "Fixed-end moment", "ft-lb"),
                ],
                assumptions: vec!["Numerical integration of point load FEMs", "20 segments"],
                category: EquationCategory::FixedEndMoments,
            },

            // Fixed-Fixed Beam
            Equation::FixedFixedUniformEndMoments => EquationMetadata {
                name: "Fixed-Fixed End Moments",
                description: "End moments for beam fixed at both ends with uniform load",
                formula_typst: r#"$M_A = M_B = w L^2 / 12$ (hogging)"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2e" },
                variables: vec![
                    Variable::new("M_A", "Moment at left support", "ft-lb"),
                    Variable::new("M_B", "Moment at right support", "ft-lb"),
                ],
                assumptions: vec!["Both ends fully fixed", "Symmetric loading"],
                category: EquationCategory::InternalForces,
            },

            Equation::FixedFixedUniformMaxPositiveMoment => EquationMetadata {
                name: "Fixed-Fixed Max Positive Moment",
                description: "Maximum positive moment at midspan for fixed-fixed beam",
                formula_typst: r#"$M_"max" = w L^2 / 24$ (sagging at midspan)"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2e" },
                variables: vec![
                    Variable::new("M_max", "Maximum positive moment", "ft-lb"),
                ],
                assumptions: vec!["Both ends fully fixed", "Occurs at midspan"],
                category: EquationCategory::InternalForces,
            },

            Equation::FixedFixedUniformMaxDeflection => EquationMetadata {
                name: "Fixed-Fixed Max Deflection",
                description: "Maximum deflection at midspan for fixed-fixed beam with uniform load",
                formula_typst: r#"$delta_"max" = w L^4 / (384 E I)$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2e" },
                variables: vec![
                    Variable::new("delta_max", "Maximum deflection", "in"),
                ],
                assumptions: vec!["Both ends fully fixed", "1/5 of simply-supported deflection"],
                category: EquationCategory::Deflections,
            },

            // Cantilever
            Equation::CantileverUniformReactions => EquationMetadata {
                name: "Cantilever Uniform Load Reactions",
                description: "Reaction and fixed-end moment for cantilever with uniform load",
                formula_typst: r#"$R = w L$, $M = w L^2 / 2$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2b" },
                variables: vec![
                    Variable::new("R", "Support reaction", "lb"),
                    Variable::new("M", "Fixed-end moment", "ft-lb"),
                ],
                assumptions: vec!["Fixed at one end, free at other"],
                category: EquationCategory::Reactions,
            },

            Equation::CantileverUniformMaxDeflection => EquationMetadata {
                name: "Cantilever Uniform Load Max Deflection",
                description: "Maximum deflection at free end for cantilever with uniform load",
                formula_typst: r#"$delta_"max" = w L^4 / (8 E I)$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2b" },
                variables: vec![
                    Variable::new("delta_max", "Maximum deflection at free end", "in"),
                ],
                assumptions: vec!["Fixed at one end", "Deflection at free end"],
                category: EquationCategory::Deflections,
            },

            Equation::CantileverPointReactions => EquationMetadata {
                name: "Cantilever Point Load Reactions",
                description: "Reaction and fixed-end moment for cantilever with point load",
                formula_typst: r#"$R = P$, $M = P a$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "1b" },
                variables: vec![
                    Variable::new("R", "Support reaction", "lb"),
                    Variable::new("M", "Fixed-end moment", "ft-lb"),
                    Variable::new("a", "Distance from support to load", "ft"),
                ],
                assumptions: vec!["Fixed at one end, free at other"],
                category: EquationCategory::Reactions,
            },

            // Propped Cantilever
            Equation::FixedPinnedUniformReactions => EquationMetadata {
                name: "Propped Cantilever Reactions",
                description: "Reactions for beam fixed at left, pinned at right, with uniform load",
                formula_typst: r#"$R_A = 5 w L / 8$, $R_B = 3 w L / 8$, $M_A = w L^2 / 8$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2c" },
                variables: vec![
                    Variable::new("R_A", "Reaction at fixed end", "lb"),
                    Variable::new("R_B", "Reaction at pinned end", "lb"),
                    Variable::new("M_A", "Moment at fixed end", "ft-lb"),
                ],
                assumptions: vec!["Fixed-pinned supports", "Asymmetric reactions"],
                category: EquationCategory::Reactions,
            },

            Equation::FixedPinnedUniformMaxPositiveMoment => EquationMetadata {
                name: "Propped Cantilever Max Positive Moment",
                description: "Maximum positive moment for propped cantilever with uniform load",
                formula_typst: r#"$M_"max" = 9 w L^2 / 128$ at $x = 3L / 8$"#,
                reference: CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "2c" },
                variables: vec![
                    Variable::new("M_max", "Maximum positive moment", "ft-lb"),
                ],
                assumptions: vec!["Fixed-pinned supports", "Occurs at 3L/8 from fixed end"],
                category: EquationCategory::InternalForces,
            },

            // Section Properties
            Equation::RectangularArea => EquationMetadata {
                name: "Rectangular Area",
                description: "Cross-sectional area of rectangular section",
                formula_typst: r#"$A = b d$"#,
                reference: CodeReference::Mechanics,
                variables: vec![
                    Variable::new("A", "Cross-sectional area", "in^2"),
                    Variable::new("b", "Width", "in"),
                    Variable::new("d", "Depth", "in"),
                ],
                assumptions: vec!["Solid rectangular section"],
                category: EquationCategory::SectionProperties,
            },

            Equation::RectangularSectionModulus => EquationMetadata {
                name: "Rectangular Section Modulus",
                description: "Elastic section modulus of rectangular section",
                formula_typst: r#"$S = b d^2 / 6$"#,
                reference: CodeReference::Mechanics,
                variables: vec![
                    Variable::new("S", "Section modulus", "in^3"),
                    Variable::new("b", "Width", "in"),
                    Variable::new("d", "Depth", "in"),
                ],
                assumptions: vec!["Solid rectangular section", "Bending about strong axis"],
                category: EquationCategory::SectionProperties,
            },

            Equation::RectangularMomentOfInertia => EquationMetadata {
                name: "Rectangular Moment of Inertia",
                description: "Moment of inertia of rectangular section about centroidal axis",
                formula_typst: r#"$I = b d^3 / 12$"#,
                reference: CodeReference::Mechanics,
                variables: vec![
                    Variable::new("I", "Moment of inertia", "in^4"),
                    Variable::new("b", "Width", "in"),
                    Variable::new("d", "Depth", "in"),
                ],
                assumptions: vec!["Solid rectangular section", "About centroidal axis"],
                category: EquationCategory::SectionProperties,
            },

            // Stress Calculations
            Equation::BendingStress => EquationMetadata {
                name: "Bending Stress",
                description: "Maximum bending stress at extreme fiber",
                formula_typst: r#"$f_b = M / S$"#,
                reference: CodeReference::Mechanics,
                variables: vec![
                    Variable::new("f_b", "Bending stress", "psi"),
                    Variable::new("M", "Bending moment", "in-lb"),
                    Variable::new("S", "Section modulus", "in^3"),
                ],
                assumptions: vec!["Linear elastic material", "Plane sections remain plane"],
                category: EquationCategory::Stresses,
            },

            Equation::ShearStressRectangular => EquationMetadata {
                name: "Shear Stress (Rectangular)",
                description: "Maximum shear stress in rectangular section",
                formula_typst: r#"$f_v = 3 V / (2 b d)$"#,
                reference: CodeReference::NDS { year: 2018, section: "3.4.2" },
                variables: vec![
                    Variable::new("f_v", "Shear stress", "psi"),
                    Variable::new("V", "Shear force", "lb"),
                    Variable::new("b", "Width", "in"),
                    Variable::new("d", "Depth", "in"),
                ],
                assumptions: vec!["Rectangular section", "Parabolic shear distribution", "Max at neutral axis"],
                category: EquationCategory::Stresses,
            },

            // NDS Wood Design
            Equation::NDSAdjustedBendingStrength => EquationMetadata {
                name: "NDS Adjusted Bending Strength",
                description: "Reference bending design value multiplied by all adjustment factors",
                formula_typst: r#"$F'_b = F_b C_D C_M C_t C_L C_F C_"fu" C_i C_r$"#,
                reference: CodeReference::NDS { year: 2018, section: "4.3" },
                variables: vec![
                    Variable::new("F'_b", "Adjusted bending design value", "psi"),
                    Variable::new("F_b", "Reference bending design value", "psi"),
                    Variable::new("C_D", "Load duration factor", "-"),
                    Variable::new("C_M", "Wet service factor", "-"),
                    Variable::new("C_t", "Temperature factor", "-"),
                    Variable::new("C_L", "Beam stability factor", "-"),
                    Variable::new("C_F", "Size factor", "-"),
                    Variable::new("C_fu", "Flat use factor", "-"),
                    Variable::new("C_i", "Incising factor", "-"),
                    Variable::new("C_r", "Repetitive member factor", "-"),
                ],
                assumptions: vec!["Sawn lumber per NDS 2018", "ASD method"],
                category: EquationCategory::AdjustmentFactors,
            },

            Equation::NDSAdjustedShearStrength => EquationMetadata {
                name: "NDS Adjusted Shear Strength",
                description: "Reference shear design value multiplied by applicable adjustment factors",
                formula_typst: r#"$F'_v = F_v C_D C_M C_t C_i$"#,
                reference: CodeReference::NDS { year: 2018, section: "4.3" },
                variables: vec![
                    Variable::new("F'_v", "Adjusted shear design value", "psi"),
                    Variable::new("F_v", "Reference shear design value", "psi"),
                ],
                assumptions: vec!["Sawn lumber per NDS 2018", "ASD method"],
                category: EquationCategory::AdjustmentFactors,
            },

            Equation::NDSAdjustedModulusOfElasticity => EquationMetadata {
                name: "NDS Adjusted Modulus of Elasticity",
                description: "Reference modulus adjusted for service conditions",
                formula_typst: r#"$E' = E C_M C_t C_i$"#,
                reference: CodeReference::NDS { year: 2018, section: "4.3" },
                variables: vec![
                    Variable::new("E'", "Adjusted modulus of elasticity", "psi"),
                    Variable::new("E", "Reference modulus of elasticity", "psi"),
                ],
                assumptions: vec!["Sawn lumber per NDS 2018"],
                category: EquationCategory::AdjustmentFactors,
            },

            Equation::NDSBendingUnityRatio => EquationMetadata {
                name: "NDS Bending Unity Ratio",
                description: "Demand/capacity ratio for bending stress check",
                formula_typst: r#"$f_b / F'_b <= 1.0$"#,
                reference: CodeReference::NDS { year: 2018, section: "3.3" },
                variables: vec![
                    Variable::new("f_b", "Actual bending stress", "psi"),
                    Variable::new("F'_b", "Adjusted allowable bending stress", "psi"),
                ],
                assumptions: vec!["Unity ratio <= 1.0 indicates adequate capacity"],
                category: EquationCategory::DesignChecks,
            },

            Equation::NDSShearUnityRatio => EquationMetadata {
                name: "NDS Shear Unity Ratio",
                description: "Demand/capacity ratio for shear stress check",
                formula_typst: r#"$f_v / F'_v <= 1.0$"#,
                reference: CodeReference::NDS { year: 2018, section: "3.4" },
                variables: vec![
                    Variable::new("f_v", "Actual shear stress", "psi"),
                    Variable::new("F'_v", "Adjusted allowable shear stress", "psi"),
                ],
                assumptions: vec!["Unity ratio <= 1.0 indicates adequate capacity"],
                category: EquationCategory::DesignChecks,
            },

            Equation::DeflectionLimit => EquationMetadata {
                name: "Deflection Limit",
                description: "Serviceability check for maximum deflection",
                formula_typst: r#"$delta <= L / "limit"$ (typical: L/240 live, L/180 total)"#,
                reference: CodeReference::ASCE7 { year: 2022, section: "Table 1604.3" },
                variables: vec![
                    Variable::new("delta", "Actual deflection", "in"),
                    Variable::new("L", "Span length", "in"),
                    Variable::new("limit", "Deflection ratio (240, 360, etc.)", "-"),
                ],
                assumptions: vec!["IBC Table 1604.3 limits", "Floor/roof specific limits"],
                category: EquationCategory::DesignChecks,
            },
        }
    }

    /// Get all equations in a given category
    pub fn in_category(category: EquationCategory) -> Vec<Equation> {
        ALL_EQUATIONS
            .iter()
            .filter(|eq| eq.metadata().category == category)
            .copied()
            .collect()
    }

    /// Get all categories that contain at least one equation
    pub fn all_categories() -> Vec<EquationCategory> {
        use EquationCategory::*;
        let mut cats = vec![
            SectionProperties,
            Reactions,
            InternalForces,
            Stresses,
            Deflections,
            FixedEndMoments,
            AdjustmentFactors,
            DesignChecks,
        ];
        cats.sort_by_key(|c| c.sort_order());
        cats
    }
}

/// All equations in the registry (for iteration)
pub static ALL_EQUATIONS: &[Equation] = &[
    // Simply-supported point load
    Equation::PointLoadReactions,
    Equation::PointLoadShear,
    Equation::PointLoadMoment,
    Equation::PointLoadDeflection,
    // Simply-supported uniform load
    Equation::UniformLoadReactions,
    Equation::UniformLoadShear,
    Equation::UniformLoadMoment,
    Equation::UniformLoadMaxMoment,
    Equation::UniformLoadDeflection,
    Equation::UniformLoadMaxDeflection,
    // Partial uniform load
    Equation::PartialUniformReactions,
    Equation::PartialUniformMoment,
    Equation::PartialUniformShear,
    // Fixed-end moments
    Equation::FEMUniformFull,
    Equation::FEMPointLoad,
    Equation::FEMPartialUniform,
    // Fixed-fixed
    Equation::FixedFixedUniformEndMoments,
    Equation::FixedFixedUniformMaxPositiveMoment,
    Equation::FixedFixedUniformMaxDeflection,
    // Cantilever
    Equation::CantileverUniformReactions,
    Equation::CantileverUniformMaxDeflection,
    Equation::CantileverPointReactions,
    // Propped cantilever
    Equation::FixedPinnedUniformReactions,
    Equation::FixedPinnedUniformMaxPositiveMoment,
    // Section properties
    Equation::RectangularArea,
    Equation::RectangularSectionModulus,
    Equation::RectangularMomentOfInertia,
    // Stresses
    Equation::BendingStress,
    Equation::ShearStressRectangular,
    // NDS
    Equation::NDSAdjustedBendingStrength,
    Equation::NDSAdjustedShearStrength,
    Equation::NDSAdjustedModulusOfElasticity,
    Equation::NDSBendingUnityRatio,
    Equation::NDSShearUnityRatio,
    Equation::DeflectionLimit,
];

// ============================================================================
// Equation Usage Tracking
// ============================================================================

/// Record of an equation being used in a calculation.
///
/// This struct is used to track which equations were applied during a
/// calculation, enabling the "List of Equations" PDF appendix feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquationUsage {
    /// The equation that was used
    pub equation: Equation,
    /// Context describing where/why it was used (e.g., "Span 1 left support")
    pub context: String,
    /// Optional: the member label this equation was applied to
    pub member_label: Option<String>,
}

impl EquationUsage {
    /// Create a new equation usage record
    pub fn new(equation: Equation, context: impl Into<String>) -> Self {
        Self {
            equation,
            context: context.into(),
            member_label: None,
        }
    }

    /// Create usage record with member label
    pub fn for_member(equation: Equation, context: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            equation,
            context: context.into(),
            member_label: Some(label.into()),
        }
    }
}

/// Collector for equation usage during a calculation.
///
/// Pass this to calculation functions to track which equations are used.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquationTracker {
    usages: Vec<EquationUsage>,
}

impl EquationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that an equation was used
    pub fn record(&mut self, equation: Equation, context: impl Into<String>) {
        self.usages.push(EquationUsage::new(equation, context));
    }

    /// Record equation usage for a specific member
    pub fn record_for_member(&mut self, equation: Equation, context: impl Into<String>, label: impl Into<String>) {
        self.usages.push(EquationUsage::for_member(equation, context, label));
    }

    /// Get all recorded usages
    pub fn usages(&self) -> &[EquationUsage] {
        &self.usages
    }

    /// Get unique equations used (deduplicated)
    pub fn unique_equations(&self) -> Vec<Equation> {
        let mut seen = std::collections::HashSet::new();
        self.usages
            .iter()
            .filter(|u| seen.insert(u.equation))
            .map(|u| u.equation)
            .collect()
    }

    /// Group usages by equation for appendix generation
    pub fn by_equation(&self) -> std::collections::HashMap<Equation, Vec<&EquationUsage>> {
        let mut map: std::collections::HashMap<Equation, Vec<&EquationUsage>> = std::collections::HashMap::new();
        for usage in &self.usages {
            map.entry(usage.equation).or_default().push(usage);
        }
        map
    }

    /// Group unique equations by category for appendix
    pub fn by_category(&self) -> Vec<(EquationCategory, Vec<Equation>)> {
        let unique = self.unique_equations();
        let mut by_cat: std::collections::HashMap<EquationCategory, Vec<Equation>> = std::collections::HashMap::new();

        for eq in unique {
            let cat = eq.metadata().category;
            by_cat.entry(cat).or_default().push(eq);
        }

        let mut result: Vec<_> = by_cat.into_iter().collect();
        result.sort_by_key(|(cat, _)| cat.sort_order());
        result
    }

    /// Merge another tracker into this one
    pub fn merge(&mut self, other: EquationTracker) {
        self.usages.extend(other.usages);
    }
}

// ============================================================================
// Typst Appendix Generation
// ============================================================================

impl EquationTracker {
    /// Generate Typst markup for the "List of Equations" appendix.
    ///
    /// The appendix is organized by category (Section Properties, Reactions, etc.)
    /// and shows each unique equation with its formula, reference, and usage.
    ///
    /// # Example
    ///
    /// ```rust
    /// use calc_core::equations::registry::{Equation, EquationTracker};
    ///
    /// let mut tracker = EquationTracker::new();
    /// tracker.record_for_member(Equation::UniformLoadMaxMoment, "Max moment calculation", "B-1");
    /// tracker.record_for_member(Equation::BendingStress, "Stress check", "B-1");
    ///
    /// let typst = tracker.generate_appendix_typst();
    /// assert!(typst.contains("Maximum Moment for Uniform Load"));
    /// ```
    pub fn generate_appendix_typst(&self) -> String {
        let mut output = String::new();

        // Appendix header
        output.push_str(r##"
#pagebreak()

#align(center)[
  #block(width: 100%, fill: rgb("#f0f0f0"), inset: 12pt, radius: 4pt)[
    #text(size: 18pt, weight: "bold")[Appendix: List of Equations]
  ]
]

#v(12pt)

#text(size: 10pt)[
  This appendix lists all structural engineering equations used in this calculation package.
  Each equation includes its formula, code reference, and the members to which it was applied.
]

#v(16pt)
"##);

        // Get equations grouped by category
        let by_category = self.by_category();

        if by_category.is_empty() {
            output.push_str("#text(style: \"italic\")[No equations recorded for this project.]\n");
            return output;
        }

        // Get usage map for member references
        let usage_by_eq = self.by_equation();

        // Process each category
        for (category, equations) in by_category {
            // Category header
            output.push_str(&format!(
                "\n== {}\n\n",
                category.display_name()
            ));

            // Each equation in this category
            for equation in equations {
                let meta = equation.metadata();

                // Equation name and description
                output.push_str(&format!(
                    "=== {}\n\n",
                    meta.name
                ));
                output.push_str(&format!(
                    "#text(size: 10pt)[{}]\n\n",
                    meta.description
                ));

                // Formula (using Typst math notation)
                output.push_str(&format!(
                    "*Formula:* {}\n\n",
                    meta.formula_typst
                ));

                // Reference
                output.push_str(&format!(
                    "*Reference:* {}\n\n",
                    meta.reference.citation()
                ));

                // Variables table (if any)
                if !meta.variables.is_empty() {
                    output.push_str("*Variables:*\n");
                    output.push_str("#table(\n");
                    output.push_str("  columns: (auto, 1fr, auto),\n");
                    output.push_str("  inset: 6pt,\n");
                    output.push_str("  stroke: 0.5pt,\n");
                    output.push_str("  align: (left, left, left),\n");
                    output.push_str("  table.header([*Symbol*], [*Description*], [*Units*]),\n");

                    for var in &meta.variables {
                        output.push_str(&format!(
                            "  [${}$], [{}], [{}],\n",
                            escape_typst_math(var.symbol),
                            var.description,
                            var.units
                        ));
                    }
                    output.push_str(")\n\n");
                }

                // Members using this equation
                if let Some(usages) = usage_by_eq.get(&equation) {
                    let member_labels: Vec<&str> = usages
                        .iter()
                        .filter_map(|u| u.member_label.as_deref())
                        .collect();

                    if !member_labels.is_empty() {
                        // Deduplicate member labels
                        let mut unique: Vec<&str> = member_labels.clone();
                        unique.sort();
                        unique.dedup();

                        output.push_str(&format!(
                            "*Applied to:* {}\n\n",
                            unique.join(", ")
                        ));
                    }
                }

                // Assumptions (if any)
                if !meta.assumptions.is_empty() {
                    output.push_str("*Assumptions:*\n");
                    for assumption in &meta.assumptions {
                        output.push_str(&format!("- {}\n", assumption));
                    }
                    output.push_str("\n");
                }

                output.push_str("#v(8pt)\n");
                output.push_str("#line(length: 100%, stroke: 0.25pt + gray)\n");
                output.push_str("#v(8pt)\n\n");
            }
        }

        output
    }
}

/// Generate a "List of Equations" appendix for a set of equations.
///
/// This is a convenience function for when you want to list specific equations
/// without a full tracker. Useful for generating reference documentation.
///
/// # Arguments
///
/// * `equations` - The equations to include in the appendix
///
/// # Returns
///
/// Typst markup string for the appendix
pub fn generate_static_equations_appendix_typst(equations: &[Equation]) -> String {
    let mut tracker = EquationTracker::new();
    for &eq in equations {
        tracker.record(eq, "Reference");
    }
    tracker.generate_appendix_typst()
}

/// Get the default set of equations used in beam calculations.
///
/// This returns all equations that are typically applied when analyzing
/// simply-supported beam members with uniform and/or point loads.
pub fn beam_calculation_equations() -> Vec<Equation> {
    vec![
        // Section properties
        Equation::RectangularArea,
        Equation::RectangularSectionModulus,
        Equation::RectangularMomentOfInertia,
        // Reactions and internal forces
        Equation::UniformLoadReactions,
        Equation::UniformLoadMaxMoment,
        Equation::UniformLoadShear,
        Equation::UniformLoadMaxDeflection,
        Equation::PointLoadReactions,
        Equation::PointLoadMoment,
        // Stresses
        Equation::BendingStress,
        Equation::ShearStressRectangular,
        // NDS design checks
        Equation::NDSAdjustedBendingStrength,
        Equation::NDSAdjustedShearStrength,
        Equation::NDSAdjustedModulusOfElasticity,
        Equation::NDSBendingUnityRatio,
        Equation::NDSShearUnityRatio,
        Equation::DeflectionLimit,
    ]
}

/// Escape special characters for Typst math mode
fn escape_typst_math(s: &str) -> String {
    // In Typst math mode, underscores create subscripts which is usually what we want
    // Just ensure we don't have any problematic characters
    s.replace('\\', "\\\\")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_equations_have_metadata() {
        for eq in ALL_EQUATIONS {
            let meta = eq.metadata();
            assert!(!meta.name.is_empty(), "Equation {:?} has no name", eq);
            assert!(!meta.formula_typst.is_empty(), "Equation {:?} has no formula", eq);
        }
    }

    #[test]
    fn test_code_reference_citation() {
        let roark = CodeReference::Roarks { edition: 8, table: "Table 8.1", case: "1a" };
        assert_eq!(roark.citation(), "Roark's 8ed, Table 8.1, Case 1a");

        let nds = CodeReference::NDS { year: 2018, section: "4.3" };
        assert_eq!(nds.citation(), "NDS 2018 Section 4.3");
    }

    #[test]
    fn test_equation_tracker() {
        let mut tracker = EquationTracker::new();
        tracker.record(Equation::UniformLoadMaxMoment, "Span 1");
        tracker.record(Equation::UniformLoadMaxDeflection, "Span 1");
        tracker.record(Equation::UniformLoadMaxMoment, "Span 2");

        assert_eq!(tracker.usages().len(), 3);
        assert_eq!(tracker.unique_equations().len(), 2);
    }

    #[test]
    fn test_by_category() {
        let mut tracker = EquationTracker::new();
        tracker.record(Equation::UniformLoadMaxMoment, "test");
        tracker.record(Equation::UniformLoadMaxDeflection, "test");
        tracker.record(Equation::BendingStress, "test");

        let by_cat = tracker.by_category();
        assert!(by_cat.len() >= 2); // At least InternalForces, Deflections, Stresses
    }

    #[test]
    fn test_categories_sorted() {
        let cats = Equation::all_categories();
        let orders: Vec<u8> = cats.iter().map(|c| c.sort_order()).collect();
        let mut sorted = orders.clone();
        sorted.sort();
        assert_eq!(orders, sorted, "Categories should be sorted by sort_order");
    }

    #[test]
    fn test_generate_appendix_typst() {
        let mut tracker = EquationTracker::new();
        tracker.record_for_member(Equation::UniformLoadMaxMoment, "Max moment", "B-1");
        tracker.record_for_member(Equation::BendingStress, "Stress check", "B-1");
        tracker.record_for_member(Equation::RectangularSectionModulus, "Section props", "B-1");

        let typst = tracker.generate_appendix_typst();

        // Should contain appendix header
        assert!(typst.contains("Appendix: List of Equations"), "Missing appendix header");

        // Should contain equation names
        assert!(typst.contains("Maximum Moment for Uniform Load"), "Missing uniform load moment");
        assert!(typst.contains("Bending Stress"), "Missing bending stress");
        assert!(typst.contains("Rectangular Section Modulus"), "Missing section modulus");

        // Should contain references
        assert!(typst.contains("Roark's"), "Missing Roark's reference");

        // Should contain member label
        assert!(typst.contains("B-1"), "Missing member label");

        // Should be organized by category
        assert!(typst.contains("Section Properties"), "Missing section properties category");
        assert!(typst.contains("Internal Forces"), "Missing internal forces category");
        assert!(typst.contains("Stresses"), "Missing stresses category");
    }

    #[test]
    fn test_generate_appendix_empty_tracker() {
        let tracker = EquationTracker::new();
        let typst = tracker.generate_appendix_typst();

        assert!(typst.contains("Appendix: List of Equations"));
        assert!(typst.contains("No equations recorded"));
    }

    #[test]
    fn test_beam_calculation_equations() {
        let equations = beam_calculation_equations();

        // Should include key equations
        assert!(equations.contains(&Equation::UniformLoadMaxMoment));
        assert!(equations.contains(&Equation::BendingStress));
        assert!(equations.contains(&Equation::NDSBendingUnityRatio));
        assert!(equations.contains(&Equation::RectangularSectionModulus));

        // Should have reasonable count
        assert!(equations.len() >= 10, "Expected at least 10 beam equations");
    }

    #[test]
    fn test_static_equations_appendix() {
        let equations = vec![
            Equation::UniformLoadMaxMoment,
            Equation::BendingStress,
        ];

        let typst = generate_static_equations_appendix_typst(&equations);

        assert!(typst.contains("Maximum Moment for Uniform Load"));
        assert!(typst.contains("Bending Stress"));
    }
}
