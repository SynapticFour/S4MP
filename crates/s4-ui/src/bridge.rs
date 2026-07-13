use crate::{Component, View, ViewId};
use s4_core::Result;

/// Bridge between S4MP backend state and a UI renderer.
pub trait UiBridge: Send + Sync {
    /// Render or update a view.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    fn render_view(&self, view: &View) -> Result<()>;

    /// Push component updates to the renderer.
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    fn update_components(&self, view_id: ViewId, components: &[Component]) -> Result<()>;
}
