//! # Unit Types
//!
//! Type-safe wrappers for engineering units. These provide compile-time
//! safety against unit confusion while remaining lightweight (just f64 wrappers).
//!
//! ## Design Philosophy
//!
//! We use simple newtype wrappers rather than a full units library because:
//! - Structural engineering uses a consistent set of units
//! - We want JSON serialization to be clean (just numbers)
//! - Minimal runtime overhead
//!
//! ## US Customary Units (Primary)
//!
//! Stratify uses US customary units internally as this matches US building codes:
//! - Length: feet (ft), inches (in)
//! - Force: pounds (lb), kips (k = 1000 lb)
//! - Stress: pounds per square inch (psi), kips per square inch (ksi)
//! - Moment: foot-pounds (ft-lb), kip-feet (k-ft), inch-pounds (in-lb), kip-inches (k-in)
//! - Distributed load: pounds per linear foot (plf), kips per linear foot (klf)
//!
//! ## Example
//!
//! ```rust
//! use calc_core::units::{Feet, Inches, Kips, PlF};
//!
//! let span = Feet(12.0);
//! let span_inches: Inches = span.into();
//! assert_eq!(span_inches.0, 144.0);
//!
//! let load = PlF(150.0); // 150 pounds per linear foot
//! ```

use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

// ============================================================================
// Length Units
// ============================================================================

/// Length in feet
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Feet(pub f64);

/// Length in inches
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Inches(pub f64);

impl From<Feet> for Inches {
    fn from(ft: Feet) -> Self {
        Inches(ft.0 * 12.0)
    }
}

impl From<Inches> for Feet {
    fn from(inches: Inches) -> Self {
        Feet(inches.0 / 12.0)
    }
}

// ============================================================================
// Force Units
// ============================================================================

/// Force in pounds
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pounds(pub f64);

/// Force in kips (1 kip = 1000 pounds)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Kips(pub f64);

impl From<Pounds> for Kips {
    fn from(lb: Pounds) -> Self {
        Kips(lb.0 / 1000.0)
    }
}

impl From<Kips> for Pounds {
    fn from(k: Kips) -> Self {
        Pounds(k.0 * 1000.0)
    }
}

// ============================================================================
// Stress Units
// ============================================================================

/// Stress in pounds per square inch (psi)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Psi(pub f64);

/// Stress in kips per square inch (ksi)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Ksi(pub f64);

impl From<Psi> for Ksi {
    fn from(psi: Psi) -> Self {
        Ksi(psi.0 / 1000.0)
    }
}

impl From<Ksi> for Psi {
    fn from(ksi: Ksi) -> Self {
        Psi(ksi.0 * 1000.0)
    }
}

// ============================================================================
// Moment Units
// ============================================================================

/// Moment in foot-pounds
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FtLb(pub f64);

/// Moment in kip-feet
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KipFt(pub f64);

/// Moment in inch-pounds
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InLb(pub f64);

/// Moment in kip-inches
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KipIn(pub f64);

impl From<FtLb> for InLb {
    fn from(ftlb: FtLb) -> Self {
        InLb(ftlb.0 * 12.0)
    }
}

impl From<InLb> for FtLb {
    fn from(inlb: InLb) -> Self {
        FtLb(inlb.0 / 12.0)
    }
}

impl From<KipFt> for KipIn {
    fn from(kipft: KipFt) -> Self {
        KipIn(kipft.0 * 12.0)
    }
}

impl From<KipIn> for KipFt {
    fn from(kipin: KipIn) -> Self {
        KipFt(kipin.0 / 12.0)
    }
}

impl From<FtLb> for KipFt {
    fn from(ftlb: FtLb) -> Self {
        KipFt(ftlb.0 / 1000.0)
    }
}

impl From<KipFt> for FtLb {
    fn from(kipft: KipFt) -> Self {
        FtLb(kipft.0 * 1000.0)
    }
}

// ============================================================================
// Distributed Load Units
// ============================================================================

/// Distributed load in pounds per linear foot (plf)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlF(pub f64);

/// Distributed load in kips per linear foot (klf)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KlF(pub f64);

impl From<PlF> for KlF {
    fn from(plf: PlF) -> Self {
        KlF(plf.0 / 1000.0)
    }
}

impl From<KlF> for PlF {
    fn from(klf: KlF) -> Self {
        PlF(klf.0 * 1000.0)
    }
}

// ============================================================================
// Area Units
// ============================================================================

/// Area in square inches
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SqIn(pub f64);

/// Area in square feet
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SqFt(pub f64);

impl From<SqFt> for SqIn {
    fn from(sqft: SqFt) -> Self {
        SqIn(sqft.0 * 144.0)
    }
}

impl From<SqIn> for SqFt {
    fn from(sqin: SqIn) -> Self {
        SqFt(sqin.0 / 144.0)
    }
}

// ============================================================================
// Section Properties
// ============================================================================

/// Moment of inertia in inches^4
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct In4(pub f64);

/// Section modulus in inches^3
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct In3(pub f64);

// ============================================================================
// Arithmetic Implementations (macro to reduce boilerplate)
// ============================================================================

macro_rules! impl_arithmetic {
    ($type:ty) => {
        impl Add for $type {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl Sub for $type {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl Mul<f64> for $type {
            type Output = Self;
            fn mul(self, rhs: f64) -> Self::Output {
                Self(self.0 * rhs)
            }
        }

        impl Div<f64> for $type {
            type Output = Self;
            fn div(self, rhs: f64) -> Self::Output {
                Self(self.0 / rhs)
            }
        }

        impl $type {
            /// Get the raw f64 value
            pub fn value(self) -> f64 {
                self.0
            }

            /// Create from raw f64 value
            pub fn new(value: f64) -> Self {
                Self(value)
            }
        }
    };
}

impl_arithmetic!(Feet);
impl_arithmetic!(Inches);
impl_arithmetic!(Pounds);
impl_arithmetic!(Kips);
impl_arithmetic!(Psi);
impl_arithmetic!(Ksi);
impl_arithmetic!(FtLb);
impl_arithmetic!(KipFt);
impl_arithmetic!(InLb);
impl_arithmetic!(KipIn);
impl_arithmetic!(PlF);
impl_arithmetic!(KlF);
impl_arithmetic!(SqIn);
impl_arithmetic!(SqFt);
impl_arithmetic!(In4);
impl_arithmetic!(In3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feet_to_inches() {
        let ft = Feet(10.0);
        let inches: Inches = ft.into();
        assert_eq!(inches.0, 120.0);
    }

    #[test]
    fn test_kips_to_pounds() {
        let k = Kips(1.5);
        let lb: Pounds = k.into();
        assert_eq!(lb.0, 1500.0);
    }

    #[test]
    fn test_arithmetic() {
        let a = Feet(10.0);
        let b = Feet(5.0);
        assert_eq!((a + b).0, 15.0);
        assert_eq!((a - b).0, 5.0);
        assert_eq!((a * 2.0).0, 20.0);
        assert_eq!((a / 2.0).0, 5.0);
    }

    #[test]
    fn test_serialization() {
        let ft = Feet(12.5);
        let json = serde_json::to_string(&ft).unwrap();
        assert_eq!(json, "12.5");

        let roundtrip: Feet = serde_json::from_str(&json).unwrap();
        assert_eq!(ft, roundtrip);
    }
}
