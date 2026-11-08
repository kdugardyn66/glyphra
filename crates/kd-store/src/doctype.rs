//! File-type registry: which Glyphra module or Crafting App opens a file, and which
//! filter pill it appears under on the start screen.
//!
//! Glyphra's modules are checked first, so an SVG opens in the Drawing module rather
//! than in VectorCraft.

use crate::craft::{CRAFT_APPS, CraftApp, craft_for_extension};
use crate::modules::ModuleKind;
use std::path::Path;

/// Filter pills on the start screen ("All" is no group).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocGroup {
    Markdown,
    Pdf,
    Drawing,
    Office,
    Graphics,
    Media,
    Cad,
}

impl DocGroup {
    /// In filter-pill order (after "All").
    pub const ALL: [DocGroup; 7] =
        [DocGroup::Markdown, DocGroup::Pdf, DocGroup::Drawing, DocGroup::Office, DocGroup::Graphics, DocGroup::Media, DocGroup::Cad];

    /// Stable id: Fluent ids (`group-markdown`) and the saved filter.
    pub fn id(self) -> &'static str {
        match self {
            DocGroup::Markdown => "markdown",
            DocGroup::Pdf => "pdf",
            DocGroup::Drawing => "drawing",
            DocGroup::Office => "office",
            DocGroup::Graphics => "graphics",
            DocGroup::Media => "media",
            DocGroup::Cad => "cad",
        }
    }

    /// English label; the UI translates through [`DocGroup::id`].
    pub fn label(self) -> &'static str {
        match self {
            DocGroup::Markdown => "Markdown",
            DocGroup::Pdf => "PDF",
            DocGroup::Drawing => "Drawing",
            DocGroup::Office => "Office",
            DocGroup::Graphics => "Graphics",
            DocGroup::Media => "Media",
            DocGroup::Cad => "CAD",
        }
    }
}

/// Who opens a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opener {
    Module(ModuleKind),
    Craft(&'static CraftApp),
}

/// The type of a document: its group and the app that opens it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocKind {
    pub group: DocGroup,
    pub opener: Opener,
}

impl DocKind {
    pub fn from_path(path: &Path) -> Option<DocKind> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        if let Some(m) = ModuleKind::from_extension(&ext) {
            return Some(DocKind { group: m.group(), opener: Opener::Module(m) });
        }
        craft_for_extension(&ext).map(|a| DocKind { group: a.group, opener: Opener::Craft(a) })
    }

    /// Badge text.
    pub fn badge(&self) -> &'static str {
        match self.opener {
            Opener::Module(m) => m.badge(),
            Opener::Craft(a) => a.badge,
        }
    }

    /// Name of the app that opens it ("Markdown", "SolidCraft").
    pub fn app_name(&self) -> &'static str {
        match self.opener {
            Opener::Module(m) => m.short(),
            Opener::Craft(a) => a.name,
        }
    }

    pub fn is_craft(&self) -> bool {
        matches!(self.opener, Opener::Craft(_))
    }
}

/// Every extension Glyphra lists (for the Open dialog filter).
pub fn all_extensions() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = ModuleKind::EXTENSIONS.iter().map(|(e, _)| *e).collect();
    for a in &CRAFT_APPS {
        for e in a.extensions {
            if !v.contains(e) {
                v.push(e);
            }
        }
    }
    v
}

/// The file types of the Open dialog: "All supported documents", then one entry per filter
/// pill with its extensions (`("Markdown", ["md", …])`).
pub fn open_dialog_types() -> Vec<(String, Vec<String>)> {
    let all = all_extensions();
    let mut out = vec![("All supported documents".to_string(), all.iter().map(|e| e.to_string()).collect())];
    for g in DocGroup::ALL {
        let exts: Vec<String> = all
            .iter()
            .filter(|e| DocKind::from_path(Path::new(&format!("x.{e}"))).is_some_and(|k| k.group == g))
            .map(|e| e.to_string())
            .collect();
        if !exts.is_empty() {
            out.push((format!("{} documents", g.label()), exts));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modules_win_over_crafting_apps() {
        let k = DocKind::from_path(Path::new("a/Report.PDF")).unwrap();
        assert_eq!(k.opener, Opener::Module(ModuleKind::Pdf));
        assert_eq!(k.group, DocGroup::Pdf);
        let svg = DocKind::from_path(Path::new("fig.svg")).unwrap();
        assert_eq!(svg.opener, Opener::Module(ModuleKind::Drawing));
    }

    #[test]
    fn crafting_app_types_are_recognised() {
        let k = DocKind::from_path(Path::new("bracket.step")).unwrap();
        assert_eq!(k.app_name(), "SolidCraft");
        assert_eq!(k.group, DocGroup::Cad);
        assert_eq!(DocKind::from_path(Path::new("deck.pptx")).unwrap().app_name(), "DeckCraft");
        assert_eq!(DocKind::from_path(Path::new("photo.psd")).unwrap().group, DocGroup::Graphics);
        assert!(DocKind::from_path(Path::new("notes.txt")).is_none());
    }

    #[test]
    fn open_dialog_lists_all_then_each_group() {
        let t = open_dialog_types();
        assert_eq!(t[0].0, "All supported documents");
        assert_eq!(t[0].1.len(), all_extensions().len());
        let md = t.iter().find(|(l, _)| l == "Markdown documents").unwrap();
        assert!(md.1.contains(&"md".to_string()) && !md.1.contains(&"pdf".to_string()));
        assert_eq!(t.len(), 1 + DocGroup::ALL.len());
    }

    #[test]
    fn pills_start_with_the_three_modules() {
        assert_eq!(&DocGroup::ALL[..3], &[DocGroup::Markdown, DocGroup::Pdf, DocGroup::Drawing]);
    }

    #[test]
    fn every_group_has_at_least_one_type() {
        let exts = all_extensions();
        for g in DocGroup::ALL {
            assert!(exts.iter().any(|e| DocKind::from_path(Path::new(&format!("x.{e}"))).unwrap().group == g), "{g:?} is empty");
        }
    }
}
