//! Declarative DAG pipeline execution and incremental invalidation.

pub mod executor;
pub mod invalidation;
pub mod spec;
pub mod stage;

pub use executor::Executor;
pub use invalidation::InvalidationGraph;
pub use spec::PipelineSpec;
pub use stage::Stage;
