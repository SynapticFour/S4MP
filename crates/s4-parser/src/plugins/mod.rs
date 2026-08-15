//! Tree-sitter language frontends (v1 heuristic USIR lowering).

mod java;
mod rust;

pub use java::{extract_java_module, JavaParser};
pub use rust::{extract_rust_module, RustParser};

use crate::usir::{
    UnresolvedCall, UsirEntity, UsirEntityKind, UsirLocalId, UsirModule, UsirRelation,
    UsirRelationKind,
};
use crate::{ParseContext, ParsePipeline, ParseUnit};
use s4_core::{ArtifactId, Result, S4Error, SchemaVersion};
use s4_storage::{Artifact, ArtifactKind, Store, StoreReader, StoreWriter};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Builder for a single-file [`UsirModule`].
#[derive(Clone, Debug)]
pub struct UsirModuleBuilder {
    module_name: String,
    next_id: u64,
    entities: Vec<UsirEntity>,
    relations: Vec<UsirRelation>,
    callable_ids: HashMap<String, Vec<UsirLocalId>>,
    type_ids: HashMap<String, UsirLocalId>,
    deferred_calls: Vec<(UsirLocalId, Vec<String>)>,
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
            id: UsirLocalId(0),
            kind: UsirEntityKind::Module,
            name: module_name.clone(),
            signature: None,
            qualified: None,
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
    pub fn add_type(&mut self, name: &str) -> UsirLocalId {
        self.add_entity(name, UsirEntityKind::Type, None, None)
    }

    /// Add a callable entity (method, function, constructor, …).
    ///
    /// `name` is the simple identifier used for heuristic matching. `qualified` is
    /// the report label (`Calculator.add`, `Calculator::add`) when known.
    pub fn add_callable(
        &mut self,
        name: &str,
        qualified: Option<&str>,
        signature: Option<String>,
    ) -> UsirLocalId {
        let id = self.add_entity(
            name,
            UsirEntityKind::Callable,
            signature,
            qualified.map(str::to_string),
        );
        self.callable_ids
            .entry(name.to_string())
            .or_default()
            .push(id);
        id
    }

    /// Add a symbol entity (field, const, static, …).
    pub fn add_symbol(&mut self, name: &str) -> UsirLocalId {
        self.add_entity(name, UsirEntityKind::Symbol, None, None)
    }

    fn add_entity(
        &mut self,
        name: &str,
        kind: UsirEntityKind,
        signature: Option<String>,
        qualified: Option<String>,
    ) -> UsirLocalId {
        let id = UsirLocalId(self.next_id);
        self.next_id += 1;
        if kind == UsirEntityKind::Type {
            self.type_ids.insert(name.to_string(), id);
        }
        self.entities.push(UsirEntity {
            id,
            kind,
            name: name.to_string(),
            signature,
            qualified,
        });
        self.relations.push(UsirRelation {
            from: UsirLocalId(0),
            to: id,
            kind: UsirRelationKind::Defines,
        });
        id
    }

    /// Record a `References` edge when the target type name is known in this module.
    pub fn reference_type(&mut self, from: UsirLocalId, type_name: &str) {
        if let Some(&to) = self.type_ids.get(type_name) {
            self.relations.push(UsirRelation {
                from,
                to,
                kind: UsirRelationKind::References,
            });
        }
    }

    /// Defer call resolution for `caller_id` until [`Self::build`].
    ///
    /// Callee names come from Tree-sitter `method_invocation` / `call_expression`
    /// nodes (not a text scan of the body). Names with no local callable become
    /// [`UsirModule::unresolved_calls`] for cross-module linking.
    pub fn defer_calls(&mut self, caller_id: UsirLocalId, callee_names: Vec<String>) {
        self.deferred_calls.push((caller_id, callee_names));
    }

    fn resolve_deferred_calls(&mut self) -> Vec<UnresolvedCall> {
        let deferred = std::mem::take(&mut self.deferred_calls);
        let mut unresolved = Vec::new();
        for (caller_id, names) in deferred {
            unresolved.extend(self.add_heuristic_calls(caller_id, names));
        }
        unresolved
    }

