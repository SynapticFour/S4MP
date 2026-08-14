//! Cross-graph node correspondence (e.g. Java porting to Rust).
//!
//! # Model note: `ExtraInTarget` entries
//!
//! [`CorrespondenceEntry::source_node`] is `Option<NodeRef>` so a single `Vec<CorrespondenceEntry>`
//! can represent both Java-anchored rows (`MissingInTarget`, `Diverged`, …) and Rust-only rows
//! (`ExtraInTarget` with `source_node: None`). A separate “extras” list would force every consumer
//! to merge two collections; optional `source_node` keeps one stream with explicit status.
//!
//! # Heuristic limits (v2)
//!
//! [`suggest_correspondences`] combines **tokenized name Jaccard** with optional **signature
//! Jaccard** when both sides have signatures. Matching is **exclusive** (one Rust node per
//! Java node). This is still not semantic equivalence or type checking. Entries with heuristic
//! methods and status [`CorrespondenceStatus::Diverged`] must be manually confirmed before
//! trusting them in certification workflows.

use s4_core::{ArtifactId, Result, S4Error, SchemaVersion};
use s4_graph::{GraphView, Node, NodeId, NodeKind};
use s4_storage::{Artifact, ArtifactKind, StoreReader, StoreWriter};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

/// Alias identifying a materialized graph (typically matches a source alias such as `"gatk-java-hc"`).
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct GraphId(pub String);

/// Reference to a node in a named graph.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NodeRef {
    /// Graph alias.
    pub graph: GraphId,
    /// Node within the graph.
    pub node: NodeId,
}

/// Lifecycle of a cross-graph correspondence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrespondenceStatus {
    /// Both sides present and manually or deterministically confirmed.
    Ported,
    /// Paired, but structural or heuristic signals disagree — review required.
    Diverged,
    /// Present on the source (Java) graph only.
    MissingInTarget,
    /// Present on the target (Rust) graph only.
    ExtraInTarget,
    /// Source-side node with no assignment yet (including no accepted suggestion).
    Unmapped,
}

/// How a correspondence was established.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrespondenceMethod {
    /// Human-confirmed mapping.
    Manual,
    /// Tokenized name similarity only.
    NameHeuristic,
    /// Name + signature similarity (v2).
    SignatureHeuristic,
    /// LLM-proposed mapping (always requires confirmation).
    LlmSuggested,
}

/// One correspondence row between a source graph node and an optional target node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorrespondenceEntry {
    /// Stable row identifier (Blake3 hex over graph/node pair).
    pub id: String,
    /// Source-side node when the row is anchored on the Java (source) graph.
    pub source_node: Option<NodeRef>,
    /// Target-side node when a Rust counterpart exists or is suggested.
    pub target_node: Option<NodeRef>,
    /// Review state of this correspondence.
    pub status: CorrespondenceStatus,
    /// Confidence score in `0.0–1.0`.
    pub confidence: f32,
    /// Provenance of the mapping.
    pub method: CorrespondenceMethod,
    /// Optional reviewer or pipeline note.
    pub note: Option<String>,
    /// Display label for reports (source label, or target label for extras).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// `true` when a retained manual row no longer appears in fresh suggestions (e.g. source node removed).
    #[serde(default)]
    pub stale: bool,
}

/// Minimum combined similarity to emit a heuristic pairing.
const SIMILARITY_THRESHOLD: f32 = 0.5;
/// Weight for name similarity when signatures are available on both sides.
const NAME_WEIGHT_WITH_SIG: f32 = 0.6;
/// Weight for signature similarity when available on both sides.
const SIG_WEIGHT: f32 = 0.4;

struct TokenizedNode {
    id: NodeId,
    kind: NodeKind,
    label: String,
    name_tokens: HashSet<String>,
    sig_tokens: Option<HashSet<String>>,
}

struct ScoredPair {
    java: usize,
    rust: usize,
    similarity: f32,
    method: CorrespondenceMethod,
}

