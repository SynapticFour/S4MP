use crate::workspace::{source_ref_from_flags, Workspace};
use s4_core::Result;
use s4_project::{DefaultSourceIngestor, SourceIngestor, SourceOrigin};
use std::fmt::Write as _;

/// Register or list workspace sources.
pub fn run_add(
    alias: &str,
    git: Option<&str>,
    local: Option<&str>,
    git_ref: Option<&str>,
    subpath: Option<&str>,
    lang: &str,
) -> Result<()> {
    let ws = Workspace::open(".")?;
    let source = source_ref_from_flags(alias, git, local, git_ref, subpath, lang)?;

    let mut registry = ws.load_sources()?;
    if registry.sources.iter().any(|s| s.alias == alias) {
        return Err(s4_core::S4Error::Other(format!(
            "source alias already registered: {alias}"
        )));
    }

    match &source.origin {
        SourceOrigin::Git { url, .. } => {
            println!("Cloning {alias} from {url}...");
        },
        SourceOrigin::Local { path } => {
            println!("Registering local source {alias} at {}...", path.display());
        },
    }

    let ingestor = DefaultSourceIngestor::new(ws.root().to_path_buf());
    let resolved = ingestor.resolve(&source)?;
    println!(
        "Resolved {alias} -> {} ({} files ready for graph build)",
        resolved.local_root.display(),
        if resolved.commit.is_some() {
            "git"
        } else {
            "local"
        }
    );
    if let Some(commit) = &resolved.commit {
        println!("  commit: {commit}");
    }

    registry.sources.push(source);
    ws.save_sources(&registry)?;
    println!("Registered source '{alias}' in {}", ws.sources_path().display());
    Ok(())
}

/// List registered sources.
pub fn run_list() -> Result<()> {
    let ws = Workspace::open(".")?;
    let registry = ws.load_sources()?;
    if registry.sources.is_empty() {
        println!("No sources registered. Use `s4 source add` first.");
        return Ok(());
    }
    println!("Registered sources ({}):", registry.sources.len());
    for source in &registry.sources {
        let origin = match &source.origin {
            SourceOrigin::Git {
                url,
                git_ref,
                subpath,
            } => {
                let mut detail = url.clone();
                if let Some(reference) = git_ref {
                    let _ = write!(detail, " @ {reference}");
                }
                if let Some(sub) = subpath {
                    let _ = write!(detail, " ({sub})");
                }
                detail
            },
            SourceOrigin::Local { path } => path.display().to_string(),
        };
        println!(
            "  {}  [{}]  {}",
            source.alias, source.language.0, origin
        );
    }
    Ok(())
}
