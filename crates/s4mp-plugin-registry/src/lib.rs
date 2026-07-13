//! Plugin discovery, manifest validation, and semver resolution.

pub mod registry;
pub mod resolver;

pub use registry::Registry;
pub use resolver::Resolver;
