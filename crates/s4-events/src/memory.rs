//! Sync in-process event recording (Phase 2).
//!
//! Full async [`crate::EventBus`] remains the long-term contract; CLI pipelines use this
//! sink until a runtime is wired.

use crate::{Event, EventKind};
use s4_core::ProjectId;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Thread-safe recorder for pipeline events.
#[derive(Debug, Default)]
pub struct RecordingEventSink {
    events: Mutex<Vec<Event>>,
}

impl RecordingEventSink {
    /// Create an empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an event with the current UTC unix timestamp.
    pub fn emit(&self, kind: EventKind, project_id: Option<ProjectId>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or_else(|_| "0".to_string(), |d| d.as_secs().to_string());
        let event = Event {
            kind,
            project_id,
            timestamp,
        };
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
    }

    /// Snapshot recorded events in order.
    #[must_use]
    pub fn events(&self) -> Vec<Event> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Number of recorded events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Returns true when no events were recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s4_core::ArtifactId;

    #[test]
    fn records_graph_updated() {
        let sink = RecordingEventSink::new();
        let id = ArtifactId::from_content(b"projection");
        sink.emit(EventKind::GraphUpdated { projection: id }, None);
        assert_eq!(sink.len(), 1);
        assert!(matches!(
            sink.events()[0].kind,
            EventKind::GraphUpdated { .. }
        ));
    }
}