/// Suggest Java→Rust node correspondences using name (+ optional signature) similarity.
///
/// Only [`NodeKind::Callable`] and [`NodeKind::Type`] nodes are considered.
/// Empty and `<anonymous>` labels are skipped. Assignment is exclusive: each
/// Rust node is paired with at most one Java node (highest score first).
///
/// # Heuristic (v2)
///
/// 1. Tokenize labels (`camelCase` + `snake_case` → lowercase word tokens) once per node.
/// 2. If both nodes have signatures, also tokenize signatures and combine
///    `0.6 * name + 0.4 * signature` Jaccard scores.
/// 3. Pairs ≥ 0.5 are assigned greedily by descending score → [`CorrespondenceStatus::Diverged`]
///    (never auto-`Ported`).
/// 4. Unassigned Java nodes → [`CorrespondenceStatus::MissingInTarget`].
/// 5. Unassigned Rust nodes → [`CorrespondenceStatus::ExtraInTarget`].
#[must_use]
pub fn suggest_correspondences(
    java: &dyn GraphView,
    java_id: &GraphId,
    rust: &dyn GraphView,
    rust_id: &GraphId,
) -> Vec<CorrespondenceEntry> {
    let java_nodes = collect_typed_nodes(java);
    let rust_nodes = collect_typed_nodes(rust);
    let assignment = exclusive_assignment(&java_nodes, &rust_nodes);
    emit_correspondence_entries(java_id, rust_id, &java_nodes, &rust_nodes, &assignment)
}

fn exclusive_assignment(
    java_nodes: &[TokenizedNode],
    rust_nodes: &[TokenizedNode],
) -> Vec<Option<(usize, f32, CorrespondenceMethod)>> {
    let rust_index = token_index(rust_nodes);
    let mut pairs = Vec::new();
    for (ji, java_node) in java_nodes.iter().enumerate() {
        for ri in candidate_indices(java_node, &rust_index) {
            let rust_node = &rust_nodes[ri];
            if rust_node.kind != java_node.kind {
                continue;
            }
            let (similarity, method) = score_tokenized(java_node, rust_node);
            if similarity >= SIMILARITY_THRESHOLD {
                pairs.push(ScoredPair {
                    java: ji,
                    rust: ri,
                    similarity,
                    method,
                });
            }
        }
    }

    pairs.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(Ordering::Equal)
            .then_with(|| java_nodes[a.java].id.0.cmp(&java_nodes[b.java].id.0))
            .then_with(|| rust_nodes[a.rust].id.0.cmp(&rust_nodes[b.rust].id.0))
    });

    let mut matched_java = vec![false; java_nodes.len()];
    let mut matched_rust = vec![false; rust_nodes.len()];
    let mut assignment: Vec<Option<(usize, f32, CorrespondenceMethod)>> =
        vec![None; java_nodes.len()];

    for pair in pairs {
        if matched_java[pair.java] || matched_rust[pair.rust] {
            continue;
        }
        matched_java[pair.java] = true;
        matched_rust[pair.rust] = true;
        assignment[pair.java] = Some((pair.rust, pair.similarity, pair.method));
    }
    assignment
}

fn emit_correspondence_entries(
    java_id: &GraphId,
    rust_id: &GraphId,
    java_nodes: &[TokenizedNode],
    rust_nodes: &[TokenizedNode],
    assignment: &[Option<(usize, f32, CorrespondenceMethod)>],
) -> Vec<CorrespondenceEntry> {
    let mut matched_rust = vec![false; rust_nodes.len()];
    for assigned in assignment.iter().flatten() {
        matched_rust[assigned.0] = true;
    }

    let rust_index = token_index(rust_nodes);
    let mut entries = Vec::new();
    for (ji, java_node) in java_nodes.iter().enumerate() {
        let source_ref = Some(NodeRef {
            graph: java_id.clone(),
            node: java_node.id,
        });
        if let Some((ri, similarity, method)) = assignment[ji] {
            let rust_node = &rust_nodes[ri];
            let target_ref = Some(NodeRef {
                graph: rust_id.clone(),
                node: rust_node.id,
            });
            let note = match method {
                CorrespondenceMethod::SignatureHeuristic => {
                    "name+signature heuristic v2 — manual confirmation required before treating as ported"
                }
                _ => "name heuristic v2 — manual confirmation required before treating as ported",
            };
            entries.push(CorrespondenceEntry {
                id: entry_id(source_ref.as_ref(), target_ref.as_ref()),
                source_node: source_ref,
                target_node: target_ref,
                status: CorrespondenceStatus::Diverged,
                confidence: similarity,
                method,
                note: Some(note.to_string()),
                display_name: Some(java_node.label.clone()),
                stale: false,
            });
        } else {
            let best = best_unused(java_node, rust_nodes, &matched_rust, &rust_index);
            let method = best.map_or(CorrespondenceMethod::NameHeuristic, |(_, m)| m);
            entries.push(CorrespondenceEntry {
                id: entry_id(source_ref.as_ref(), None),
                source_node: source_ref,
                target_node: None,
                status: CorrespondenceStatus::MissingInTarget,
                confidence: best.map_or(0.0, |(s, _)| s),
                method,
                note: Some("no Rust name/signature match above similarity threshold".to_string()),
                display_name: Some(java_node.label.clone()),
                stale: false,
            });
        }
    }

    for (ri, rust_node) in rust_nodes.iter().enumerate() {
        if matched_rust[ri] {
            continue;
        }
        let target_ref = Some(NodeRef {
            graph: rust_id.clone(),
            node: rust_node.id,
        });
        entries.push(CorrespondenceEntry {
            id: entry_id(None, target_ref.as_ref()),
            source_node: None,
            target_node: target_ref,
            status: CorrespondenceStatus::ExtraInTarget,
            confidence: 0.0,
            method: CorrespondenceMethod::NameHeuristic,
            note: Some("Rust node has no Java heuristic counterpart".to_string()),
            display_name: Some(rust_node.label.clone()),
            stale: false,
        });
    }
    entries
}

