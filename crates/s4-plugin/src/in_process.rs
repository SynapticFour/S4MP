//! In-process plugin host (Phase 6 — trusted first-party plugins only).

use crate::manifest::PluginManifest;
use crate::PluginHost;
use s4_core::{ApiVersion, PluginId, Result, S4Error};
use std::collections::BTreeMap;

/// Static, in-process plugin registry.
#[derive(Clone, Debug, Default)]
pub struct InProcessPluginHost {
    manifests: BTreeMap<String, PluginManifest>,
}

impl InProcessPluginHost {
    /// Create an empty host.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register the built-in first-party plugins shipped with S4MP.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut host = Self::new();
        for manifest in builtin_manifests() {
            let _ = host.register(manifest);
        }
        host
    }

    /// Iterate registered manifests in id order.
    pub fn manifests(&self) -> impl Iterator<Item = &PluginManifest> {
        self.manifests.values()
    }

    /// Validate that a manifest's API version is compatible with the host.
    ///
    /// # Errors
    ///
    /// Returns an error when the API major version mismatches.
    pub fn check_api_compatible(manifest: &PluginManifest) -> Result<()> {
        let required = parse_api_version(&manifest.api_version)?;
        let current = ApiVersion::CURRENT;
        if !current.is_compatible_with(&required) {
            return Err(S4Error::Other(format!(
                "plugin '{}' requires API {}, host provides {}",
                manifest.name, required, current
            )));
        }
        Ok(())
    }
}

impl PluginHost for InProcessPluginHost {
    fn register(&mut self, manifest: PluginManifest) -> Result<()> {
        Self::check_api_compatible(&manifest)?;
        let key = manifest.name.clone();
        if self.manifests.contains_key(&key) {
            return Err(S4Error::Other(format!("plugin already registered: {key}")));
        }
        self.manifests.insert(key, manifest);
        Ok(())
    }

    fn manifest(&self, id: &PluginId) -> Option<&PluginManifest> {
        self.manifests.get(&id.0)
    }

    fn count(&self) -> usize {
        self.manifests.len()
    }
}

fn parse_api_version(raw: &str) -> Result<ApiVersion> {
    let mut parts = raw.trim().trim_start_matches('v').split('.');
    let major = parts
        .next()
        .ok_or_else(|| S4Error::Other(format!("invalid api_version '{raw}'")))?
        .parse::<u32>()
        .map_err(|_| S4Error::Other(format!("invalid api_version major in '{raw}'")))?;
    let minor = parts
        .next()
        .unwrap_or("0")
        .parse::<u32>()
        .map_err(|_| S4Error::Other(format!("invalid api_version minor in '{raw}'")))?;
    Ok(ApiVersion { major, minor })
}

fn builtin_manifests() -> Vec<PluginManifest> {
    use crate::{CapabilitySet, PluginCapability};
    vec![
        PluginManifest {
            name: "s4-parser-java".into(),
            version: "0.1.0".into(),
            api_version: "0.1".into(),
            capabilities: CapabilitySet {
                roles: vec![PluginCapability::Parser],
                languages: vec!["java".into()],
                file_patterns: vec!["**/*.java".into()],
            },
            description: Some("Tree-sitter Java frontend (in-process)".into()),
        },
        PluginManifest {
            name: "s4-parser-rust".into(),
            version: "0.1.0".into(),
            api_version: "0.1".into(),
            capabilities: CapabilitySet {
                roles: vec![PluginCapability::Parser],
                languages: vec!["rust".into()],
                file_patterns: vec!["**/*.rs".into()],
            },
            description: Some("Tree-sitter Rust frontend (in-process)".into()),
        },
        PluginManifest {
            name: "s4-reasoner-heuristic".into(),
            version: "0.1.0".into(),
            api_version: "0.1".into(),
            capabilities: CapabilitySet {
                roles: vec![PluginCapability::Reasoner],
                languages: vec![],
                file_patterns: vec![],
            },
            description: Some(
                "Offline heuristic reasoner — outputs always Proposed (no network LLM)".into(),
            ),
        },
        PluginManifest {
            name: "s4-verifier-port-diff".into(),
            version: "0.1.0".into(),
            api_version: "0.1".into(),
            capabilities: CapabilitySet {
                roles: vec![PluginCapability::Verifier],
                languages: vec!["java".into(), "rust".into()],
                file_patterns: vec![],
            },
            description: Some("Port-diff coverage verifier (in-process)".into()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_register() {
        let host = InProcessPluginHost::with_builtins();
        assert!(host.count() >= 3);
        assert!(host
            .manifest(&PluginId("s4-reasoner-heuristic".into()))
            .is_some());
    }

    #[test]
    fn rejects_duplicate() {
        let mut host = InProcessPluginHost::new();
        let m = builtin_manifests().into_iter().next().unwrap();
        host.register(m.clone()).unwrap();
        assert!(host.register(m).is_err());
    }
}
