//! The AI Tools page: ready-made actions for documents, grouped by module. The Markdown tools
//! work now; the PDF and Drawing groups are listed and switch on with their modules.

use crate::app::{run, run_effect};
use crate::icons::Icon;
use crate::model::{AppModel, Effect};
use crate::preview;
use dioxus::prelude::*;
use kd_ai::tasks::TaskOutput;
use kd_commands::{Command, SettingsSection};

struct Tool {
    /// The task id (`md.improve`); also the base of the Fluent ids.
    id: &'static str,
    icon: &'static str,
}

struct Group {
    id: &'static str,
    /// The release that switches the group on; `None` = available now.
    soon: Option<&'static str>,
    tools: &'static [Tool],
}

const GROUPS: [Group; 3] = [
    Group {
        id: "markdown",
        soon: None,
        tools: &[
            Tool { id: "md.draft", icon: "sparkles" },
            Tool { id: "md.improve", icon: "tools" },
            Tool { id: "md.proofread", icon: "tools" },
            Tool { id: "md.summarize", icon: "chat" },
            Tool { id: "md.translate", icon: "chat" },
            Tool { id: "md.outline", icon: "apps" },
            Tool { id: "md.tidy", icon: "tools" },
            Tool { id: "md.toc", icon: "apps" },
        ],
    },
    Group {
        id: "pdf",
        soon: Some("0.7.0"),
        tools: &[Tool { id: "pdf.summarize", icon: "chat" }, Tool { id: "pdf.ask", icon: "chat" }, Tool { id: "pdf.tables", icon: "apps" }],
    },
    Group {
        id: "drawing",
        soon: Some("0.8.0"),
        tools: &[Tool { id: "draw.create", icon: "sparkles" }, Tool { id: "draw.describe", icon: "chat" }],
    },
];

fn key(id: &str) -> String {
    format!("tool-{}", id.replace('.', "-"))
}

#[component]
pub fn AiToolsPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let open = model.read().ai.tool.is_some();
    if open {
        return rsx! { ToolPanel {} };
    }
    let m = model.read();
    let title = m.t("ai-tools-title");
    let sub = m.t("ai-tools-sub");
    let ready = m.ai.ready();
    let (l_setup, l_setup_btn) = (m.t("ai-setup-note"), m.t("ai-setup-button"));
    struct Card {
        id: &'static str,
        icon: &'static str,
        title: String,
        desc: String,
        badge: Option<String>,
    }
    let groups: Vec<(String, Vec<Card>)> = GROUPS
        .iter()
        .map(|g| {
            let cards = g
                .tools
                .iter()
                .map(|t| Card {
                    id: t.id,
                    icon: t.icon,
                    title: m.t(&key(t.id)),
                    desc: m.t(&format!("{}-desc", key(t.id))),
                    badge: g.soon.map(|v| m.t_args("ai-tools-soon", &[("version", v.into())])),
                })
                .collect();
            (m.t(&format!("ai-group-{}", g.id)), cards)
        })
        .collect();
    drop(m);
    rsx! {
        div { class: "ai-tools",
            h1 { class: "page-title", "{title}" }
            div { class: "sub", "{sub}" }
            if !ready {
                div { class: "ai-note",
                    span { "{l_setup}" }
                    button { class: "btn primary", onclick: move |_| run(model, Command::OpenSettings(SettingsSection::Ai)), "{l_setup_btn}" }
                }
            }
            for (heading , cards) in groups {
                div { key: "{heading}", class: "section",
                    div { class: "section-head",
                        h2 { "{heading}" }
                    }
                    div { class: "tool-grid",
                        for c in cards {
                            button {
                                key: "{c.id}",
                                class: if c.badge.is_some() { "tool-card later" } else { "tool-card" },
                                disabled: c.badge.is_some(),
                                onclick: move |_| run(model, Command::AiOpenTool(c.id.to_string())),
                                span { class: "ico", Icon { name: c.icon, size: 20 } }
                                span { class: "tt",
                                    span { class: "n", "{c.title}" }
                                    span { class: "d", "{c.desc}" }
                                    if let Some(b) = &c.badge {
                                        span { class: "badge", "{b}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ToolPanel() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let mut input = use_signal(String::new);
    let mut note = use_signal(String::new);
    let m = model.read();
    let Some(run_state) = m.ai.tool.as_ref() else { return rsx! {} };
    let id = run_state.task.id;
    let creates = run_state.task.creates;
    let shows_markdown = matches!(run_state.task.output, TaskOutput::Content | TaskOutput::Answer) && !id.starts_with("draw.");
    let running = run_state.running();
    let output = run_state.output.clone();
    let error = run_state.error.clone();
    let html = if shows_markdown && !output.is_empty() { preview::render(&output) } else { String::new() };
    let saveable = id.starts_with("md.") && !output.trim().is_empty() && !running;
    let t = |k: &str| m.t(k);
    let (name, desc) = (t(&key(id)), t(&format!("{}-desc", key(id))));
    let (l_back, l_in, l_note, l_note_ph, l_run, l_stop, l_out, l_copy, l_save, l_thinking) = (
        t("ai-tool-back"),
        t(if creates { "ai-tool-input-create" } else { "ai-tool-input" }),
        t("ai-tool-note"),
        t("ai-tool-note-ph"),
        t("ai-tool-run"),
        t("ai-stop"),
        t("ai-tool-result"),
        t("ai-copy"),
        t("ai-tool-save"),
        t("ai-thinking"),
    );
    drop(m);
    let can_run = !running && !input.read().trim().is_empty();
    rsx! {
        div { class: "ai-tools tool-panel",
            button { class: "link back", onclick: move |_| run(model, Command::AiCloseTool),
                Icon { name: "back", size: 14 }
                " {l_back}"
            }
            h1 { class: "page-title", "{name}" }
            div { class: "sub", "{desc}" }
            div { class: "field",
                label { "{l_in}" }
                textarea { class: "tool-in", rows: "8", value: "{input}", oninput: move |e| input.set(e.value()) }
            }
            div { class: "field",
                label { "{l_note}" }
                input { r#type: "text", placeholder: "{l_note_ph}", value: "{note}", oninput: move |e| note.set(e.value()) }
            }
            div { class: "tool-actions",
                if running {
                    button { class: "btn", onclick: move |_| run(model, Command::AiStop), "{l_stop}" }
                } else {
                    button {
                        class: "btn primary",
                        disabled: !can_run,
                        onclick: move |_| run(model, Command::AiRunTool { input: input.read().clone(), note: note.read().clone() }),
                        "{l_run}"
                    }
                }
            }
            if running || !output.is_empty() || error.is_some() {
                div { class: "field",
                    label { "{l_out}" }
                    if let Some(e) = error {
                        div { class: "tool-out error selectable", "{e}" }
                    } else if output.is_empty() {
                        div { class: "tool-out dim", "{l_thinking}" }
                    } else if shows_markdown {
                        div { class: "tool-out md selectable", dangerous_inner_html: "{html}" }
                    } else {
                        pre { class: "tool-out selectable", "{output}" }
                    }
                }
                if !output.is_empty() && !running {
                    div { class: "tool-actions",
                        button { class: "btn", onclick: { let o = output.clone(); move |_| run_effect(model, Effect::CopyText(o.clone())) }, "{l_copy}" }
                        if saveable {
                            button { class: "btn primary", onclick: move |_| run(model, Command::AiSaveToolResult), "{l_save}" }
                        }
                    }
                }
            }
        }
    }
}