fn best_unused(
    java: &TokenizedNode,
    rust_nodes: &[TokenizedNode],
    matched_rust: &[bool],
    rust_index: &HashMap<String, Vec<usize>>,
) -> Option<(f32, CorrespondenceMethod)> {
    let mut best: Option<(f32, CorrespondenceMethod)> = None;
    for ri in candidate_indices(java, rust_index) {
        if matched_rust[ri] {
            continue;
        }
        let rust_node = &rust_nodes[ri];
        if rust_node.kind != java.kind {
            continue;
        }
        let (similarity, method) = score_tokenized(java, rust_node);
        if best.as_ref().map_or(true, |(s, _)| similarity > *s) {
            best = Some((similarity, method));
        }
    }
    best
}

fn token_index(nodes: &[TokenizedNode]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        for token in &node.name_tokens {
            index.entry(token.clone()).or_default().push(i);
        }
        if let Some(sig) = &node.sig_tokens {
            for token in sig {
                index.entry(token.clone()).or_default().push(i);
            }
        }
    }
    index
}

fn candidate_indices(java: &TokenizedNode, index: &HashMap<String, Vec<usize>>) -> HashSet<usize> {
    let mut out = HashSet::new();
    for token in &java.name_tokens {
        if let Some(ids) = index.get(token) {
            out.extend(ids);
        }
    }
    if let Some(sig) = &java.sig_tokens {
        for token in sig {
            if let Some(ids) = index.get(token) {
                out.extend(ids);
            }
        }
    }
    out
}

fn score_tokenized(java: &TokenizedNode, rust: &TokenizedNode) -> (f32, CorrespondenceMethod) {
    let name_sim = jaccard(&java.name_tokens, &rust.name_tokens);
    match (&java.sig_tokens, &rust.sig_tokens) {
        (Some(js), Some(rs)) => {
            let sig_sim = jaccard(js, rs);
            let combined = NAME_WEIGHT_WITH_SIG * name_sim + SIG_WEIGHT * sig_sim;
            (combined, CorrespondenceMethod::SignatureHeuristic)
        },
        _ => (name_sim, CorrespondenceMethod::NameHeuristic),
    }
}

#[cfg(test)]
fn score_pair(java: &Node, rust: &Node) -> (f32, CorrespondenceMethod) {
    score_tokenized(&tokenize_node(java), &tokenize_node(rust))
}

fn tokenize_node(node: &Node) -> TokenizedNode {
    TokenizedNode {
        id: node.id,
        kind: node.kind.clone(),
        label: node.label.clone(),
        name_tokens: tokenize_name(&node.label),
        sig_tokens: node.signature.as_deref().map(tokenize_signature),
    }
}

fn tokenize_signature(sig: &str) -> HashSet<String> {
    let mut tokens = HashSet::new();
    let mut current = String::new();
    for ch in sig.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch.to_ascii_lowercase());
        } else {
            push_token(&mut current, &mut tokens);
        }
    }
    push_token(&mut current, &mut tokens);
    tokens
}

/// Load a correspondence map artifact from the content-addressed store.
///
/// # Errors
///
/// Returns an error if the artifact is missing, not a [`ArtifactKind::CorrespondenceMap`], or
/// the payload cannot be deserialized.
pub fn load_correspondence_map(
    store: &dyn StoreReader,
    id: &ArtifactId,
) -> Result<Vec<CorrespondenceEntry>> {
    let artifact = store
        .read(id)?
        .ok_or_else(|| S4Error::Storage(format!("correspondence map artifact not found: {id}")))?;
    artifact.expect_current_schema()?;
    if artifact.kind != ArtifactKind::CorrespondenceMap {
        return Err(S4Error::Storage(format!(
            "expected correspondence_map artifact, got {:?}",
            artifact.kind
        )));
    }
    serde_json::from_value(artifact.payload)
        .map_err(|e| S4Error::Storage(format!("failed to deserialize correspondence map: {e}")))
}

