//! Workspace maturity and metadata constants.

/// Current product maturity for the shipped porting pipeline.
///
/// Diff reports and certificates must not claim stronger guarantees than this.
pub const MATURITY: &str = "heuristic-map-v2";

/// Human-readable maturity notice for CLI and Markdown reports.
pub const MATURITY_NOTICE: &str = concat!(
    "Maturity: heuristic-map-v2. ",
    "Name (+ optional signature) similarity maps only — not semantic equivalence, not a certificate. ",
    "`s4 certify` / `s4 verify` are not implemented."
);
