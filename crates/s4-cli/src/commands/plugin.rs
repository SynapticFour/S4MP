//! Plugin registry listing (`s4 plugin`).

use s4_core::Result;
use s4_plugin::{InProcessPluginHost, PluginHost};

/// List built-in in-process plugins.
#[allow(clippy::unnecessary_wraps)] // matches other CLI `run_*` signatures
pub fn run_list() -> Result<()> {
    let host = InProcessPluginHost::with_builtins()?;
    println!("registered plugins: {}", host.count());
    for manifest in host.manifests() {
        let roles: Vec<_> = manifest
            .capabilities
            .roles
            .iter()
            .map(|r| format!("{r:?}").to_ascii_lowercase())
            .collect();
        let langs = if manifest.capabilities.languages.is_empty() {
            "—".to_string()
        } else {
            manifest.capabilities.languages.join(",")
        };
        println!(
            "  {} v{}  api={}  roles=[{}]  langs=[{}]",
            manifest.name,
            manifest.version,
            manifest.api_version,
            roles.join(","),
            langs
        );
        if let Some(desc) = &manifest.description {
            println!("    {desc}");
        }
    }
    println!("note: these are first-party in-process frontends, not a loadable plugin runtime. WASM is deferred (ADR-016).");
    Ok(())
}

/// Confirm a plugin id is registered (used by `s4 reason` default provider check).
pub fn ensure_registered(plugin_id: &str) -> Result<()> {
    let host = InProcessPluginHost::with_builtins()?;
    if host
        .manifest(&s4_core::PluginId(plugin_id.to_string()))
        .is_none()
    {
        return Err(s4_core::S4Error::Plugin {
            plugin_id: plugin_id.to_string(),
            message: "plugin not registered".into(),
        });
    }
    Ok(())
}
