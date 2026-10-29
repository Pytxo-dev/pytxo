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
    /// Called as `a.name`: inside another symbol, such as a method, or a Go
    /// method, which its name alone does not show.
    #[serde(skip)]
    pub member: bool,
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

/// Whether `symbol_spans` reads a file at `path`: its extension names a
/// language with a grammar here. Lets a caller skip reading a file whose
/// definitions it could not get anyway.
pub fn parses(path: &Path) -> bool {
    language::detect(path).is_some()
}

/// Whether the code in `source` says `name`: an identifier of that name,
/// which a word in a string or a comment is not, so `require('./price')`
/// does not use a function `price`. `None` without a grammar for `path`.
pub fn uses_name(path: &Path, source: &str, name: &str) -> Option<bool> {
    let lang = language::detect(path)?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang.grammar).ok()?;
    let tree = parser.parse(source, None)?;
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        // `identifier`, `property_identifier`, JavaScript's
        // `shorthand_property_identifier_pattern` and the like in every
        // grammar here; Ruby's capitalized names are `constant`.
        let kind = node.kind();
        if (kind.contains("identifier") || kind == "constant") && text(node, source) == Some(name) {
            return Some(true);
        }
        if cursor.goto_first_child() || cursor.goto_next_sibling() {
            continue;
        }
        loop {
            if !cursor.goto_parent() {
                return Some(false);
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// How the code in one file says one name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NameUse {
    /// The 1-based lines, in order, where code refers to the name, an import
    /// of it included. Not a string or a comment, an object key or a keyword
    /// argument, a declaration, nor `a.name`, a property of something else,
    /// unless `a` is a module the file imports, or the caller counts members
    /// and the code calls it: `a.name()`.
    pub lines: Vec<usize>,
    /// The file binds the name, in any scope: a function, class, variable or
    /// parameter of that name, or an import that names what it brings in so
    /// (a default or namespace import, a module required whole, an alias),
    /// so the lines may refer to its own.
    pub binds: bool,
    /// It binds the name outside every function, class and block, where code
    /// that imports the file reaches it: a function or class, a variable such
    /// as `const price = memo(...)`, a Python or Go one, a Rust `const` or
    /// `static`, `exports.price = ...` or a key of `module.exports = {...}`.
    pub exports: bool,
}

/// How the code in `source` says `name`. With `members`, `a.name()` refers
/// to it too, as a call of a method does; `a.name` without a call reads a
/// field of that name, which is no use of a method. `None` without a grammar
/// for `path`.
// ponytail: a file that binds the name anywhere, a parameter included, has
// no uses of another's, and `const { price } = cart` binds it; resolve scopes
// if a shadowed name ever hides a real use.
pub fn name_use(path: &Path, source: &str, name: &str, members: bool) -> Option<NameUse> {
    let lang = language::detect(path)?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang.grammar).ok()?;
    let tree = parser.parse(source, None)?;
    let mut found = NameUse::default();
    // `a.name`, kept until the file's imports say whether `a` is a module.
    let mut properties = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        let kind = node.kind();
        if (kind.contains("identifier") || kind == "constant") && text(node, source) == Some(name) {
            let line = node.start_position().row + 1;
            match role(node, cursor.field_name(), members, source) {
                Role::Refers => found.lines.push(line),
                Role::Property(of) => properties.push((line, of)),
                Role::Binds { top } => {
                    found.binds = true;
                    found.exports |= top;
                }
                Role::Neither => {}
            }
        }
        if cursor.goto_first_child() || cursor.goto_next_sibling() {
            continue;
        }
        loop {
            if !cursor.goto_parent() {
                if !properties.is_empty() {
                    let modules = imported(tree.root_node(), source);
                    let module = |of: &Option<tree_sitter::Node>| {
                        of.is_some_and(|n| is_import_call(n, source))
                            || of
                                .and_then(|n| text(n, source))
                                .is_some_and(|t| modules.contains(t))
                    };
                    found.lines.extend(
                        properties
                            .iter()
                            .filter(|(_, of)| module(of))
                            .map(|(line, _)| *line),
                    );
                    found.lines.sort_unstable();
                }
                found.lines.dedup();
                return Some(found);
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// What an identifier of the name is to the code around it.
enum Role<'t> {
    Refers,
    /// `a.name`: a use only of a member, or of a module's name; with the
    /// innermost `a` of `a.b.name`.
    Property(Option<tree_sitter::Node<'t>>),
    /// A binding other than an import, `top` outside every function, class
    /// and block.
    Binds {
        top: bool,
    },
    /// An object key, a keyword argument, a module path or another class's
    /// member: no use and no binding.
    Neither,
}

/// Functions, classes and blocks: a binding inside one is not the file's own.
const SCOPES: &[&str] = &[
    "function_declaration",
    "generator_function_declaration",
    "function_expression",
    "function",
    "generator_function",
    "arrow_function",
    "method_definition",
    "class_body",
    "statement_block",
    "function_definition",
    "class_definition",
    "lambda",
    "method_declaration",
    "func_literal",
    "function_item",
    "closure_expression",
    "impl_item",
    "trait_item",
    "compound_statement",
    "method",
    "singleton_method",
];

/// Whether nothing around `node` is a function, class or block.
fn at_top(node: tree_sitter::Node<'_>) -> bool {
    let mut up = node.parent();
    while let Some(n) = up {
        if SCOPES.contains(&n.kind()) {
            return false;
        }
        up = n.parent();
    }
    true
}

fn role<'t>(
    node: tree_sitter::Node<'t>,
    field: Option<&str>,
    members: bool,
    source: &str,
) -> Role<'t> {
    let Some(parent) = node.parent() else {
        return Role::Refers;
    };
    if let Some(import) = import_role(node, source) {
        return import;
    }
    let pk = parent.kind();
    let member = |declared: bool| match (members, declared) {
        (false, _) => Role::Neither,
        (true, true) => Role::Binds { top: false },
        (true, false) => Role::Refers,
    };
    // Whether `n` is the `field` child of its parent.
    let is_field = |n: tree_sitter::Node<'t>, field: &str| {
        n.parent()
            .is_some_and(|p| p.child_by_field_name(field) == Some(n))
    };
    match (pk, field) {
        // `exports.price = ...` and `module.exports.price = ...`.
        ("member_expression", Some("property")) if is_export(parent, source) => Role::Binds {
            top: parent.parent().is_some_and(at_top),
        },
        // `a.price()` calls a method; `a.price` without a call reads a field
        // of that name, whatever `a` is, unless `a` is a module.
        ("member_expression", Some("property"))
        | ("attribute", Some("attribute"))
        | ("selector_expression", Some("field"))
        | ("field_expression", Some("field"))
        | ("field_access", Some("field")) => match (members, is_field(parent, "function")) {
            (false, _) => Role::Property(innermost(parent)),
            (true, true) => Role::Refers,
            (true, false) => Role::Neither,
        },
        ("method_invocation", Some("name")) | ("call", Some("method"))
            if parent.child_by_field_name("object").is_some()
                || parent.child_by_field_name("receiver").is_some() =>
        {
            if members {
                Role::Refers
            } else {
                Role::Property(innermost(parent))
            }
        }
        // `{ price: 1 }`, `<Item price={1} />`, `Item { price: 1 }`,
        // `f(price=1)`: keys, except what `module.exports = {...}` exports.
        ("pair", Some("key")) | ("method_definition", Some("name"))
            if parent
                .parent()
                .is_some_and(|o| is_exported_object(o, source)) =>
        {
            Role::Binds { top: true }
        }
        ("pair", Some("key")) if node.kind() == "property_identifier" => Role::Neither,
        ("pair_pattern", Some("key")) => member(false),
        ("jsx_attribute", _) | ("field_initializer", Some("field")) => Role::Neither,
        ("keyword_argument", Some("name")) => Role::Neither,
        ("literal_element", _)
            if is_field(parent, "key")
                && parent.parent().is_some_and(|k| k.kind() == "keyed_element") =>
        {
            Role::Neither
        }
        // A method or field: another class's, unless members are the point.
        ("method_definition", Some("name"))
        | ("field_definition", Some("property"))
        | ("public_field_definition", Some("name"))
        | ("method_signature", Some("name"))
        | ("property_signature", Some("name"))
        | ("field_declaration", Some("name"))
        | ("method_declaration", Some("name")) => member(true),
        // Declarations by name, and the targets of assignments and loops.
        (_, Some("name")) if declares(pk) => Role::Binds {
            top: at_top(parent),
        },
        ("function_expression" | "generator_function" | "class" | "function", Some("name")) => {
            Role::Binds { top: false }
        }
        (
            "required_parameter" | "optional_parameter" | "let_declaration" | "parameter"
            | "for_expression",
            Some("pattern"),
        )
        | ("arrow_function" | "catch_clause", Some("parameter"))
        | ("parameter_declaration" | "function_declarator", Some("declarator")) => {
            Role::Binds { top: false }
        }
        (
            "assignment"
            | "augmented_assignment"
            | "for_statement"
            | "for_in_clause"
            | "for_in_statement",
            Some("left"),
        )
        | (
            "init_declarator"
            | "declaration"
            | "pointer_declarator"
            | "reference_declarator"
            | "array_declarator",
            Some("declarator"),
        )
        | ("assignment_pattern", Some("left"))
        | ("pair_pattern", Some("value")) => Role::Binds {
            top: at_top(parent),
        },
        (
            "formal_parameters"
            | "parameters"
            | "lambda_parameters"
            | "typed_parameter"
            | "list_splat_pattern"
            | "dictionary_splat_pattern"
            | "closure_parameters"
            | "method_parameters",
            _,
        ) => Role::Binds { top: false },
        (
            "pattern_list" | "tuple_pattern" | "list_pattern" | "array_pattern" | "rest_pattern"
            | "object_pattern" | "as_pattern_target",
            _,
        ) => Role::Binds {
            top: at_top(parent),
        },
        // Go's `price := 1` and `for _, price := range xs`.
        ("expression_list", _)
            if is_field(parent, "left")
                && parent.parent().is_some_and(|d| {
                    matches!(d.kind(), "short_var_declaration" | "range_clause")
                }) =>
        {
            Role::Binds { top: false }
        }
        _ if node.kind() == "shorthand_property_identifier_pattern" => Role::Binds {
            top: at_top(parent),
        },
        _ => Role::Refers,
    }
}

