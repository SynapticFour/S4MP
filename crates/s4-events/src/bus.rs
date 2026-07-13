use crate::{Event, EventHandler, SubscriptionId};
use async_trait::async_trait;
use s4_core::Result;

/// Publish/subscribe event bus.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to all subscribers.
    async fn publish(&self, event: Event) -> Result<()>;

    /// Subscribe to events matching `kind_filter`. `None` matches all kinds.
    async fn subscribe(
        &self,
        kind_filter: Option<String>,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId>;
}
