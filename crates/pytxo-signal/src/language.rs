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
        _ => None,
    }
}
