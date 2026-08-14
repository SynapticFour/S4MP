//! Ordered analysis passes over CAS artifacts (ADR-013 Phase 2).

use s4_core::{ArtifactId, Result};
use s4_events::{EventKind, RecordingEventSink};
use std::collections::BTreeMap;

/// Outcome of a single pass execution.
#[derive(Clone, Debug, Default)]
pub struct PassOutcome {
    /// Human-readable notes for CLI progress.
    pub notes: Vec<String>,
    /// Named artifact IDs produced by this pass.
    pub artifacts: BTreeMap<String, ArtifactId>,
}

/// Shared context threaded through a [`PassPipeline`].
#[derive(Debug)]
pub struct PassContext<'a> {
    /// Optional event sink (CLI pipelines).
    pub events: Option<&'a RecordingEventSink>,
    /// Human-readable notes accumulated from pass outcomes.
    pub notes: Vec<String>,
    /// Artifacts accumulated across passes (key → id).
    pub artifacts: BTreeMap<String, ArtifactId>,
}

impl<'a> PassContext<'a> {
    /// Create an empty context.
    #[must_use]
    pub fn new(events: Option<&'a RecordingEventSink>) -> Self {
        Self {
            events,
            notes: Vec::new(),
            artifacts: BTreeMap::new(),
        }
    }

    /// Record an event when a sink is attached.
    pub fn emit(&self, kind: EventKind) {
        if let Some(sink) = self.events {
            sink.emit(kind, None);
        }
    }

    /// Merge pass artifacts and notes into the shared context.
    pub fn merge(&mut self, outcome: PassOutcome) {
        self.notes.extend(outcome.notes);
        self.artifacts.extend(outcome.artifacts);
    }
}

/// A composable analysis/materialization step (ADR-013 pass model).
pub trait Pass: Send + Sync {
    /// Stable pass name for logs and manifests.
    fn name(&self) -> &'static str;

    /// Execute the pass.
    ///
    /// # Errors
    ///
    /// Returns an error if the pass fails.
    fn run(&self, ctx: &mut PassContext<'_>) -> Result<PassOutcome>;
}

/// Ordered list of passes run left-to-right.
pub struct PassPipeline {
    passes: Vec<Box<dyn Pass>>,
}

impl PassPipeline {
    /// Create an empty pipeline.
    #[must_use]
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Append a pass.
    #[must_use]
    pub fn then(mut self, pass: impl Pass + 'static) -> Self {
        self.passes.push(Box::new(pass));
        self
    }

    /// Run all passes, merging artifacts into `ctx`.
    ///
    /// # Errors
    ///
    /// Returns the first pass error.
    pub fn run(&self, ctx: &mut PassContext<'_>) -> Result<()> {
        for pass in &self.passes {
            let outcome = pass.run(ctx)?;
            ctx.merge(outcome);
        }
        Ok(())
    }

    /// Pass names in execution order.
    #[must_use]
    pub fn names(&self) -> Vec<&'static str> {
        self.passes.iter().map(|p| p.name()).collect()
    }
}

impl Default for PassPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Canonical Phase 2 pass order for the porting slice (CLI `s4 analyze`).
pub const PORTING_PASS_ORDER: &[&str] = &["graph_build", "suggest_map", "diff_report"];

#[cfg(test)]
mod tests {
    use super::*;

    struct CountingPass {
        name: &'static str,
    }

    impl Pass for CountingPass {
        fn name(&self) -> &'static str {
            self.name
        }

        fn run(&self, ctx: &mut PassContext<'_>) -> Result<PassOutcome> {
            ctx.emit(EventKind::Extension(format!("pass:{}", self.name)));
            Ok(PassOutcome {
                notes: vec![self.name.to_string()],
                artifacts: BTreeMap::new(),
            })
        }
    }

    #[test]
    fn pipeline_runs_in_order() {
        let sink = RecordingEventSink::new();
        let pipeline = PassPipeline::new()
            .then(CountingPass {
                name: "physical_snapshot",
            })
            .then(CountingPass { name: "parse_usir" });
        let mut ctx = PassContext::new(Some(&sink));
        pipeline.run(&mut ctx).unwrap();
        assert_eq!(pipeline.names(), vec!["physical_snapshot", "parse_usir"]);
        assert_eq!(sink.len(), 2);
        assert_eq!(ctx.notes, vec!["physical_snapshot", "parse_usir"]);
    }
}
