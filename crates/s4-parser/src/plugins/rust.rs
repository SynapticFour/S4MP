use super::{
    child_by_field, collect_type_names, parse_tree, persist_usir_module, read_unit_source,
    UsirModuleBuilder,
};
use crate::{ParseContext, ParsePipeline, ParseUnit};
use s4_core::{ArtifactId, Result};
use tree_sitter::Node;

/// Tree-sitter frontend for Rust sources (heuristic USIR extraction).
///
/// Call edges use substring matching for `name(` within the same module; unresolved names
/// are linked across modules during graph lowering. Signatures are captured when present.
#[derive(Clone, Debug, Default)]
pub struct RustParser;

impl ParsePipeline for RustParser {
    fn parse_unit(&self, unit: &ParseUnit, ctx: &mut ParseContext<'_>) -> Result<Vec<ArtifactId>> {
        let source = read_unit_source(unit)?;
        let module = extract_rust_module(&source, &unit.path, ctx.source_root)?;
        let id = persist_usir_module(ctx.store, &module)?;
        Ok(vec![id])
    }
}

/// Extract a USIR module from Rust source (no store I/O).
///
/// # Errors
///
/// Returns an error if parsing fails.
pub fn extract_rust_module(
    source: &str,
    path: &str,
    source_root: &std::path::Path,
) -> Result<crate::UsirModule> {
    let mut parser = tree_sitter::Parser::new();
    let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
    let tree = parse_tree(&mut parser, &language, source)?;
    let mut builder = UsirModuleBuilder::new(path, source_root)?;
    walk_rust_node(tree.root_node(), source, &mut builder);
    Ok(builder.build())
}

fn walk_rust_node(node: Node<'_>, source: &str, builder: &mut UsirModuleBuilder) {
    match node.kind() {
        "struct_item" | "enum_item" | "trait_item" | "union_item" => {
            if let Some(name) = child_by_field(&node, "name", source) {
                builder.add_type(name);
            }
        },
        "function_item" => {
            if let Some(name) = child_by_field(&node, "name", source) {
                let signature = rust_fn_signature(name, node, source);
                let id = builder.add_callable(name, Some(signature));
                add_type_references(id, node, source, builder);
                if let Some(body) = node.child_by_field_name("body") {
                    if let Ok(body_text) = body.utf8_text(source.as_bytes()) {
                        builder.defer_calls(id, body_text.to_string());
                    }
                }
            }
        },
        "const_item" | "static_item" | "field_declaration" => {
            if let Some(name) = child_by_field(&node, "name", source) {
                builder.add_symbol(name);
            }
        },
        _ => {},
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_rust_node(cursor.node(), source, builder);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn rust_fn_signature(name: &str, node: Node<'_>, source: &str) -> String {
    let params = node
        .child_by_field_name("parameters")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("()");
    let ret = node
        .child_by_field_name("return_type")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("()");
    format!("{name}{params}->{ret}")
}

fn add_type_references(from: u64, node: Node<'_>, source: &str, builder: &mut UsirModuleBuilder) {
    for type_name in collect_type_names(node, source) {
        builder.reference_type(from, &type_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usir::{UsirEntityKind, UsirRelationKind};

    fn has_calls_relation(module: &crate::UsirModule, from_name: &str, to_name: &str) -> bool {
        let callable_id = |name: &str| {
            module
                .entities
                .iter()
                .find(|e| e.name == name && e.kind == UsirEntityKind::Callable)
                .map(|e| e.id)
        };
        let Some(from) = callable_id(from_name) else {
            return false;
        };
        let Some(to) = callable_id(to_name) else {
            return false;
        };
        module
            .relations
            .iter()
            .any(|r| r.from == from && r.to == to && r.kind == UsirRelationKind::Calls)
    }

    #[test]
    fn forward_call_in_document_order_is_detected() {
        let source = r"
fn caller() {
    callee();
}
fn callee() {}
";
        let source_root = std::env::temp_dir();
        let module = extract_rust_module(source, "example.rs", &source_root).unwrap();
        assert!(
            has_calls_relation(&module, "caller", "callee"),
            "caller defined before callee must still produce a Calls edge after deferred resolution"
        );
    }

    #[test]
    fn function_signature_is_captured() {
        let source = r"
fn add(a: i32, b: i32) -> i32 { a + b }
";
        let source_root = std::env::temp_dir();
        let module = extract_rust_module(source, "example.rs", &source_root).unwrap();
        let add = module
            .entities
            .iter()
            .find(|e| e.name == "add")
            .expect("add");
        let sig = add.signature.as_deref().expect("signature");
        assert!(sig.contains("add"), "{sig}");
        assert!(sig.contains("i32"), "{sig}");
    }
}
