//! Language detection for Signal Core ([[signal-core]]).
//!
//! Supported tree-sitter grammars (Phase 31): **Rust**, **TypeScript/TSX**, **JavaScript**,
//! **Python**, **Go**, **Java**, **C/C++** (`.c`, `.h`, `.cpp`, `.hpp`, `.cc`), **Ruby**.
//! Unknown extensions fall back to raw file content (`fallback_raw`).

use std::path::Path;

use tree_sitter::Language;

pub struct DetectedLanguage {
    pub id: &'static str,
    pub grammar: Language,
}

pub fn detect(path: &Path) -> Option<DetectedLanguage> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some(DetectedLanguage {
            id: "rust",
            grammar: tree_sitter_rust::LANGUAGE.into(),
        }),
        "ts" | "tsx" => Some(DetectedLanguage {
            id: "typescript",
            grammar: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        }),
        "js" | "jsx" | "mjs" | "cjs" => Some(DetectedLanguage {
            id: "javascript",
            grammar: tree_sitter_javascript::LANGUAGE.into(),
        }),
        "py" | "pyi" => Some(DetectedLanguage {
            id: "python",
            grammar: tree_sitter_python::LANGUAGE.into(),
        }),
        "go" => Some(DetectedLanguage {
            id: "go",
            grammar: tree_sitter_go::LANGUAGE.into(),
        }),
        "java" => Some(DetectedLanguage {
            id: "java",
            grammar: tree_sitter_java::LANGUAGE.into(),
        }),
        "c" | "h" => Some(DetectedLanguage {
            id: "c",
            grammar: tree_sitter_cpp::LANGUAGE.into(),
        }),
        "cpp" | "hpp" | "cc" | "cxx" => Some(DetectedLanguage {
            id: "cpp",
            grammar: tree_sitter_cpp::LANGUAGE.into(),
        }),
        "rb" => Some(DetectedLanguage {
            id: "ruby",
            grammar: tree_sitter_ruby::LANGUAGE.into(),
        }),
        _ => None,
    }
}