    /// Map callee names to local callables; unknown names become unresolved calls.
    ///
    /// Overloads of the same name all receive a `Calls` edge.
    pub fn add_heuristic_calls(
        &mut self,
        caller_id: UsirLocalId,
        names: Vec<String>,
    ) -> Vec<UnresolvedCall> {
        let mut unresolved = Vec::new();
        let mut seen_unresolved = HashSet::new();

        for name in names {
            if let Some(ids) = self.callable_ids.get(&name) {
                for &callee_id in ids {
                    if callee_id == caller_id {
                        continue;
                    }
                    self.relations.push(UsirRelation {
                        from: caller_id,
                        to: callee_id,
                        kind: UsirRelationKind::Calls,
                    });
                }
            } else if seen_unresolved.insert(name.clone()) {
                unresolved.push(UnresolvedCall {
                    from: caller_id,
                    callee_name: name,
                });
            }
        }
        unresolved
    }

    /// Finalize the module graph.
    #[must_use]
    pub fn build(mut self) -> UsirModule {
        let unresolved_calls = self.resolve_deferred_calls();
        UsirModule {
            name: self.module_name,
            entities: self.entities,
            relations: self.relations,
            unresolved_calls,
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
        .map_err(|e| S4Error::Storage(format!("failed to read {}: {e}", unit.path)))
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
        .map_err(|e| S4Error::Storage(format!("failed to serialize USIR module: {e}")))?;
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
    parser.set_language(language).map_err(|e| S4Error::Plugin {
        plugin_id: "tree-sitter".into(),
        message: format!("failed to set language: {e}"),
    })?;
    parser.parse(source, None).ok_or_else(|| S4Error::Plugin {
        plugin_id: "tree-sitter".into(),
        message: "parser returned no syntax tree".into(),
    })
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
            S4Error::InvalidInput(format!(
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

fn walk_skip_opaque(kind: &str) -> bool {
    matches!(
        kind,
        "line_comment"
            | "block_comment"
            | "string_literal"
            | "raw_string_literal"
            | "character_literal"
            | "char_literal"
            | "text_block"
    )
}

/// Collect callee identifiers from a Tree-sitter subtree (Java `method_invocation`).
pub(super) fn collect_java_callee_names(body: tree_sitter::Node<'_>, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    walk_java_callees(body, source, &mut names);
    dedupe_preserve_order(names)
}

fn walk_java_callees(node: tree_sitter::Node<'_>, source: &str, names: &mut Vec<String>) {
    if walk_skip_opaque(node.kind()) {
        return;
    }
    if node.kind() == "method_invocation" {
        if let Some(name) = child_by_field(&node, "name", source) {
            names.push(name.to_string());
        }
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_java_callees(cursor.node(), source, names);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// Collect callee identifiers from a Tree-sitter subtree (Rust `call_expression`).
pub(super) fn collect_rust_callee_names(body: tree_sitter::Node<'_>, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    walk_rust_callees(body, source, &mut names);
    dedupe_preserve_order(names)
}

fn walk_rust_callees(node: tree_sitter::Node<'_>, source: &str, names: &mut Vec<String>) {
    if walk_skip_opaque(node.kind()) {
        return;
    }
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            if let Some(name) = rust_callee_name(func, source) {
                names.push(name);
            }
        }
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_rust_callees(cursor.node(), source, names);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn rust_callee_name(func: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    match func.kind() {
        "identifier" => func.utf8_text(source.as_bytes()).ok().map(str::to_string),
        "field_expression" => child_by_field(&func, "field", source).map(str::to_string),
        "generic_function" => func
            .child_by_field_name("function")
            .and_then(|inner| rust_callee_name(inner, source)),
        "scoped_identifier" => {
            let mut last = None;
            let mut cursor = func.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "identifier" {
                        last = child.utf8_text(source.as_bytes()).ok().map(str::to_string);
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            last
        },
        _ => None,
    }
}

/// Extract a USIR module using the shipped in-process frontend for `language`.
///
/// Shipped languages: `java`, `rust`. This is the single dispatch table used by
/// `s4 graph build` — not a loadable plugin runtime.
///
/// # Errors
///
/// Returns an error if the language has no frontend or parsing fails.
pub fn extract_for_language(
    language: &str,
    source: &str,
    path: &str,
    source_root: &Path,
) -> Result<UsirModule> {
    match language {
        "java" => extract_java_module(source, path, source_root),
        "rust" => extract_rust_module(source, path, source_root),
        other => Err(S4Error::InvalidInput(format!(
            "no in-process frontend for language '{other}' (shipped: java, rust)"
        ))),
    }
}

/// Languages with a shipped in-process Tree-sitter frontend.
#[must_use]
pub fn shipped_parser_languages() -> &'static [&'static str] {
    &["java", "rust"]
}

fn dedupe_preserve_order(names: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    names
        .into_iter()
        .filter(|n| seen.insert(n.clone()))
        .collect()
}

/// Parse units sequentially and persist USIR artifacts.
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

/// Extracted USIR modules plus their persisted artifact IDs.
#[derive(Debug)]
pub struct ParsedModules {
    /// Content addresses of persisted USIR artifacts, parallel to [`Self::modules`].
    pub ids: Vec<s4_core::ArtifactId>,
    /// In-memory modules (same order as `ids`).
    pub modules: Vec<UsirModule>,
}

fn usir_cache_id(language: &str, module_name: &str, file_hash: &str) -> ArtifactId {
    let mut buf = Vec::with_capacity(32 + language.len() + module_name.len() + file_hash.len());
    buf.extend_from_slice(b"s4-usir-cache-v1\0");
    buf.extend_from_slice(language.as_bytes());
    buf.push(0);
    buf.extend_from_slice(module_name.as_bytes());
    buf.push(0);
    buf.extend_from_slice(file_hash.as_bytes());
    ArtifactId::from_content(&buf)
}

fn load_cached_usir(
    store: &dyn StoreReader,
    language: &str,
    module_name: &str,
    file_hash: &str,
) -> Result<Option<(ArtifactId, UsirModule)>> {
    let cache_id = usir_cache_id(language, module_name, file_hash);
    let Some(record) = store.read(&cache_id)? else {
        return Ok(None);
    };
    if record.kind != ArtifactKind::UsirCache {
        return Ok(None);
    }
    record.expect_current_schema()?;
    let Some(hex) = record
        .payload
        .get("usir_id")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(None);
    };
    let usir_id: ArtifactId = hex.parse()?;
    let Some(artifact) = store.read(&usir_id)? else {
        return Ok(None);
    };
    if artifact.kind != ArtifactKind::UsirModule {
        return Ok(None);
    }
    artifact.expect_current_schema()?;
    let module: UsirModule = serde_json::from_value(artifact.payload)
        .map_err(|e| S4Error::Storage(format!("failed to deserialize cached USIR: {e}")))?;
    Ok(Some((usir_id, module)))
}

fn index_usir_cache(
    store: &mut dyn StoreWriter,
    language: &str,
    module_name: &str,
    file_hash: &str,
    usir_id: ArtifactId,
) -> Result<()> {
    let cache_id = usir_cache_id(language, module_name, file_hash);
    let artifact = Artifact {
        kind: ArtifactKind::UsirCache,
        schema_version: SchemaVersion::CURRENT,
        payload: serde_json::json!({ "usir_id": usir_id.to_string() }),
    };
    store.write_at(cache_id, &artifact)
}

/// Extract USIR modules with bounded parallelism, reusing CAS entries keyed by
/// `(language, module path, file hash)` when [`ParseUnit::source_hash`] is set.
///
/// # Errors
///
/// Returns an error if any unit fails to parse or persist, or if a worker thread panics.
pub fn parse_all_parallel<F>(
    units: &[ParseUnit],
    source_root: &Path,
    store: &mut dyn Store,
    extract: F,
) -> Result<ParsedModules>
where
    F: Fn(&ParseUnit, &Path) -> Result<UsirModule> + Sync,
{
    let mut modules: Vec<Option<UsirModule>> = vec![None; units.len()];
    let mut ids: Vec<Option<ArtifactId>> = vec![None; units.len()];
    let mut pending = Vec::new();

    for (index, unit) in units.iter().enumerate() {
        if let Some(hash) = &unit.source_hash {
            let name = module_name_from_path(&unit.path, source_root)?;
            if let Some((id, module)) = load_cached_usir(store, &unit.language.0, &name, hash)? {
                ids[index] = Some(id);
                modules[index] = Some(module);
                continue;
            }
        }
        pending.push(index);
    }

    let extracted = extract_pending(units, source_root, &pending, &extract)?;
    for (index, module) in pending.iter().zip(extracted) {
        let id = persist_usir_module(store, &module)?;
        if let Some(hash) = &units[*index].source_hash {
            let name = module_name_from_path(&units[*index].path, source_root)?;
            index_usir_cache(store, &units[*index].language.0, &name, hash, id)?;
        }
        ids[*index] = Some(id);
        modules[*index] = Some(module);
    }

    let mut out_ids = Vec::with_capacity(units.len());
    let mut out_modules = Vec::with_capacity(units.len());
    for (id, module) in ids.into_iter().zip(modules) {
        let id = id.ok_or_else(|| S4Error::Storage("internal: missing USIR id slot".into()))?;
        let module =
            module.ok_or_else(|| S4Error::Storage("internal: missing USIR module slot".into()))?;
        out_ids.push(id);
        out_modules.push(module);
    }
    Ok(ParsedModules {
        ids: out_ids,
        modules: out_modules,
    })
}

fn extract_pending<F>(
    units: &[ParseUnit],
    source_root: &Path,
    pending: &[usize],
    extract: &F,
) -> Result<Vec<UsirModule>>
where
    F: Fn(&ParseUnit, &Path) -> Result<UsirModule> + Sync,
{
    if pending.is_empty() {
        return Ok(Vec::new());
    }
    let parallelism = std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get);
    let mut out = Vec::with_capacity(pending.len());
    if pending.len() <= 1 || parallelism <= 1 {
        for &index in pending {
            out.push(extract(&units[index], source_root)?);
        }
        return Ok(out);
    }
    for chunk in pending.chunks(parallelism) {
        std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|&index| scope.spawn(move || extract(&units[index], source_root)))
                .collect();
            for handle in handles {
                out.push(handle.join().unwrap_or_else(|_| {
                    Err(S4Error::Plugin {
                        plugin_id: "parser".into(),
                        message: "parse worker panicked".into(),
                    })
                })?);
            }
            Ok::<(), S4Error>(())
        })?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_calls_ignore_comments_and_strings() {
        let source = r#"
class Example {
    void caller() {
        callee();
        // callee();
        String s = "callee()";
    }
    void callee() {}
}
"#;
        let module = extract_java_module(source, "Example.java", Path::new(".")).unwrap();
        let caller_id = module
            .entities
            .iter()
            .find(|e| e.name == "caller")
            .map(|e| e.id)
            .unwrap();
        let target_id = module
            .entities
            .iter()
            .find(|e| e.name == "callee")
            .map(|e| e.id)
            .unwrap();
        let calls: Vec<_> = module
            .relations
            .iter()
            .filter(|r| r.from == caller_id && r.kind == UsirRelationKind::Calls)
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].to, target_id);
    }

    #[test]
    fn extract_for_language_rejects_unknown() {
        let err = extract_for_language("python", "", "x.py", Path::new(".")).unwrap_err();
        assert!(err.to_string().contains("python"));
    }

    #[test]
    fn substring_identifier_is_not_a_call() {
        let source = r"
class Example {
    void caller() { reget(1); }
    void get() {}
}
";
        let module = extract_java_module(source, "Example.java", Path::new(".")).unwrap();
        assert!(module
            .unresolved_calls
            .iter()
            .any(|c| c.callee_name == "reget"));
        let get_id = module
            .entities
            .iter()
            .find(|e| e.name == "get")
            .map(|e| e.id)
            .unwrap();
        assert!(!module
            .relations
            .iter()
            .any(|r| r.to == get_id && r.kind == UsirRelationKind::Calls));
    }

    #[test]
    fn parse_all_parallel_reuses_cas_by_file_hash() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let root = std::env::temp_dir().join(format!("s4-parse-cache-{n}"));
        let _ = std::fs::remove_dir_all(&root);
        let mut store = s4_storage::FileSystemStore::new(root.join("store")).expect("store");
        let unit = ParseUnit {
            path: "Example.java".into(),
            language: crate::LanguageId("java".into()),
            content: None,
            source_text: Some("class Example { void a() {} }".into()),
            source_hash: Some("abc123".into()),
        };
        let calls = AtomicUsize::new(0);
        let parsed = parse_all_parallel(
            std::slice::from_ref(&unit),
            Path::new("."),
            &mut store,
            |u, r| {
                calls.fetch_add(1, Ordering::SeqCst);
                extract_java_module(u.source_text.as_deref().unwrap_or(""), &u.path, r)
            },
        )
        .expect("first parse");
        assert_eq!(parsed.modules.len(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let parsed2 = parse_all_parallel(
            std::slice::from_ref(&unit),
            Path::new("."),
            &mut store,
            |u, r| {
                calls.fetch_add(1, Ordering::SeqCst);
                extract_java_module(u.source_text.as_deref().unwrap_or(""), &u.path, r)
            },
        )
        .expect("cached parse");
        assert_eq!(parsed2.ids, parsed.ids);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let _ = std::fs::remove_dir_all(&root);
    }
}
