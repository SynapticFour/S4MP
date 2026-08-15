use super::{
    child_by_field, collect_type_names, parse_tree, persist_usir_module, read_unit_source,
    UsirModuleBuilder,
};
use crate::{ParseContext, ParsePipeline, ParseUnit};
use s4_core::{ArtifactId, Result};
use tree_sitter::Node;

/// Tree-sitter frontend for Rust sources (heuristic USIR extraction).
///
/// Call edges come from `call_expression` AST nodes. Unresolved names are linked
/// across modules during graph lowering. Signatures are captured when present.
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
    walk_rust_node(tree.root_node(), source, &mut builder, None);
    Ok(builder.build())
}

fn walk_rust_node(
    node: Node<'_>,
    source: &str,
    builder: &mut UsirModuleBuilder,
    impl_type: Option<&str>,
) {
    match node.kind() {
        "impl_item" => {
            let ty = child_by_field(&node, "type", source).map(str::to_string);
            walk_rust_children(node, source, builder, ty.as_deref());
            return;
        },
        "struct_item" | "enum_item" | "trait_item" | "union_item" => {
            if let Some(name) = child_by_field(&node, "name", source) {
                builder.add_type(name);
            }
        },
        "function_item" => {
            if let Some(name) = child_by_field(&node, "name", source) {
                let signature = rust_fn_signature(name, node, source);
                let qualified = impl_type.map(|ty| format!("{ty}::{name}"));
                let id = builder.add_callable(name, qualified.as_deref(), Some(signature));
                add_type_references(id, node, source, builder);
                if let Some(body) = node.child_by_field_name("body") {
                    builder.defer_calls(id, super::collect_rust_callee_names(body, source));
                }
            }
            return;
        },
        "const_item" | "static_item" | "field_declaration" => {
            if let Some(name) = child_by_field(&node, "name", source) {
                builder.add_symbol(name);
            }
        },
        _ => {},
    }

    walk_rust_children(node, source, builder, impl_type);
}

fn walk_rust_children(
    node: Node<'_>,
    source: &str,
    builder: &mut UsirModuleBuilder,
    impl_type: Option<&str>,
) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_rust_node(cursor.node(), source, builder, impl_type);
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

fn add_type_references(
    from: crate::UsirLocalId,
    node: Node<'_>,
    source: &str,
    builder: &mut UsirModuleBuilder,
) {
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
        assert!(add.qualified.is_none());
    }

    #[test]
    fn impl_method_is_qualified() {
        let source = r"
pub struct Calculator;
impl Calculator {
    pub fn add(a: i32, b: i32) -> i32 { a + b }
}
fn helper(x: i32) -> i32 { x }
";
        let source_root = std::env::temp_dir();
        let module = extract_rust_module(source, "example.rs", &source_root).unwrap();
        let add = module
            .entities
            .iter()
            .find(|e| e.name == "add")
            .expect("add");
        assert_eq!(add.qualified.as_deref(), Some("Calculator::add"));
        let helper = module
            .entities
            .iter()
            .find(|e| e.name == "helper")
            .expect("helper");
        assert!(helper.qualified.is_none());
    }

    #[test]
    fn comments_and_strings_are_not_calls() {
        let source = r#"
fn caller() {
    callee();
    // callee();
    let _ = "callee()";
}
fn callee() {}
"#;
        let source_root = std::env::temp_dir();
        let module = extract_rust_module(source, "example.rs", &source_root).unwrap();
        assert!(has_calls_relation(&module, "caller", "callee"));
        let caller = module
            .entities
            .iter()
            .find(|e| e.name == "caller")
            .map(|e| e.id)
            .unwrap();
        let n = module
            .relations
            .iter()
            .filter(|r| r.from == caller && r.kind == UsirRelationKind::Calls)
            .count();
        assert_eq!(n, 1);
    }
}
