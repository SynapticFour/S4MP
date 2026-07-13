use crate::Event;
use async_trait::async_trait;
use s4_core::Result;

/// Opaque subscription identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);

/// Active event subscription handle.
pub struct Subscription {
    /// Subscription identifier.
    pub id: SubscriptionId,
}

/// Handler invoked when a matching event is published.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Process a single event.
    async fn handle(&self, event: &Event) -> Result<()>;
}