/// Persist correspondence entries as a [`ArtifactKind::CorrespondenceMap`] artifact.
///
/// # Errors
///
/// Returns an error if serialization or storage fails.
pub fn save_correspondence_map(
    store: &mut dyn StoreWriter,
    entries: &[CorrespondenceEntry],
) -> Result<ArtifactId> {
    let payload = serde_json::to_value(entries)
        .map_err(|e| S4Error::Storage(format!("failed to serialize correspondence map: {e}")))?;
    let artifact = Artifact {
        kind: ArtifactKind::CorrespondenceMap,
        schema_version: SchemaVersion::CURRENT,
        payload,
    };
    store.write(&artifact)
}

/// Merge a persisted map with freshly suggested correspondences.
///
/// # Rules
///
/// - [`CorrespondenceMethod::Manual`] rows from `existing` are **never** replaced by `suggested`
///   (even when IDs match).
/// - Heuristic/LLM rows from prior runs are dropped; `suggested` becomes the new non-manual set.
/// - Suggested IDs not present in `existing` are appended.
/// - `existing` IDs absent from `suggested` are discarded, except manual rows — those are kept
///   with [`CorrespondenceEntry::stale`] set and a `source_node_missing` note.
#[must_use]
pub fn merge_correspondences(
    existing: Vec<CorrespondenceEntry>,
    suggested: Vec<CorrespondenceEntry>,
) -> Vec<CorrespondenceEntry> {
    let suggested_ids: HashSet<String> = suggested.iter().map(|e| e.id.clone()).collect();
    let manual_ids: HashSet<String> = existing
        .iter()
        .filter(|e| e.method == CorrespondenceMethod::Manual)
        .map(|e| e.id.clone())
        .collect();

    let mut merged = Vec::new();

    for mut entry in existing
        .into_iter()
        .filter(|e| e.method == CorrespondenceMethod::Manual)
    {
        if !suggested_ids.contains(&entry.id) {
            entry.stale = true;
            entry.note = Some(append_stale_note(entry.note.as_deref()));
        }
        merged.push(entry);
    }

    for entry in suggested {
        if !manual_ids.contains(&entry.id) {
            merged.push(entry);
        }
    }

    merged
}

fn append_stale_note(existing: Option<&str>) -> String {
    const FLAG: &str = "source_node_missing";
    match existing {
        Some(note) if note.contains(FLAG) => note.to_string(),
        Some(note) => format!("{note}; {FLAG}"),
        None => FLAG.to_string(),
    }
}

fn collect_typed_nodes(view: &dyn GraphView) -> Vec<TokenizedNode> {
    view.nodes()
        .filter(|node| matches!(node.kind, NodeKind::Callable | NodeKind::Type))
        .filter(|node| !node.label.is_empty() && node.label != "<anonymous>")
        .map(tokenize_node)
        .collect()
}

fn tokenize_name(name: &str) -> HashSet<String> {
    let mut tokens = HashSet::new();
    let mut current = String::new();

    for ch in name.chars() {
        if ch == '_' {
            push_token(&mut current, &mut tokens);
            continue;
        }
        if ch.is_ascii_uppercase() {
            push_token(&mut current, &mut tokens);
            current.push(ch.to_ascii_lowercase());
        } else {
            current.push(ch);
        }
    }
    push_token(&mut current, &mut tokens);
    tokens
}

fn push_token(current: &mut String, tokens: &mut HashSet<String>) {
    if !current.is_empty() {
        tokens.insert(std::mem::take(current));
    }
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count();
    let union = a.len() + b.len() - inter;
    if union == 0 {
        0.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        {
            inter as f32 / union as f32
        }
    }
}

