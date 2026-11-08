//! Who is asking: the start screen or one of the modules. A scope picks the default model and
//! adds the module's own instructions, so every module gets a model that knows its job and the
//! format its output has to be in.

use serde::{Deserialize, Serialize};

/// The Glyphra modules Glyphra AI works in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppKind {
    Markdown,
    Pdf,
    Drawing,
}

impl AppKind {
    pub const ALL: [AppKind; 3] = [AppKind::Markdown, AppKind::Pdf, AppKind::Drawing];

    pub fn id(self) -> &'static str {
        match self {
            AppKind::Markdown => "markdown",
            AppKind::Pdf => "pdf",
            AppKind::Drawing => "drawing",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            AppKind::Markdown => "Glyphra Markdown",
            AppKind::Pdf => "Glyphra PDF",
            AppKind::Drawing => "Glyphra Drawing",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Scope {
    /// The AI Chat page.
    #[default]
    Home,
    /// A module.
    App(AppKind),
}

/// The text format a module wants generated content in, so it can import it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    /// One self-contained SVG document.
    Svg,
}

impl Scope {
    pub const ALL: [Scope; 4] = [Scope::Home, Scope::App(AppKind::Markdown), Scope::App(AppKind::Pdf), Scope::App(AppKind::Drawing)];

    /// Stable id used in `ai.toml` (`home`, `markdown`, ...).
    pub fn id(self) -> &'static str {
        match self {
            Scope::Home => "home",
            Scope::App(k) => k.id(),
        }
    }

    pub fn from_id(id: &str) -> Option<Scope> {
        Scope::ALL.into_iter().find(|s| s.id() == id)
    }

    pub fn label(self) -> &'static str {
        match self {
            Scope::Home => "AI Chat (Glyphra AI)",
            Scope::App(k) => k.name(),
        }
    }

    /// The format this module imports generated content in.
    pub fn output_format(self) -> OutputFormat {
        match self {
            Scope::App(AppKind::Drawing) => OutputFormat::Svg,
            _ => OutputFormat::Markdown,
        }
    }

    /// The module's standing instructions, put in front of every request's own system prompt.
    pub fn system_prompt(self) -> &'static str {
        match self {
            Scope::Home => {
                "You are Glyphra AI, the assistant in Glyphra, a native app for Markdown, PDF and drawings. \
                 Answer clearly and concisely in the user's language. Use Markdown for structure when it helps."
            }
            Scope::App(AppKind::Markdown) => {
                "You are Glyphra AI in Glyphra Markdown. Return CommonMark/GitHub-flavoured Markdown only (tables, task \
                 lists and fenced code allowed), with no preamble, so it can be inserted as is."
            }
            Scope::App(AppKind::Pdf) => {
                "You are Glyphra AI in Glyphra PDF, the reading assistant. Answer from the attached PDF; quote the passage and \
                 name the page when you can. Say so plainly when the document doesn't contain the answer. Use Markdown. \
                 When asked to draft a new document or slide deck, there is no attached PDF: write it as Markdown only, with no \
                 preamble, for Glyphra PDF to turn into a PDF."
            }
            Scope::App(AppKind::Drawing) => {
                "You are Glyphra AI in Glyphra Drawing. When you produce a drawing, return one self-contained SVG document \
                 only (starting with <svg, with a viewBox, no scripts, no external references), with no preamble. \
                 Otherwise help with layout, shapes and labels, in Markdown."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_and_cover_every_module() {
        for s in Scope::ALL {
            assert_eq!(Scope::from_id(s.id()), Some(s));
            assert!(!s.system_prompt().is_empty());
        }
        for k in AppKind::ALL {
            assert!(Scope::ALL.contains(&Scope::App(k)), "{} has no AI scope", k.name());
        }
    }
}
