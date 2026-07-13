use async_trait::async_trait;
use s4_core::Result;

/// API server lifecycle.
#[async_trait]
pub trait ApiServer: Send + Sync {
    /// Bind and start listening.
    async fn start(&self) -> Result<()>;

    /// Gracefully shut down.
    async fn shutdown(&self) -> Result<()>;
}
