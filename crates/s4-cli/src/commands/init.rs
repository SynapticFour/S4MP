//! Initialize an S4MP workspace on disk.

use crate::workspace::{SourceRegistry, Workspace, WorkspaceMeta};
use s4_core::{Result, S4Error, SchemaVersion, MATURITY};
use std::path::Path;

/// Create `.s4/` layout, empty source registry, and workspace metadata.
///
/// Idempotent: existing workspaces are left intact and reported as already initialized.
///
/// # Errors
///
/// Returns an error if directories or metadata cannot be written.
pub fn run(path: &str) -> Result<()> {
    let root = Path::new(path);
    std::fs::create_dir_all(root).map_err(|e| {
        S4Error::Storage(format!(
            "failed to create workspace root {}: {e}",
            root.display()
        ))
    })?;

    let ws = Workspace::open(root)?;
    let meta_path = ws.meta_path();
    let already = meta_path.is_file();

    ws.ensure_layout()?;

    if already {
        let meta = ws.load_meta()?;
        println!("Initialized S4MP workspace at {}", ws.root().display());
        println!("  maturity: {}", meta.maturity);
        println!("  schema:   {}", meta.schema_version);
        println!("  note:     workspace metadata already present (layout ensured)");
        println!("Next: s4 source add <alias> --local <path> --lang <java|rust>");
        return Ok(());
    }

    let meta = WorkspaceMeta {
        schema_version: SchemaVersion::CURRENT,
        maturity: MATURITY.to_string(),
    };
    ws.save_meta(&meta)?;
    let registry = SourceRegistry::default();
    ws.save_sources(&registry)?;

    println!("Initialized S4MP workspace at {}", ws.root().display());
    println!("  maturity: {MATURITY}");
    println!("  schema:   {}", SchemaVersion::CURRENT);
    println!("Next: s4 source add <alias> --local <path> --lang <java|rust>");
    Ok(())
}
