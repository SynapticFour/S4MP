use super::{
    child_by_field, collect_type_names, parse_tree, persist_usir_module, read_unit_source,
    UsirModuleBuilder,
};
use crate::{ParseContext, ParsePipeline, ParseUnit};
use s4_core::{ArtifactId, Result};
use tree_sitter::Node;

/// Tree-sitter frontend for Java sources (heuristic USIR extraction).
///
/// Call and reference edges use simple name matching. Cross-file callees are recorded as
/// unresolved calls for the graph linker. Signatures are captured when present on the AST.
#[derive(Clone, Debug, Default)]
pub struct JavaParser;

impl ParsePipeline for JavaParser {
    fn parse_unit(&self, unit: &ParseUnit, ctx: &mut ParseContext<'_>) -> Result<Vec<ArtifactId>> {
        let source = read_unit_source(unit)?;
        let module = extract_java_module(&source, &unit.path, ctx.source_root)?;
        let id = persist_usir_module(ctx.store, &module)?;
        Ok(vec![id])
    }
}

/// Extract a USIR module from Java source (no store I/O).
///
/// # Errors
///
/// Returns an error if parsing fails.
pub fn extract_java_module(
    source: &str,
    path: &str,
    source_root: &std::path::Path,
) -> Result<crate::UsirModule> {
    let mut parser = tree_sitter::Parser::new();
    let language: tree_sitter::Language = tree_sitter_java::LANGUAGE.into();
    let tree = parse_tree(&mut parser, &language, source)?;
    let mut builder = UsirModuleBuilder::new(path, source_root)?;
    let root = tree.root_node();
    walk_java_node(root, source, &mut builder, None);
    Ok(builder.build())
}

fn walk_java_node(
    node: Node<'_>,
    source: &str,
    builder: &mut UsirModuleBuilder,
    enclosing_type: Option<&str>,
) {
    match node.kind() {
        "class_declaration" | "interface_declaration" | "enum_declaration" => {
            let name = child_by_field(&node, "name", source).unwrap_or("<anonymous>");
            builder.add_type(name);
            walk_java_children(node, source, builder, Some(name));
            return;
        },
        "method_declaration" => {
            let name = child_by_field(&node, "name", source).unwrap_or("<anonymous>");
            let signature = java_method_signature(name, node, source);
            let id = builder.add_callable(name, Some(signature));
            add_type_references(id, node, source, builder);
            if let Some(body) = node.child_by_field_name("body") {
                if let Ok(body_text) = body.utf8_text(source.as_bytes()) {
                    builder.defer_calls(id, body_text.to_string());
                }
            }
        },
        "constructor_declaration" => {
            let name = enclosing_type.unwrap_or("constructor");
            let signature = java_method_signature(name, node, source);
            let id = builder.add_callable(name, Some(signature));
            add_type_references(id, node, source, builder);
            if let Some(body) = node.child_by_field_name("body") {
                if let Ok(body_text) = body.utf8_text(source.as_bytes()) {
                    builder.defer_calls(id, body_text.to_string());
                }
            }
        },
        "field_declaration" | "constant_declaration" => {
            collect_field_symbols(node, source, builder);
        },
        _ => {},
    }

    if !matches!(
        node.kind(),
        "class_declaration" | "interface_declaration" | "enum_declaration"
    ) {
        walk_java_children(node, source, builder, enclosing_type);
    }
}

fn java_method_signature(name: &str, node: Node<'_>, source: &str) -> String {
    let params = node
        .child_by_field_name("parameters")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("()");
    let ret = node
        .child_by_field_name("type")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("void");
    format!("{name}{params}:{ret}")
}

fn walk_java_children(
    node: Node<'_>,
    source: &str,
    builder: &mut UsirModuleBuilder,
    enclosing_type: Option<&str>,
) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_java_node(cursor.node(), source, builder, enclosing_type);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn collect_field_symbols(node: Node<'_>, source: &str, builder: &mut UsirModuleBuilder) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == "variable_declarator" {
                if let Some(name) = child_by_field(&child, "name", source) {
                    builder.add_symbol(name);
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
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
class Example {
    void caller() {
        callee();
    }
    void callee() {
    }
}
";
        let source_root = std::env::temp_dir();
        let module = extract_java_module(source, "Example.java", &source_root).unwrap();
        assert!(
            has_calls_relation(&module, "caller", "callee"),
            "caller defined before callee must still produce a Calls edge after deferred resolution"
        );
    }

    #[test]
    fn method_signature_is_captured() {
        let source = r"
class Example {
    int add(int a, int b) { return a + b; }
}
";
        let source_root = std::env::temp_dir();
        let module = extract_java_module(source, "Example.java", &source_root).unwrap();
        let add = module
            .entities
            .iter()
            .find(|e| e.name == "add")
            .expect("add");
        let sig = add.signature.as_deref().expect("signature");
        assert!(sig.contains("add"), "{sig}");
        assert!(sig.contains("int"), "{sig}");
    }

    #[test]
    fn cross_file_call_is_unresolved() {
        let source = r"
class Example {
    void caller() {
        scale(2);
    }
}
";
        let source_root = std::env::temp_dir();
        let module = extract_java_module(source, "Example.java", &source_root).unwrap();
        assert!(
            module
                .unresolved_calls
                .iter()
                .any(|c| c.callee_name == "scale"),
            "expected unresolved scale: {:?}",
            module.unresolved_calls
        );
    }
}
