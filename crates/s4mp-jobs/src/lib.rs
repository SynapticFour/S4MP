//! Async job queue abstraction (local first, distributed later).

pub mod job;
pub mod scheduler;

pub use job::{Job, JobId, JobStatus};
pub use scheduler::Scheduler;
