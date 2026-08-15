//! Offline heuristic reasoning (`s4 reason`).

use crate::commands::plugin;
use crate::workspace::Workspace;
use s4_core::{ArtifactId, Result, S4Error, MATURITY};
use s4_llm::{ContextBundle, HeuristicLlmProvider, ReasonIntent, ReasonPolicy, ReasonRequest};
use std::path::PathBuf;

/// Run the built-in heuristic reasoner and write a proposal JSON artifact.
pub fn run(intent: &str, prompt: Option<&str>, out: &str) -> Result<()> {
    plugin::ensure_registered(HeuristicLlmProvider::ID)?;
    let intent = parse_intent(intent)?;
    let ws = Workspace::open(".")?;
    let mut context = ContextBundle::default();
    if let Some(text) = prompt {
        let id = ArtifactId::from_content(text.as_bytes());
        context.artifacts.push(id);
        context.prompt = Some(text.to_string());
    }
    let provider = HeuristicLlmProvider;
    let proposal = provider.reason_sync(&ReasonRequest {
        intent,
        context,
        policy: ReasonPolicy {
            allow_network: false,
            ..ReasonPolicy::default()
        },
    })?;

    let out_path = resolve_out(ws.root(), out);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| S4Error::Storage(format!("failed to create {}: {e}", parent.display())))?;
    }
    let bytes = serde_json::to_vec_pretty(&proposal)
        .map_err(|e| S4Error::Storage(format!("serialize proposal: {e}")))?;
    std::fs::write(&out_path, &bytes)
        .map_err(|e| S4Error::Storage(format!("write {}: {e}", out_path.display())))?;

    println!(
        "proposal lifecycle={:?} kind={:?} claims={} → {}",
        proposal.lifecycle,
        proposal.kind,
        proposal.claims.len(),
        out_path.display()
    );
    for claim in &proposal.claims {
        println!("  [{:.2}] {}", claim.confidence, claim.statement);
    }
    println!("note: LLM / heuristic outputs are always Proposed — never ground truth ({MATURITY})");
    Ok(())
}

fn parse_intent(raw: &str) -> Result<ReasonIntent> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "explain" => Ok(ReasonIntent::Explain),
        "refactor" | "refactor_plan" => Ok(ReasonIntent::RefactorPlan),
        "map" | "map_requirement" | "requirement" => Ok(ReasonIntent::MapRequirement),
        "architecture" | "architecture_review" | "review" => Ok(ReasonIntent::ArchitectureReview),
        other => Err(S4Error::InvalidInput(format!(
            "unknown intent '{other}' (explain|refactor|map|architecture)"
        ))),
    }
}

fn resolve_out(root: &std::path::Path, out: &str) -> PathBuf {
    let p = PathBuf::from(out);
    if p.is_absolute() {
        p
    } else {
        root.join(p)
    }
}
