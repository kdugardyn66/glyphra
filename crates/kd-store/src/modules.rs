//! The Glyphra modules and the file types they own.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A Glyphra module. 0.1.0 only shows them on the start screen; the editors arrive in
/// later releases (Markdown 0.3.0, PDF 0.7.0, Drawing 0.8.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleKind {
    Markdown,
    Pdf,
    Drawing,
}

impl ModuleKind {
    /// All modules in start-screen order.
    pub const ALL: [ModuleKind; 3] = [ModuleKind::Markdown, ModuleKind::Pdf, ModuleKind::Drawing];

    /// Extensions owned by Glyphra's own modules.
    pub const EXTENSIONS: [(&'static str, ModuleKind); 4] =
        [("md", ModuleKind::Markdown), ("markdown", ModuleKind::Markdown), ("pdf", ModuleKind::Pdf), ("svg", ModuleKind::Drawing)];

    /// Short label for pills and badges (English; the UI translates through Fluent ids).
    pub fn short(self) -> &'static str {
        match self {
            ModuleKind::Markdown => "Markdown",
            ModuleKind::Pdf => "PDF",
            ModuleKind::Drawing => "Drawing",
        }
    }

    /// Two- or three-letter badge text.
    pub fn badge(self) -> &'static str {
        match self {
            ModuleKind::Markdown => "MD",
            ModuleKind::Pdf => "PDF",
            ModuleKind::Drawing => "DRW",
        }
    }

    /// The release in which the module's editor ships.
    pub fn release(self) -> &'static str {
        match self {
            ModuleKind::Markdown => "0.3.0",
            ModuleKind::Pdf => "0.7.0",
            ModuleKind::Drawing => "0.8.0",
        }
    }

    /// Stable id fragment: command ids (`file.new.markdown`) and Fluent ids (`module-markdown`).
    pub fn id(self) -> &'static str {
        match self {
            ModuleKind::Markdown => "markdown",
            ModuleKind::Pdf => "pdf",
            ModuleKind::Drawing => "drawing",
        }
    }

    /// Extension of a new document.
    pub fn new_extension(self) -> &'static str {
        match self {
            ModuleKind::Markdown => "md",
            ModuleKind::Pdf => "pdf",
            ModuleKind::Drawing => "svg",
        }
    }

    /// Starter content for a new document.
    pub fn starter_content(self, title: &str) -> String {
        match self {
            ModuleKind::Markdown => format!("# {title}\n\n"),
            ModuleKind::Pdf => crate::files::blank_pdf(title),
            ModuleKind::Drawing => crate::files::blank_drawing(title),
        }
    }

    /// The module that owns a file, judged by extension.
    pub fn from_path(path: &Path) -> Option<ModuleKind> {
        Self::from_extension(&path.extension()?.to_str()?.to_ascii_lowercase())
    }

    /// The module that owns a lower-case extension.
    pub fn from_extension(ext: &str) -> Option<ModuleKind> {
        Self::EXTENSIONS.iter().find(|(e, _)| *e == ext).map(|(_, m)| *m)
    }

    /// Start-screen filter group of this module's documents.
    pub fn group(self) -> crate::doctype::DocGroup {
        use crate::doctype::DocGroup;
        match self {
            ModuleKind::Markdown => DocGroup::Markdown,
            ModuleKind::Pdf => DocGroup::Pdf,
            ModuleKind::Drawing => DocGroup::Drawing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_module_from_extension() {
        assert_eq!(ModuleKind::from_path(Path::new("a/b.MD")), Some(ModuleKind::Markdown));
        assert_eq!(ModuleKind::from_path(Path::new("x.pdf")), Some(ModuleKind::Pdf));
        assert_eq!(ModuleKind::from_path(Path::new("x.SVG")), Some(ModuleKind::Drawing));
        assert_eq!(ModuleKind::from_path(Path::new("x.txt")), None);
        assert_eq!(ModuleKind::from_path(Path::new("noext")), None);
    }

    #[test]
    fn new_extension_round_trips() {
        for kind in ModuleKind::ALL {
            let p = format!("file.{}", kind.new_extension());
            assert_eq!(ModuleKind::from_path(Path::new(&p)), Some(kind));
        }
    }
}
