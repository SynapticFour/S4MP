use serde::{Deserialize, Serialize};
use std::fmt;

/// Content-addressed identifier for immutable artifacts (Blake3, hex-encoded).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId([u8; 32]);

impl ArtifactId {
    /// Construct from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Derive identifier from content bytes.
    #[must_use]
    pub fn from_content(content: &[u8]) -> Self {
        Self(*blake3::hash(content).as_bytes())
    }

    /// Access raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArtifactId({})", hex_short(&self.0))
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex_full(&self.0))
    }
}

/// Opaque identifier for a graph entity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct EntityId(pub u64);

/// Identifies a registered plugin.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct PluginId(pub String);

/// Identifies a S4MP project workspace.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ProjectId(pub String);

fn hex_short(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(16);
    for b in &bytes[..8] {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn hex_full(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
    }
    s
}
