//! Named symbol spans (functions, methods, types) for change attribution.
//!
//! The landing gate maps each input's changed lines onto these spans so two
//! agents editing the same function in one file can be told apart from two
//! agents editing different functions of that file.

use std::path::Path;

use crate::language;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SymbolSpan {
    /// Qualified with enclosing symbols, e.g. `Cart.total` or `impl Cart.total`.
    pub name: String,
    /// 1-based inclusive line range.
    pub start_line: usize,
    pub end_line: usize,
}

/// Symbol spans for a supported language; empty for unknown languages or
/// sources that do not parse.
pub fn symbol_spans(path: &Path, source: &str) -> Vec<SymbolSpan> {
    let Some(lang) = language::detect(path) else {
        return Vec::new();
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&lang.grammar).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    walk(tree.root_node(), source, lang.id, "", &mut out);
    out
}

/// Innermost symbol containing `line` (1-based), if any.
pub fn innermost_symbol(spans: &[SymbolSpan], line: usize) -> Option<&SymbolSpan> {
    spans
        .iter()
        .filter(|s| s.start_line <= line && line <= s.end_line)
        .min_by_key(|s| s.end_line - s.start_line)
}

fn walk(
    node: tree_sitter::Node<'_>,
    source: &str,
    lang: &str,
    prefix: &str,
    out: &mut Vec<SymbolSpan>,
) {
    let mut next_prefix = prefix.to_string();
    if let Some(name) = symbol_name(node, source, lang) {
        let qualified = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}.{name}")
        };
        out.push(SymbolSpan {
            name: qualified.clone(),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
        });
        next_prefix = qualified;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, source, lang, &next_prefix, out);
    }
}

fn text<'a>(node: tree_sitter::Node<'_>, source: &'a str) -> Option<&'a str> {
    node.utf8_text(source.as_bytes())
        .ok()
        .filter(|s| !s.is_empty())
}

fn named_field(node: tree_sitter::Node<'_>, source: &str, field: &str) -> Option<String> {
    node.child_by_field_name(field)
        .and_then(|n| text(n, source))
        .map(str::to_string)
}

fn symbol_name(node: tree_sitter::Node<'_>, source: &str, lang: &str) -> Option<String> {
    let kind = node.kind();
    match lang {
        "rust" => match kind {
            "function_item" | "struct_item" | "enum_item" | "trait_item" | "mod_item"
            | "union_item" | "macro_definition" => named_field(node, source, "name"),
            "impl_item" => named_field(node, source, "type").map(|t| format!("impl {t}")),
            _ => None,
        },
        "typescript" | "javascript" => match kind {
            "function_declaration"
            | "generator_function_declaration"
            | "class_declaration"
            | "method_definition"
            | "interface_declaration"
            | "type_alias_declaration"
            | "enum_declaration" => named_field(node, source, "name"),
            // `const total = () => …` and `const total = function () {…}`
            "variable_declarator" => {
                let value = node.child_by_field_name("value")?;
                if matches!(
                    value.kind(),
                    "arrow_function" | "function_expression" | "function" | "class"
                ) {
                    named_field(node, source, "name")
                } else {
                    None
                }
            }
            _ => None,
        },
        "python" => match kind {
            "function_definition" | "class_definition" => named_field(node, source, "name"),
            _ => None,
        },
        "go" => match kind {
            "function_declaration" | "method_declaration" | "type_spec" => {
                named_field(node, source, "name")
            }
            _ => None,
        },
        "java" => match kind {
            "method_declaration"
            | "class_declaration"
            | "interface_declaration"
            | "constructor_declaration"
            | "enum_declaration"
            | "record_declaration" => named_field(node, source, "name"),
            _ => None,
        },
        "c" | "cpp" => match kind {
            "function_definition" => {
                let declarator = node.child_by_field_name("declarator")?;
                let raw = text(declarator, source)?;
                let head = raw.split('(').next().unwrap_or(raw).trim();
                let name = head
                    .rsplit(|c: char| c.is_whitespace() || c == '*' || c == '&')
                    .next()?;
                (!name.is_empty()).then(|| name.to_string())
            }
            "class_specifier" | "struct_specifier" | "namespace_definition" => {
                named_field(node, source, "name")
            }
            _ => None,
        },
        "ruby" => match kind {
            "method" | "singleton_method" | "class" | "module" => named_field(node, source, "name"),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn javascript_functions_methods_and_arrow_consts() {
        let src = "function add(a, b) {\n  return a + b;\n}\n\nclass Cart {\n  total() {\n    return 1;\n  }\n}\n\nconst fmt = (n) => `${n}`;\n";
        let spans = symbol_spans(Path::new("m.js"), src);
        let names: Vec<_> = spans.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["add", "Cart", "Cart.total", "fmt"]);
        assert_eq!(innermost_symbol(&spans, 7).unwrap().name, "Cart.total");
        assert_eq!(innermost_symbol(&spans, 2).unwrap().name, "add");
        assert!(innermost_symbol(&spans, 4).is_none());
    }

    #[test]
    fn rust_impl_methods_are_qualified() {
        let src = "struct Cart;\nimpl Cart {\n    fn total(&self) -> u32 {\n        1\n    }\n}\n";
        let spans = symbol_spans(Path::new("lib.rs"), src);
        assert!(spans
            .iter()
            .any(|s| s.name == "impl Cart.total" && s.start_line == 3));
    }

    #[test]
    fn python_and_unknown_languages() {
        let spans = symbol_spans(
            Path::new("a.py"),
            "class A:\n    def go(self):\n        pass\n",
        );
        assert_eq!(
            spans.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            ["A", "A.go"]
        );
        assert!(symbol_spans(Path::new("notes.txt"), "fn x() {}").is_empty());
    }
}
