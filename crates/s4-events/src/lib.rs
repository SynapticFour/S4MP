//! # s4-events
//!
//! Event bus contracts for decoupled platform subsystems.

#![warn(missing_docs)]

/// Event bus trait.
pub mod bus;
/// Domain event types.
pub mod event;
/// Sync in-process recording sink (Phase 2 CLI pipelines).
pub mod memory;
/// Subscription and handler traits.
pub mod subscription;

pub use bus::EventBus;
pub use event::{Event, EventKind};
pub use memory::RecordingEventSink;
pub use subscription::{EventHandler, Subscription, SubscriptionId};
