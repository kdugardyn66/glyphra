//! AI Tools: the catalogue of tools per kind of file. They arrive in a later version; the
//! page shows what is planned and says so when one is clicked.

use dioxus::prelude::*;
use gl_core::{AppModel, NoticeKind, t, tr, tx};

/// One group of tools on the page.
pub struct Section {
    pub title: &'static str,
    /// CSS class of the file icon (`doc`, `slides`, `other`).
    pub accent: &'static str,
    /// The letter on the file icon.
    pub letter: &'static str,
    /// (title, description)
    pub tools: &'static [(&'static str, &'static str)],
}

pub const SECTIONS: [Section; 3] = [
    Section {
        title: tx("Document"),
        accent: "doc",
        letter: "D",
        tools: &[
            (tx("Translate Document"), tx("Translate a document and keep its formatting")),
            (tx("Compare Documents"), tx("Find the differences between two documents")),
            (tx("Review Document"), tx("Find factual errors, inconsistencies and typos")),
            (tx("Proofread Document"), tx("Fix typos and grammar as suggestions")),
            (tx("Fix Formatting"), tx("Make headings, lists and spacing consistent")),
            (tx("Summarize Document"), tx("Get the key points, figures and dates")),
            (tx("Convert to Book"), tx("Turn documents into e-books (EPUB)")),
        ],
    },
    Section {
        title: tx("Presentation"),
        accent: "slides",
        letter: "P",
        tools: &[
            (tx("Slides from Document"), tx("Draft a slide deck from a document")),
            (tx("Improve Slides"), tx("Tighten titles and bullet points")),
            (tx("Speaker Notes"), tx("Write notes for every slide")),
            (tx("Translate Presentation"), tx("Translate a deck and keep its layout")),
        ],
    },
    Section {
        title: tx("Other Tools"),
        accent: "other",
        letter: "T",
        tools: &[
            (tx("Check Before Publishing"), tx("Find local paths and broken links")),
            (tx("Remove Personal Data"), tx("Clear names, emails and phones")),
        ],
    },
];

#[component]
pub fn AiToolsPage() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    rsx! {
        div { class: "page ai-tools",
            h1 { {t("AI Tools")} }
            for sec in SECTIONS {
                section { key: "{sec.title}",
                    h2 { {t(sec.title)} }
                    div { class: "tool-grid",
                        for (title , desc) in sec.tools.iter().copied() {
                            button {
                                key: "{title}",
                                class: "tool",
                                onclick: move |_| model.write().notify(NoticeKind::Info, tr("“{title}” arrives in a later version of Glyphra.", &[("title", &t(title))])),
                                span { class: "tool-ico {sec.accent}", "{sec.letter}" }
                                span { class: "tool-tx",
                                    span { class: "tool-title", {t(title)} }
                                    span { class: "tool-desc", {t(desc)} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_has_a_title_and_description_and_titles_are_unique() {
        let mut titles = Vec::new();
        for sec in SECTIONS {
            assert!(!sec.tools.is_empty());
            for (t, d) in sec.tools {
                assert!(!t.is_empty() && !d.is_empty());
                titles.push(*t);
            }
        }
        let n = titles.len();
        titles.sort();
        titles.dedup();
        assert_eq!(n, titles.len());
    }
}
