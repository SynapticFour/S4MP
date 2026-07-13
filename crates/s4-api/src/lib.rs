//! # s4-api
//!
//! HTTP and gRPC API contracts. Transport implementations are added in later phases.

#![warn(missing_docs)]

/// API route identifiers and payloads.
pub mod routes;
/// API server lifecycle trait.
pub mod server;
/// Transport configuration types.
pub mod transport;

pub use routes::{ApiRoute, HealthResponse};
pub use server::ApiServer;
pub use transport::{Transport, TransportConfig, TransportKind};
