use super::{
    child_by_field, collect_type_names, parse_tree, persist_usir_module, read_unit_source,
    UsirModuleBuilder,
};
use crate::{ParseContext, ParsePipeline, ParseUnit};
use s4_core::{ArtifactId, Result};
use tree_sitter::Node;

/// Tree-sitter frontend for Java sources (v1 heuristic USIR extraction).
///
/// This is **not** a full Java compiler frontend. Call and reference edges are inferred
/// with simple name matching inside the same file/module.
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

fn extract_java_module(
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
            let id = builder.add_callable(name);
            add_type_references(id, node, source, builder);
            if let Some(body) = node.child_by_field_name("body") {
                if let Ok(body_text) = body.utf8_text(source.as_bytes()) {
                    builder.add_heuristic_calls(id, body_text);
                }
            }
        },
        "constructor_declaration" => {
            let name = enclosing_type.unwrap_or("constructor");
            let id = builder.add_callable(name);
            add_type_references(id, node, source, builder);
            if let Some(body) = node.child_by_field_name("body") {
                if let Ok(body_text) = body.utf8_text(source.as_bytes()) {
                    builder.add_heuristic_calls(id, body_text);
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
