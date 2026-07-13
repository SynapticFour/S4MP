use serde::{Deserialize, Serialize};

/// Opaque view identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewId(pub u64);

/// UI view descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    /// View identifier.
    pub id: ViewId,
    /// View title.
    pub title: String,
    /// Current view state.
    pub state: ViewState,
}

/// View lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewState {
    /// View is loading data.
    Loading,
    /// View is ready for interaction.
    Ready,
    /// View encountered an error.
    Error,
}
