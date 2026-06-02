use pytxo_core::FidelityTier;
use tree_sitter::Node;

const BODY_PLACEHOLDER: &str = " { /* … */ }\n";

pub fn emit_skeleton(root: Node<'_>, source: &str, tier: FidelityTier, lang: &str) -> String {
    let mut out = String::new();
    walk_node(root, source, tier, lang, &mut out, 0);
    out
}

fn walk_node(
    node: Node<'_>,
    source: &str,
    tier: FidelityTier,
    lang: &str,
    out: &mut String,
    depth: u8,
) {
    match lang {
        "rust" => emit_rust(node, source, tier, out, depth),
        "javascript" | "typescript" => emit_js(node, source, tier, out, depth),
        "python" => emit_python(node, source, tier, out, depth),
        "go" => emit_go(node, source, tier, out, depth),
        _ => append_range(out, source, node.start_byte(), node.end_byte()),
    }
}

fn emit_rust(node: Node<'_>, source: &str, tier: FidelityTier, out: &mut String, depth: u8) {
    match node.kind() {
        "source_file" | "declaration_list" => walk_children(node, source, tier, "rust", out, depth),
        "use_declaration"
        | "extern_crate_declaration"
        | "inner_attribute_item"
        | "outer_attribute_item" => {
            append_line(out, source, node);
        }
        "function_item" | "function_signature_item" => {
            emit_before_body(node, source, tier, out, "body");
        }
        "struct_item" | "enum_item" | "union_item" | "trait_item" | "type_item" | "const_item"
        | "static_item" | "mod_item" => {
            if tier == FidelityTier::Medium {
                append_line(out, source, node);
            } else {
                emit_header_only(node, source, out);
            }
        }
        "impl_item" => emit_before_body(node, source, tier, out, "body"),
        "macro_definition" if tier == FidelityTier::Medium => append_line(out, source, node),
        _ if depth == 0 => walk_children(node, source, tier, "rust", out, depth + 1),
        _ => {}
    }
}

fn emit_js(node: Node<'_>, source: &str, tier: FidelityTier, out: &mut String, depth: u8) {
    match node.kind() {
        "program" => walk_children(node, source, tier, "javascript", out, depth),
        "import_statement" | "import_clause" | "export_statement" => append_line(out, source, node),
        "function_declaration"
        | "method_definition"
        | "arrow_function"
        | "class_declaration"
        | "interface_declaration"
        | "type_alias_declaration"
        | "enum_declaration" => {
            emit_before_body(node, source, tier, out, "body");
        }
        "lexical_declaration" | "variable_declaration" if tier == FidelityTier::Medium => {
            append_line(out, source, node);
        }
        _ if depth == 0 => walk_children(node, source, tier, "javascript", out, depth + 1),
        _ => {}
    }
}

fn emit_python(node: Node<'_>, source: &str, tier: FidelityTier, out: &mut String, depth: u8) {
    match node.kind() {
        "module" => walk_children(node, source, tier, "python", out, depth),
        "import_statement" | "import_from_statement" | "future_import_statement" => {
            append_line(out, source, node);
        }
        "function_definition" | "class_definition" => {
            emit_before_body(node, source, tier, out, "body");
        }
        "decorated_definition" => {
            if tier == FidelityTier::Medium {
                append_line(out, source, node);
            } else {
                emit_header_only(node, source, out);
            }
        }
        _ if depth == 0 => walk_children(node, source, tier, "python", out, depth + 1),
        _ => {}
    }
}

fn emit_go(node: Node<'_>, source: &str, tier: FidelityTier, out: &mut String, depth: u8) {
    match node.kind() {
        "source_file" => walk_children(node, source, tier, "go", out, depth),
        "import_declaration" | "import_spec" | "package_clause" => append_line(out, source, node),
        "function_declaration" | "method_declaration" => {
            emit_before_body(node, source, tier, out, "body");
        }
        "type_declaration" | "const_declaration" | "var_declaration" => {
            if tier == FidelityTier::Medium {
                append_line(out, source, node);
            } else {
                emit_header_only(node, source, out);
            }
        }
        _ if depth == 0 => walk_children(node, source, tier, "go", out, depth + 1),
        _ => {}
    }
}

fn walk_children(
    node: Node<'_>,
    source: &str,
    tier: FidelityTier,
    lang: &str,
    out: &mut String,
    depth: u8,
) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_node(cursor.node(), source, tier, lang, out, depth + 1);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn emit_before_body(
    node: Node<'_>,
    source: &str,
    tier: FidelityTier,
    out: &mut String,
    body_field: &str,
) {
    if tier == FidelityTier::High {
        append_line(out, source, node);
        return;
    }
    if let Some(body) = node.child_by_field_name(body_field) {
        append_range(out, source, node.start_byte(), body.start_byte());
        out.push_str(BODY_PLACEHOLDER);
    } else {
        append_line(out, source, node);
    }
}

fn emit_header_only(node: Node<'_>, source: &str, out: &mut String) {
    if let Some(body) = node.child_by_field_name("body") {
        append_range(out, source, node.start_byte(), body.start_byte());
        out.push_str(BODY_PLACEHOLDER);
        return;
    }
    if let Some(value) = node.child_by_field_name("value") {
        append_range(out, source, node.start_byte(), value.start_byte());
        out.push_str(" = /* … */;\n");
        return;
    }
    append_line(out, source, node);
}

fn append_line(out: &mut String, source: &str, node: Node<'_>) {
    append_range(out, source, node.start_byte(), node.end_byte());
    if !out.ends_with('\n') {
        out.push('\n');
    }
}

fn append_range(out: &mut String, source: &str, start: usize, end: usize) {
    if start <= end && end <= source.len() {
        out.push_str(&source[start..end]);
    }
}