/// Kinds whose `name` field declares what they bind.
fn declares(kind: &str) -> bool {
    matches!(
        kind,
        "variable_declarator"
            | "function_declaration"
            | "generator_function_declaration"
            | "class_declaration"
            | "abstract_class_declaration"
            | "function_signature"
            | "interface_declaration"
            | "type_alias_declaration"
            | "enum_declaration"
            | "function_definition"
            | "class_definition"
            | "default_parameter"
            | "typed_default_parameter"
            | "var_spec"
            | "const_spec"
            | "type_spec"
            | "parameter_declaration"
            | "variadic_parameter_declaration"
            | "function_item"
            | "const_item"
            | "static_item"
            | "struct_item"
            | "enum_item"
            | "trait_item"
            | "type_item"
            | "mod_item"
            | "formal_parameter"
            | "record_declaration"
            | "class_specifier"
            | "struct_specifier"
            | "namespace_definition"
            | "preproc_def"
            | "preproc_function_def"
            | "method"
            | "singleton_method"
            | "module"
    )
}

/// The innermost object of `a.b.name`, the member expression `of` reads.
fn innermost(of: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    let mut n = of;
    loop {
        let next = ["object", "operand", "value", "argument", "receiver"]
            .iter()
            .find_map(|f| n.child_by_field_name(f))?;
        match next.kind() {
            "member_expression"
            | "attribute"
            | "selector_expression"
            | "field_expression"
            | "field_access" => n = next,
            _ => return Some(next),
        }
    }
}

