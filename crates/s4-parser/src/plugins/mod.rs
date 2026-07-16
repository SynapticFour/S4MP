//! Tree-sitter language frontends (v1 heuristic USIR lowering).

mod java;
mod rust;

pub use java::JavaParser;
pub use rust::RustParser;

use crate::usir::{UsirEntity, UsirEntityKind, UsirModule, UsirRelation, UsirRelationKind};
use crate::{ParseContext, ParsePipeline, ParseUnit};
use s4_core::{Result, S4Error, SchemaVersion};
use s4_storage::{Artifact, ArtifactKind, StoreWriter};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Builder for a single-file [`UsirModule`].
#[derive(Clone, Debug)]
pub struct UsirModuleBuilder {
    module_name: String,
    next_id: u64,
    entities: Vec<UsirEntity>,
    relations: Vec<UsirRelation>,
    callable_ids: HashMap<String, u64>,
    type_ids: HashMap<String, u64>,
    deferred_calls: Vec<(u64, String)>,
}

impl UsirModuleBuilder {
    /// Create a builder for a module named by `path` relative to `source_root`.
    ///
    /// # Errors
    ///
    /// Returns an error if the relative path cannot be computed.
    pub fn new(path: &str, source_root: &Path) -> Result<Self> {
        let module_name = module_name_from_path(path, source_root)?;
        let entities = vec![UsirEntity {
            id: 0,
            kind: UsirEntityKind::Module,
            name: module_name.clone(),
        }];
        Ok(Self {
            module_name,
            next_id: 1,
            entities,
            relations: Vec::new(),
            callable_ids: HashMap::new(),
            type_ids: HashMap::new(),
            deferred_calls: Vec::new(),
        })
    }

    /// Add a type entity (class, interface, enum, struct, trait, …).
    pub fn add_type(&mut self, name: &str) -> u64 {
        self.add_entity(name, UsirEntityKind::Type)
    }

    /// Add a callable entity (method, function, constructor, …).
    pub fn add_callable(&mut self, name: &str) -> u64 {
        let id = self.add_entity(name, UsirEntityKind::Callable);
        self.callable_ids.insert(name.to_string(), id);
        id
    }

    /// Add a symbol entity (field, const, static, …).
    pub fn add_symbol(&mut self, name: &str) -> u64 {
        self.add_entity(name, UsirEntityKind::Symbol)
    }

    fn add_entity(&mut self, name: &str, kind: UsirEntityKind) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        if kind == UsirEntityKind::Type {
            self.type_ids.insert(name.to_string(), id);
        }
        self.entities.push(UsirEntity {
            id,
            kind,
            name: name.to_string(),
        });
        self.relations.push(UsirRelation {
            from: 0,
            to: id,
            kind: UsirRelationKind::Defines,
        });
        id
    }

    /// Record a `References` edge when the target type name is known in this module.
    pub fn reference_type(&mut self, from: u64, type_name: &str) {
        if let Some(&to) = self.type_ids.get(type_name) {
            self.relations.push(UsirRelation {
                from,
                to,
                kind: UsirRelationKind::References,
            });
        }
    }

    /// Defer heuristic call detection for `caller_id` until [`Self::build`].
    ///
    /// Bodies are resolved against the full `callable_ids` map after all callables
    /// in the module have been registered.
    pub fn defer_calls(&mut self, caller_id: u64, body: String) {
        self.deferred_calls.push((caller_id, body));
    }

    fn resolve_deferred_calls(&mut self) {
        let deferred = std::mem::take(&mut self.deferred_calls);
        for (caller_id, body) in deferred {
            self.add_heuristic_calls(caller_id, &body);
        }
    }

    /// Heuristic call detection: `callee(` substring in `body` maps to known callables.
    ///
    /// v1 only — no overload resolution, imports, or method dispatch semantics.
    pub fn add_heuristic_calls(&mut self, caller_id: u64, body: &str) {
        for (name, &callee_id) in &self.callable_ids {
            if caller_id == callee_id {
                continue;
            }
            if contains_call(body, name) {
                self.relations.push(UsirRelation {
                    from: caller_id,
                    to: callee_id,
                    kind: UsirRelationKind::Calls,
                });
            }
        }
    }

    /// Finalize the module graph.
    #[must_use]
    pub fn build(mut self) -> UsirModule {
        self.resolve_deferred_calls();
        UsirModule {
            name: self.module_name,
            entities: self.entities,
            relations: self.relations,
        }
    }
}

/// Load source text for a parse unit.
///
/// Priority: inline `source_text`, then filesystem `path`. `content` artifact IDs are reserved
/// for a future blob artifact kind.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn read_unit_source(unit: &ParseUnit) -> Result<String> {
    if let Some(text) = &unit.source_text {
        return Ok(text.clone());
    }
    std::fs::read_to_string(&unit.path)
        .map_err(|e| S4Error::Other(format!("failed to read {}: {e}", unit.path)))
}

