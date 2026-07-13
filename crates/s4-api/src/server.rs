use async_trait::async_trait;
use s4_core::Result;

/// API server lifecycle.
#[async_trait]
pub trait ApiServer: Send + Sync {
    /// Bind and start listening.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot start.
    async fn start(&self) -> Result<()>;

    /// Gracefully shut down.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    async fn shutdown(&self) -> Result<()>;
}
