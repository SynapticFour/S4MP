use serde::{Deserialize, Serialize};

/// UI theme identifier.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name.
    pub name: String,
    /// Design tokens.
    pub tokens: ThemeTokens,
}

/// Design tokens for UI themes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThemeTokens {
    /// Primary accent color (hex).
    pub accent: Option<String>,
    /// Background color (hex).
    pub background: Option<String>,
    /// Font family name.
    pub font_family: Option<String>,
}
