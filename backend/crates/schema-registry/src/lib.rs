// SPEC-009: Versioned Schema Registry — core v1 implementation.

#![recursion_limit = "512"]

pub mod core;
pub mod descriptor;
pub mod lint;
pub mod registry;

pub use registry::{NamespaceDoc, RegistryError, SchemaRegistry};
