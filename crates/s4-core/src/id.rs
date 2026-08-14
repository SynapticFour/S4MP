use crate::{Result, S4Error};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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

impl FromStr for ArtifactId {
    type Err = S4Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 64 {
            return Err(S4Error::InvalidId(format!(
                "artifact id must be 64 hex characters, got {}",
                s.len()
            )));
        }
        let mut out = [0_u8; 32];
        for (index, chunk) in s.as_bytes().chunks(2).enumerate() {
            let pair = std::str::from_utf8(chunk)
                .map_err(|e| S4Error::InvalidId(format!("invalid artifact id encoding: {e}")))?;
            out[index] = u8::from_str_radix(pair, 16)
                .map_err(|e| S4Error::InvalidId(format!("invalid artifact id hex: {e}")))?;
        }
        Ok(Self(out))
    }
}

/// Snapshot-scoped knowledge identifier (graph alias + node id).
///
/// Distinct from graph-local node ids: traces name both the source graph and the
/// node so identifiers are not mixed across snapshots.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct EntityId {
    /// Source graph alias (workspace source name).
    pub graph: String,
    /// Node identifier within that graph.
    pub node: u64,
}

impl EntityId {
    /// Bind a graph-local node id to a named source graph.
    #[must_use]
    pub fn new(graph: impl Into<String>, node: u64) -> Self {
        Self {
            graph: graph.into(),
            node,
        }
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.graph, self.node)
    }
}

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