/// Persist a USIR module artifact and return its content address.
///
/// # Errors
///
/// Returns an error if serialization or storage fails.
pub fn persist_usir_module(
    store: &mut dyn StoreWriter,
    module: &UsirModule,
) -> Result<s4_core::ArtifactId> {
    let payload = serde_json::to_value(module)
        .map_err(|e| S4Error::Other(format!("failed to serialize USIR module: {e}")))?;
    let artifact = Artifact {
        kind: ArtifactKind::UsirModule,
        schema_version: SchemaVersion::CURRENT,
        payload,
    };
    store.write(&artifact)
}

/// Parse with tree-sitter and map failures to [`S4Error`].
///
/// # Errors
///
/// Returns an error if the parser cannot be configured or parsing fails.
pub fn parse_tree(
    parser: &mut tree_sitter::Parser,
    language: &tree_sitter::Language,
    source: &str,
) -> Result<tree_sitter::Tree> {
    parser
        .set_language(language)
        .map_err(|e| S4Error::Other(format!("failed to set tree-sitter language: {e}")))?;
    parser
        .parse(source, None)
        .ok_or_else(|| S4Error::Other("tree-sitter returned no syntax tree".to_string()))
}

/// Extract child node text for a field name when present.
#[must_use]
pub fn child_by_field<'a>(
    node: &tree_sitter::Node<'a>,
    field: &str,
    source: &'a str,
) -> Option<&'a str> {
    node.child_by_field_name(field)
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
}

/// Collect descendant node texts for nodes matching any of `kinds`.
#[must_use]
pub fn collect_identifiers(
    node: tree_sitter::Node<'_>,
    source: &str,
    kinds: &[&str],
) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = node.walk();
    collect_identifiers_recursive(node, source, kinds, &mut out, &mut cursor);
    out
}

fn collect_identifiers_recursive(
    node: tree_sitter::Node<'_>,
    source: &str,
    kinds: &[&str],
    out: &mut Vec<String>,
    cursor: &mut tree_sitter::TreeCursor<'_>,
) {
    if kinds.contains(&node.kind()) {
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            out.push(text.to_string());
        }
    }
    if cursor.goto_first_child() {
        loop {
            collect_identifiers_recursive(cursor.node(), source, kinds, out, cursor);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

/// Collect unique type-like identifiers under `node` (heuristic v1).
#[must_use]
pub fn collect_type_names(node: tree_sitter::Node<'_>, source: &str) -> Vec<String> {
    let raw = collect_identifiers(
        node,
        source,
        &[
            "type_identifier",
            "scoped_type_identifier",
            "generic_type",
            "integral_type",
            "floating_point_type",
            "boolean_type",
            "void_type",
            "type",
            "primitive_type",
        ],
    );
    dedupe_preserve_order(raw)
}

fn module_name_from_path(path: &str, source_root: &Path) -> Result<String> {
    let path_buf = PathBuf::from(path);
    let relative = if path_buf.is_absolute() {
        path_buf.strip_prefix(source_root).map_err(|_| {
            S4Error::Other(format!(
                "path {} is not under source root {}",
                path,
                source_root.display()
            ))
        })?
    } else {
        path_buf.as_path()
    };
    Ok(relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn contains_call(body: &str, callee: &str) -> bool {
    let needle = format!("{callee}(");
    let bytes = body.as_bytes();
    let mut start = 0;
    while let Some(pos) = body[start..].find(&needle) {
        let abs_pos = start + pos;
        let boundary_ok = abs_pos == 0
            || !(bytes[abs_pos - 1].is_ascii_alphanumeric() || bytes[abs_pos - 1] == b'_');
        if boundary_ok {
            return true;
        }
        start = abs_pos + 1;
    }
    false
}

fn dedupe_preserve_order(names: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    names
        .into_iter()
        .filter(|n| seen.insert(n.clone()))
        .collect()
}

/// Parse units sequentially (correctness before parallelism in v1).
///
/// # Errors
///
/// Returns an error if any unit fails to parse.
pub fn parse_all_sequential<P: ParsePipeline + ?Sized>(
    pipeline: &P,
    units: &[ParseUnit],
    ctx: &mut ParseContext<'_>,
) -> Result<Vec<s4_core::ArtifactId>> {
    let mut ids = Vec::with_capacity(units.len());
    for unit in units {
        ids.extend(pipeline.parse_unit(unit, ctx)?);
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_call_detects_simple_invocation() {
        assert!(contains_call("foo(bar);", "foo"));
        assert!(!contains_call("foo bar;", "foo"));
    }

    #[test]
    fn contains_call_rejects_substring_false_positive() {
        assert!(!contains_call("reget(x);", "get"));
    }

    #[test]
    fn contains_call_finds_real_match_among_similar_identifiers() {
        assert!(contains_call("x = get(1) + reget(2);", "get"));
    }
}