fn entry_id(source: Option<&NodeRef>, target: Option<&NodeRef>) -> String {
    let payload = format!(
        "{}|{}",
        source.map_or_else(
            || "-".to_string(),
            |n| format!("{}:{}", n.graph.0, n.node.0)
        ),
        target.map_or_else(
            || "-".to_string(),
            |n| format!("{}:{}", n.graph.0, n.node.0)
        )
    );
    blake3::hash(payload.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use s4_graph::memory::InMemoryGraphView;

    fn entry(
        id: &str,
        method: CorrespondenceMethod,
        status: CorrespondenceStatus,
        confidence: f32,
    ) -> CorrespondenceEntry {
        CorrespondenceEntry {
            id: id.into(),
            source_node: None,
            target_node: None,
            status,
            confidence,
            method,
            note: None,
            display_name: None,
            stale: false,
        }
    }

    fn callable(id: u64, label: &str) -> Node {
        Node {
            id: NodeId(id),
            kind: NodeKind::Callable,
            label: label.into(),
            signature: None,
        }
    }

    #[test]
    fn tokenize_camel_and_snake_align() {
        let a = tokenize_name("haplotypeCaller");
        let b = tokenize_name("haplotype_caller");
        assert!((jaccard(&a, &b) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn empty_jaccard_is_zero() {
        assert!((jaccard(&HashSet::new(), &HashSet::new()) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn signature_boosts_score_method() {
        let java = Node {
            id: NodeId(0),
            kind: NodeKind::Callable,
            label: "add".into(),
            signature: Some("add(int a, int b):int".into()),
        };
        let rust = Node {
            id: NodeId(1),
            kind: NodeKind::Callable,
            label: "add".into(),
            signature: Some("add(a: i32, b: i32)->i32".into()),
        };
        let (score, method) = score_pair(&java, &rust);
        assert!(score >= SIMILARITY_THRESHOLD, "{score}");
        assert_eq!(method, CorrespondenceMethod::SignatureHeuristic);
    }

    #[test]
    fn exclusive_assignment_does_not_share_rust_nodes() {
        let java = InMemoryGraphView::new(vec![callable(10, "add"), callable(11, "add")], vec![]);
        let rust = InMemoryGraphView::new(vec![callable(20, "add")], vec![]);
        let entries =
            suggest_correspondences(&java, &GraphId("j".into()), &rust, &GraphId("r".into()));
        let diverged: Vec<_> = entries
            .iter()
            .filter(|e| e.status == CorrespondenceStatus::Diverged)
            .collect();
        assert_eq!(diverged.len(), 1);
        let missing = entries
            .iter()
            .filter(|e| e.status == CorrespondenceStatus::MissingInTarget)
            .count();
        assert_eq!(missing, 1);
        let extra = entries
            .iter()
            .filter(|e| e.status == CorrespondenceStatus::ExtraInTarget)
            .count();
        assert_eq!(extra, 0);
    }

    #[test]
    fn sparse_node_ids_are_enumerated() {
        let java = InMemoryGraphView::new(vec![callable(100, "run")], vec![]);
        let rust = InMemoryGraphView::new(vec![callable(200, "run")], vec![]);
        let entries =
            suggest_correspondences(&java, &GraphId("j".into()), &rust, &GraphId("r".into()));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].status, CorrespondenceStatus::Diverged);
        assert_eq!(entries[0].source_node.as_ref().map(|n| n.node.0), Some(100));
        assert_eq!(entries[0].target_node.as_ref().map(|n| n.node.0), Some(200));
    }

    #[test]
    fn merge_preserves_manual_over_suggested_same_id() {
        let manual = entry(
            "same",
            CorrespondenceMethod::Manual,
            CorrespondenceStatus::Ported,
            1.0,
        );
        let suggested = entry(
            "same",
            CorrespondenceMethod::NameHeuristic,
            CorrespondenceStatus::Diverged,
            0.6,
        );
        let merged = merge_correspondences(vec![manual.clone()], vec![suggested]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].method, CorrespondenceMethod::Manual);
        assert_eq!(merged[0].status, CorrespondenceStatus::Ported);
    }

    #[test]
    fn merge_marks_missing_manual_as_stale() {
        let manual = entry(
            "gone",
            CorrespondenceMethod::Manual,
            CorrespondenceStatus::Ported,
            1.0,
        );
        let merged = merge_correspondences(vec![manual], vec![]);
        assert_eq!(merged.len(), 1);
        assert!(merged[0].stale);
    }

    #[test]
    fn merge_replaces_old_heuristic_with_new_suggestions() {
        let old = entry(
            "old",
            CorrespondenceMethod::NameHeuristic,
            CorrespondenceStatus::Diverged,
            0.5,
        );
        let new = entry(
            "new",
            CorrespondenceMethod::SignatureHeuristic,
            CorrespondenceStatus::Diverged,
            0.9,
        );
        let merged = merge_correspondences(vec![old], vec![new.clone()]);
        assert_eq!(merged, vec![new]);
    }
}
