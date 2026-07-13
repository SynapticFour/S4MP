//! Project workspace: configuration, plugin resolution, snapshot refs.

pub mod config;
pub mod lockfile;
pub mod workspace;

pub use config::ProjectConfig;
pub use lockfile::Lockfile;
pub use workspace::Workspace;
