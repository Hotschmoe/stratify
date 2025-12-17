//! # calc_core - Structural Engineering Calculation Engine
//!
//! `calc_core` is the computational heart of Stratify, providing structural engineering
//! calculations with a clean, LLM-friendly API. All inputs and outputs are JSON-serializable,
//! making it ideal for integration with AI assistants via MCP or similar protocols.
//!
//! ## Design Philosophy
//!
//! - **Stateless**: Pure functions that take input and return results
//! - **JSON-First**: All types implement Serialize/Deserialize
//! - **Rich Errors**: Structured error types, not just strings
//! - **Well-Documented**: Every type and function has examples
//!
//! ## Quick Start
//!
//! ```rust
//! use calc_core::project::Project;
//!
//! // Create a new project
//! let project = Project::new("John Engineer", "25-001", "Acme Construction");
//!
//! // Serialize to JSON for storage or transmission
//! let json = serde_json::to_string_pretty(&project).unwrap();
//! ```
//!
//! ## Modules
//!
//! - [`project`] - Project container, metadata, and settings
//! - [`calculations`] - All structural calculation types (beams, columns, etc.)
//! - [`materials`] - Material definitions and databases
//! - [`units`] - Type-safe unit wrappers
//! - [`errors`] - Structured error types
//! - [`file_io`] - File operations with atomic saves and locking

pub mod calculations;
pub mod errors;
pub mod file_io;
pub mod materials;
pub mod project;
pub mod units;

// Re-export commonly used types at crate root for convenience
pub use errors::{CalcError, CalcResult};
pub use file_io::{save_project, load_project, FileLock};
pub use project::{Project, ProjectMetadata, GlobalSettings};
