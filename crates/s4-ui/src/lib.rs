//! # s4-ui
//!
//! Headless UI contracts for IDE extensions and web frontends.

#![warn(missing_docs)]

/// UI bridge trait.
pub mod bridge;
/// Component descriptor types.
pub mod component;
/// Theme and token types.
pub mod theme;
/// View descriptor types.
pub mod view;

pub use bridge::UiBridge;
pub use component::{Component, ComponentId, ComponentKind};
pub use theme::{Theme, ThemeTokens};
pub use view::{View, ViewId, ViewState};
