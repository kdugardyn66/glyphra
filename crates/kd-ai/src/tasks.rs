//! Ready-made AI actions per module. A module lists `tasks_for(its scope)` in its AI menu and
//! runs one with `task.request(selection_or_prompt)`; the home screen offers the "create" tasks.

use serde_json::Value;

use crate::scope::{AppKind, Scope};
use crate::types::Request;

/// What a task's reply is.
#[derive(Debug, Clone, Copy)]
pub enum TaskOutput {
    /// Text in the scope's output format (Markdown, CSV, HTML, draw.io XML).
    Content,
    /// A short answer to show, not insert.
    Answer,
    /// JSON matching the schema.
    Json(fn() -> Value),
}

#[derive(Debug, Clone, Copy)]
pub struct Task {
    pub id: &'static str,
    pub scope: Scope,
    pub label: &'static str,
    /// Put in front of the user's input.
    pub instruction: &'static str,
    pub output: TaskOutput,
    /// Creates a new file (as opposed to working on a selection).
    pub creates: bool,
}

impl Task {
    /// The request for this task on `input` (the user's prompt, or the selected text).
    pub fn request(&self, input: &str) -> Request {
        self.request_with("", input)
    }

    /// Like [`Task::request`], with a remark of the user's after the instruction ("into French").
    pub fn request_with(&self, note: &str, input: &str) -> Request {
        let note = note.trim();
        let instruction = if note.is_empty() { self.instruction.to_string() } else { format!("{} {note}", self.instruction) };
        let prompt = if self.creates {
            format!("{instruction}\n\n{}", input.trim())
        } else {
            format!("{instruction}\n\n<selection>\n{}\n</selection>", input.trim())
        };
        let r = Request::new(self.scope).user(prompt);
        match self.output {
            TaskOutput::Json(schema) => r.json(self.id, schema()),
            _ => r,
        }
    }
}

const fn t(id: &'static str, scope: Scope, label: &'static str, instruction: &'static str, output: TaskOutput, creates: bool) -> Task {
    Task { id, scope, label, instruction, output, creates }
}

const MD: Scope = Scope::App(AppKind::Markdown);
const PDF: Scope = Scope::App(AppKind::Pdf);
const DRAW: Scope = Scope::App(AppKind::Drawing);
use TaskOutput::{Answer, Content};

pub static TASKS: &[Task] = &[
    t("md.draft", MD, "Draft a Markdown file", "Write a Markdown document about the following.", Content, true),
    t(
        "md.improve",
        MD,
        "Improve writing",
        "Rewrite this text to be clearer and more concise. Keep the meaning and the language.",
        Content,
        false,
    ),
    t("md.proofread", MD, "Proofread & fix", "Correct spelling, grammar and punctuation. Change nothing else.", Content, false),
    t("md.summarize", MD, "Summarize", "Summarize this in a short paragraph followed by up to five bullet points.", Content, false),
    t(
        "md.translate",
        MD,
        "Translate",
        "Translate this into the language named in the request, or into English if none is named. Keep the Markdown structure.",
        Content,
        false,
    ),
    t("md.outline", MD, "Generate outline", "Write a hierarchical outline (headings and bullets) of this text.", Content, false),
    t("md.tidy", MD, "Tidy Markdown", "Clean up this Markdown: consistent headings, lists and tables. Keep the content.", Content, false),
    t("md.toc", MD, "Table of contents", "Write a linked table of contents for this Markdown.", Content, false),
    t(
        "pdf.summarize",
        PDF,
        "Summarize PDF",
        "Summarize the attached document: one paragraph, then the key points with page numbers.",
        Answer,
        false,
    ),
    t("pdf.ask", PDF, "Ask the PDF", "Answer the question about the attached document.", Answer, false),
    t(
        "pdf.tables",
        PDF,
        "Extract tables",
        "Extract every table in the attached document as Markdown pipe tables, each with its page number.",
        Content,
        false,
    ),
    t(
        "draw.create",
        DRAW,
        "Draw from a description",
        "Create an SVG drawing of the following. Use a clear layout, readable labels and a viewBox.",
        Content,
        true,
    ),
    t("draw.describe", DRAW, "Describe this drawing", "Describe what this SVG shows in two or three sentences.", Answer, false),
];

/// The tasks a scope offers. The home screen offers every module's "create" tasks.
pub fn tasks_for(scope: Scope) -> Vec<&'static Task> {
    match scope {
        Scope::Home => TASKS.iter().filter(|t| t.creates).collect(),
        s => TASKS.iter().filter(|t| t.scope == s).collect(),
    }
}

pub fn task(id: &str) -> Option<&'static Task> {
    TASKS.iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_module_has_tasks_and_one_create_task() {
        for k in AppKind::ALL {
            let ts = tasks_for(Scope::App(k));
            assert!(ts.len() >= 2, "{}", k.name());
            assert_eq!(ts.iter().filter(|t| t.creates).count(), if k == AppKind::Pdf { 0 } else { 1 }, "{}", k.name());
        }
        let mut ids: Vec<_> = TASKS.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), TASKS.len(), "task ids are unique");
    }

    #[test]
    fn requests() {
        let r = task("draw.create").unwrap().request("A flow chart");
        assert_eq!(r.scope, Scope::App(AppKind::Drawing));
        assert!(r.messages[0].text().ends_with("A flow chart"));
        let r = task("md.translate").unwrap().request_with("Into French.", "Hello");
        assert!(r.messages[0].text().contains("Keep the Markdown structure. Into French.\n\n<selection>"));
        let r = task("md.improve").unwrap().request("Some text");
        assert!(r.messages[0].text().contains("<selection>\nSome text\n</selection>"));
    }
}
