//! Plugin host: loading, sandboxing, and invocation lifecycle.

pub mod host;
pub mod sandbox;

pub use host::PluginHost;
pub use sandbox::{Sandbox, TrustTier};