/// The call `require('./cart')` or `import('./cart')`, awaited or not.
fn import_call<'t>(node: tree_sitter::Node<'t>, source: &str) -> Option<tree_sitter::Node<'t>> {
    let mut n = node;
    while matches!(n.kind(), "await_expression" | "parenthesized_expression") {
        n = n.named_child(0)?;
    }
    let f = n.child_by_field_name("function")?;
    let imports =
        f.kind() == "import" || (f.kind() == "identifier" && text(f, source) == Some("require"));
    (n.kind() == "call_expression" && imports).then_some(n)
}

fn is_import_call(node: tree_sitter::Node<'_>, source: &str) -> bool {
    import_call(node, source).is_some()
}

/// Whether `specifier`, the string an import names its module by, names one
/// of Node's own, such as `'node:path'`, which no branch defines.
fn names_node_module(specifier: Option<tree_sitter::Node<'_>>, source: &str) -> bool {
    specifier
        .and_then(|s| text(s, source))
        .is_some_and(|t| t.trim_start_matches(['\'', '"', '`']).starts_with("node:"))
}

/// What an identifier is inside an import: the name it brings in refers to
/// what it imports, a module's own path does not. A name the file gives what
/// it imports, whatever the module calls it, is the file's own: a default or
/// namespace import, a module required whole, an alias, and whatever comes
/// from one of Node's own modules. `None` outside imports.
fn import_role<'t>(node: tree_sitter::Node<'t>, source: &str) -> Option<Role<'t>> {
    // What `const ... = require('./cart')` or `require('./cart').price`
    // requires.
    let required = |declarator: tree_sitter::Node<'t>| {
        let mut v = declarator.child_by_field_name("value")?;
        while v.kind() == "member_expression" {
            v = v.child_by_field_name("object")?;
        }
        import_call(v, source)
    };
    // `import price from`, `import * as price`, `import price = require()`,
    // and the alias of `{ unitPrice as price }` or `unit_price as price`.
    let named_here = node.parent().is_some_and(|p| {
        matches!(
            p.kind(),
            "import_clause" | "namespace_import" | "import_require_clause"
        ) || p.child_by_field_name("alias") == Some(node)
    });
    let mut child = node;
    let mut up = node.parent();
    while let Some(n) = up {
        match n.kind() {
            "import_statement"
            | "import_declaration"
            | "import_spec"
            | "use_declaration"
            | "import_require_clause" => {
                let node_module = names_node_module(n.child_by_field_name("source"), source);
                return Some(if named_here || node_module {
                    Role::Binds { top: false }
                } else {
                    Role::Refers
                });
            }
            "import_from_statement" => {
                return Some(if n.child_by_field_name("module_name") == Some(child) {
                    Role::Neither
                } else if named_here {
                    Role::Binds { top: false }
                } else {
                    Role::Refers
                });
            }
            // `const { price } = require('./cart')` and `const p =
            // require('./cart').price` bring in its `price`; `const cart =
            // require('./cart')` names the module.
            "variable_declarator"
                if n.child_by_field_name("name") == Some(child) && required(n).is_some() =>
            {
                let whole = child == node
                    && n.child_by_field_name("value")
                        .is_some_and(|v| v.kind() != "member_expression");
                let specifier = required(n)
                    .and_then(|call| call.child_by_field_name("arguments"))
                    .and_then(|args| args.named_child(0));
                return Some(if whole || names_node_module(specifier, source) {
                    Role::Binds { top: false }
                } else {
                    Role::Refers
                });
            }
            _ if SCOPES.contains(&n.kind()) => return None,
            _ => {}
        }
        child = n;
        up = n.parent();
    }
    None
}

