use s4_parser::LanguageId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Provenance of a registered source tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOrigin {
    /// Remote Git repository, optionally pinned to a ref and scoped to a subdirectory.
    Git {
        /// Clone URL (e.g. `https://github.com/broadinstitute/gatk.git`).
        url: String,
        /// Branch, tag, or commit to checkout. `None` uses the repository default.
        git_ref: Option<String>,
        /// Path within the repository to limit parsing scope (e.g. a package subdirectory).
        subpath: Option<String>,
    },
    /// Filesystem directory on the local machine.
    Local {
        /// Absolute or workspace-relative path to the source root.
        path: PathBuf,
    },
}

/// User-defined reference to a source tree for ingestion.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceRef {
    /// Short user-assigned name (e.g. `"gatk-java-hc"`, `"hc-rust"`).
    pub alias: String,
    /// Primary language of the source for parser selection.
    pub language: LanguageId,
    /// Where the source lives.
    pub origin: SourceOrigin,
}