/// `exports.x` or `module.exports.x`, assigned to.
fn is_export(member: tree_sitter::Node<'_>, source: &str) -> bool {
    let assigned = member.parent().is_some_and(|a| {
        a.kind() == "assignment_expression" && a.child_by_field_name("left") == Some(member)
    });
    let object = member.child_by_field_name("object");
    assigned
        && object.is_some_and(|o| {
            (o.kind() == "identifier" && text(o, source) == Some("exports"))
                || is_module_exports(o, source)
        })
}

fn is_module_exports(node: tree_sitter::Node<'_>, source: &str) -> bool {
    node.kind() == "member_expression"
        && node
            .child_by_field_name("object")
            .is_some_and(|o| text(o, source) == Some("module"))
        && node
            .child_by_field_name("property")
            .is_some_and(|p| text(p, source) == Some("exports"))
}

/// The object of `module.exports = { ... }`.
fn is_exported_object(object: tree_sitter::Node<'_>, source: &str) -> bool {
    object.kind() == "object"
        && object.parent().is_some_and(|a| {
            a.kind() == "assignment_expression"
                && a.child_by_field_name("right") == Some(object)
                && a.child_by_field_name("left")
                    .is_some_and(|l| is_module_exports(l, source))
        })
}

/// The names the file's imports bind, a module's among them: `cart` of
/// `const cart = require('./cart')`, `import * as cart`, `import cart` or
/// Go's `import "example.com/cart"`.
fn imported<'s>(
    root: tree_sitter::Node<'_>,
    source: &'s str,
) -> std::collections::BTreeSet<&'s str> {
    let mut names = std::collections::BTreeSet::new();
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        let kind = node.kind();
        if kind.contains("identifier")
            && matches!(
                import_role(node, source),
                Some(Role::Refers | Role::Binds { .. })
            )
        {
            names.extend(text(node, source));
        }
        // Go names a package by its path's last part.
        if kind == "interpreted_string_literal"
            && node.parent().is_some_and(|p| p.kind() == "import_spec")
        {
            if let Some(path) = text(node, source) {
                names.extend(path.trim_matches('"').rsplit('/').next());
            }
        }
        if cursor.goto_first_child() || cursor.goto_next_sibling() {
            continue;
        }
        loop {
            if !cursor.goto_parent() {
                return names;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
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
            member: !prefix.is_empty() || node.kind() == "method_declaration",
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
            // `export default function (…) {…}` has no name of its own.
            "function_expression" | "function" | "arrow_function" | "class"
                if node
                    .parent()
                    .is_some_and(|p| p.kind() == "export_statement") =>
            {
                Some(named_field(node, source, "name").unwrap_or_else(|| "default".into()))
            }
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
    fn anonymous_default_export_is_named_default() {
        // mri's whole parser is `export default function (args, opts) {…}`;
        // without a name, two agents editing it read as "separate symbols".
        let src = "function toVal() {}\n\nexport default function (args) {\n  return args;\n}\n";
        let spans = symbol_spans(Path::new("index.js"), src);
        assert_eq!(innermost_symbol(&spans, 4).unwrap().name, "default");
        assert_eq!(innermost_symbol(&spans, 1).unwrap().name, "toVal");
        let arrow = symbol_spans(
            Path::new("a.ts"),
            "export default () => {\n  return 1;\n};\n",
        );
        assert_eq!(innermost_symbol(&arrow, 2).unwrap().name, "default");
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
        assert!(parses(Path::new("src/a.py")) && !parses(Path::new("notes.txt")));
    }

    #[test]
    fn a_name_in_a_string_or_a_comment_is_not_a_use() {
        let js = Path::new("cart.js");
        let only_said =
            "// price became cost\nconst { cost } = require('./price');\nconst s = \"price\";\n";
        assert_eq!(uses_name(js, only_said, "price"), Some(false));
        for used in [
            "module.exports = () => price(1);\n",
            "const s = `${price}`;\n",
            "const { price } = require('./cart');\n",
            "cart.price();\n",
        ] {
            assert_eq!(uses_name(js, used, "price"), Some(true), "{used}");
        }
        let py = Path::new("cart.py");
        assert_eq!(
            uses_name(
                py,
                "# price\ns = 'price'\nfrom price import cost\n",
                "price"
            ),
            Some(true)
        );
        assert_eq!(
            uses_name(py, "# price\ns = f'{price}'\n", "price"),
            Some(true)
        );
        assert_eq!(
            uses_name(py, "# price\ns = 'price'\n", "price"),
            Some(false)
        );
        assert_eq!(
            uses_name(Path::new("Cart.rb"), "Price.new\n", "Price"),
            Some(true)
        );
        assert_eq!(uses_name(Path::new("notes.txt"), "price", "price"), None);
    }

    /// `name_use` of `price` in `source`, as `path`.
    fn price_in(path: &str, source: &str) -> NameUse {
        name_use(Path::new(path), source, "price", false).unwrap()
    }

    #[test]
    fn a_name_is_used_where_code_refers_to_it() {
        let js = "const { price } = require('./cart.js');\n// price: what one item costs\nconst label = 'price';\nconst q = { price: 1, total: price(2) };\nitem.price;\n<Item price={1} />;\nmodule.exports = () => price({ cost: 2 });\n";
        let found = price_in("src/a.js", js);
        assert_eq!(found.lines, [1, 4, 7]);
        assert!(!found.binds && !found.exports);
        // `item.price` reads a field, even when members count.
        assert_eq!(
            name_use(Path::new("a.js"), js, "price", true)
                .unwrap()
                .lines,
            [1, 4, 7]
        );
        // `a.price` is a use of a module the file imports.
        let module = "const cart = require('./cart');\nimport * as shop from './shop';\ncart.price(1);\nshop.price(2);\nrequire('./cart').price(3);\nthis.price;\n";
        assert_eq!(price_in("a.js", module).lines, [3, 4, 5]);
        let py = "from shop.cart import price\nimport shop.tax as tax\nx = price(1)\ny = tax.price(2)\nz = item.price\nf(price=3)\ns = 'price'  # price\n";
        assert_eq!(price_in("a.py", py).lines, [1, 3, 4]);
        let go = "package main\n\nimport \"example.com/shop\"\n\nfunc F(c Cart) int {\n\tx := shop.Price(1)\n\ty := Item{Price: 2}\n\treturn x + c.Price + y.Price\n}\n";
        let found = name_use(Path::new("main.go"), go, "Price", false).unwrap();
        assert_eq!(found.lines, [6]);
        let found = name_use(Path::new("main.go"), go, "Price", true).unwrap();
        assert_eq!(found.lines, [6]);
        let rs = "use crate::cart::price;\nfn f(a: A) -> u32 {\n    let z = Item { price: 1 };\n    cart::price(1) + a.price\n}\n";
        assert_eq!(price_in("lib.rs", rs).lines, [1, 4]);
        assert_eq!(
            name_use(Path::new("notes.md"), "price", "price", false),
            None
        );
    }

    /// A default import, a namespace import, a whole module required or an
    /// alias gives what it brings in a name of the file's own choosing: a
    /// file with one binds the name itself. `cart.price()` on such a module
    /// is still a use.
    #[test]
    fn an_import_named_here_binds_its_own_name() {
        for (path, own) in [
            (
                "a.js",
                "import price from './pricing.js';\nprice({ cost: 2 });\n",
            ),
            (
                "a.js",
                "import * as price from './pricing.js';\nprice.of(1);\n",
            ),
            (
                "a.js",
                "const price = require('./pricing.js');\nprice(1);\n",
            ),
            ("a.ts", "import price = require('./pricing');\nprice(1);\n"),
            (
                "a.js",
                "import { unitPrice as price } from './cart.js';\nprice(1);\n",
            ),
            (
                "a.py",
                "from shop.cart import unit_price as price\nprice(1)\n",
            ),
            ("a.py", "import shop.pricing as price\nprice.of(1)\n"),
        ] {
            let found = price_in(path, own);
            assert!(found.binds && !found.exports, "{own}");
        }
        let module = "import cart from './cart.js';\nimport * as shop from './shop.js';\nconst tax = require('./tax.js');\ncart.price(1);\nshop.price(2);\ntax.price(3);\n";
        assert_eq!(price_in("a.js", module).lines, [4, 5, 6]);
    }

    /// Node's own modules are no branch's code: a name brought in from
    /// `node:path` is the file's own, however the repository names its own.
    #[test]
    fn a_name_from_a_node_module_is_the_files_own() {
        let basename =
            |source: &str| name_use(Path::new("a.js"), source, "basename", false).unwrap();
        for own in [
            "import { basename } from 'node:path';\nbasename(p);\n",
            "const { basename } = require(\"node:path\");\nbasename(p);\n",
            "const basename = require('node:path').basename;\nbasename(p);\n",
        ] {
            assert!(basename(own).binds, "{own}");
        }
        let repos = "import { basename } from './files.js';\nbasename(p);\n";
        assert!(!basename(repos).binds);
    }

    /// A method is used where it is called. `a.name` read without a call is
    /// a field of that name, whatever `a` is: an order's `subtotal`, an
    /// invoice's `Total`, `self.total = 0`.
    #[test]
    fn a_method_is_used_only_where_it_is_called() {
        let method = |path: &str, source: &str, name: &str| {
            name_use(Path::new(path), source, name, true).unwrap().lines
        };
        let js = "cart.subtotal();\norder.subtotal;\ncart?.subtotal();\nthis.subtotal().toFixed(2);\nconst s = { subtotal: order.subtotal };\nconst t = shop.cart.subtotal();\n";
        assert_eq!(method("a.js", js, "subtotal"), [1, 3, 4, 6]);
        let py = "cart.total()\ninv.total\nself.total = 0\nself.total += 1\nprint(Receipt().add(4).total)\n";
        assert_eq!(method("a.py", py, "total"), [1]);
        let go = "package p\n\nfunc F(c Cart, got Invoice) int {\n\tx := c.Total()\n\ty := got.Total\n\treturn x + y\n}\n";
        assert_eq!(method("a_test.go", go, "Total"), [4]);
        let rs = "fn f(a: A) -> u32 {\n    let x = a.price();\n    x + a.price\n}\n";
        assert_eq!(method("lib.rs", rs, "price"), [2]);
    }

    /// A file that brings its own `price`, in whatever form, uses that one;
    /// one that binds it outside every function gives it to its importers.
    #[test]
    fn a_name_a_file_binds_itself_is_its_own() {
        for (path, own) in [
            (
                "a.js",
                "const memo = (f) => f;\nconst price = memo((i) => i.cost);\n",
            ),
            ("a.js", "let price;\n"),
            ("a.js", "exports.price = (i) => i.cost;\n"),
            ("a.js", "module.exports.price = (i) => i.cost;\n"),
            ("a.js", "module.exports = { price: (i) => i.cost };\n"),
            (
                "a.js",
                "module.exports = { price(i) { return i.cost; } };\n",
            ),
            ("a.ts", "export const price = memo((i: Item) => i.cost);\n"),
            ("a.py", "price = memo(lambda i: i.cost)\n"),
            ("a.py", "if fast:\n    price = fast_price\n"),
            ("a.go", "package shop\n\nvar price = memo(cost)\n"),
            ("a.rs", "const price: u32 = 1;\n"),
        ] {
            let found = price_in(path, own);
            assert!(found.binds && found.exports, "{own}");
        }
        // Inside a function, or a parameter: the file's, not its importers'.
        for (path, local) in [
            (
                "a.js",
                "function f(items) {\n  let price = 0;\n  return price;\n}\n",
            ),
            ("a.js", "const f = (price) => price * 2;\n"),
            ("a.js", "function f({ price }) {}\n"),
            ("a.ts", "function f(price: number) {}\n"),
            ("a.py", "def f(price):\n    return price\n"),
            ("a.py", "def f():\n    price = 1\n"),
            (
                "a.go",
                "package shop\n\nfunc F() { price := 1; _ = price }\n",
            ),
            ("a.rs", "fn f() { let price = 1; }\n"),
        ] {
            let found = price_in(path, local);
            assert!(found.binds && !found.exports, "{local}");
        }
        // An import is a use, not a binding of its own; nor is another
        // class's method, unless members count.
        for (path, import) in [
            ("a.js", "const { price } = require('./cart');\n"),
            ("a.js", "const price = require('./cart').price;\n"),
            ("a.js", "import { price } from './cart';\n"),
            ("a.py", "from .cart import price\n"),
            ("a.js", "class A {\n  price() {}\n}\n"),
        ] {
            let found = price_in(path, import);
            assert!(!found.binds && !found.exports, "{import}");
        }
        let method = "class A {\n  price() {}\n}\n";
        assert!(
            name_use(Path::new("a.js"), method, "price", true)
                .unwrap()
                .binds
        );
    }
}
