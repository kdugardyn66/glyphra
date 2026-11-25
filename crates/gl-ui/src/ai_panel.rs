//! The AI Chat panel on the right. The layout is final; the conversation itself arrives in a
//! later version, so sending says so instead of silently doing nothing.

use crate::icons::Icon;
use crate::pick::{Pick, PickItem};
use crate::shell::run;
use dioxus::prelude::*;
use gl_core::{AppModel, Command, NoticeKind, SettingsSection, Step, t, tr, tx};

const CHIPS: [&str; 10] = [
    tx("Summarize document"),
    tx("Improve writing"),
    tx("Write email"),
    tx("Extract key info"),
    tx("Explain simply"),
    tx("Generate outline"),
    tx("Proofread & fix"),
    tx("Brainstorm ideas"),
    tx("Adapt tone"),
    tx("Paraphrase"),
];

/// Say that a part of the chat is not there yet.
fn soon(mut model: Signal<AppModel>, what: &str) {
    model.write().notify(NoticeKind::Info, tr("{what} arrives in a later version of Glyphra.", &[("what", what)]));
}

/// The last entry of the model list: connect another model or provider.
const ADD_PROVIDER: &str = "::add";

#[component]
pub fn AiPanel() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let (choices, has_model) = {
        let m = model.read();
        (m.model_choices(), m.settings.model_for("chat").is_some())
    };
    let current = choices.iter().find(|c| c.current).map(|c| format!("{}::{}", c.provider, c.model)).unwrap_or_default();
    let files = model.read().chat_files.clone();
    let soon = move |what: &str| soon(model, what);
    rsx! {
        aside { class: "ai-panel",
            div { class: "ai-head",
                h2 { {t("AI Chat")} }
                div { class: "ai-tools-row",
                    button { class: "icon-btn", title: t("History"), onclick: move |_| soon(&t("Chat history")), Icon { name: "history", size: 20 } }
                    button { class: "icon-btn", title: t("AI settings"), onclick: move |_| run(model, Command::OpenSettings(SettingsSection::AiModels)), Icon { name: "sliders", size: 20 } }
                    button { class: "icon-btn", title: t("Expand"), onclick: move |_| soon(&t("The expanded chat")), Icon { name: "expand", size: 20 } }
                    button { class: "icon-btn", title: t("Close"), onclick: move |_| run(model, Command::ToggleAiPanel), Icon { name: "x", size: 20 } }
                }
            }
            div { class: "ai-hero",
                svg { class: "hero", view_box: "0 0 320 190",
                    rect { x: "34", y: "30", width: "86", height: "112", rx: "8", class: "pg back" }
                    rect { x: "200", y: "30", width: "86", height: "112", rx: "8", class: "pg back" }
                    rect { x: "98", y: "8", width: "124", height: "148", rx: "8", class: "pg front" }
                    for i in 0..7 {
                        rect { key: "{i}", x: "114", y: "{26 + i * 15}", width: "92", height: "7", rx: "2", class: "ln" }
                    }
                    path { d: "M128 112 Q142 132 158 136 Q142 142 134 160 Q126 142 108 136 Q124 132 128 112Z", class: "spark" }
                }
                p { {t("Your AI Chat for documents")} }
            }
            div { class: "chips",
                for chip in CHIPS {
                    button { key: "{chip}", class: "chip", onclick: move |_| soon(&t(chip)), {t(chip)} }
                }
            }
            div { class: "composer",
                if !files.is_empty() {
                    div { class: "attached",
                        for f in files {
                            {
                                let path = f.clone();
                                let name = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                rsx! {
                                    span { key: "{f.display()}", class: "pill", title: "{f.display()}",
                                        span { class: "pill-name", "{name}" }
                                        button { class: "icon-btn", title: t("Remove attachment"), onclick: move |_| run(model, Command::DetachFromChat(path.clone())), Icon { name: "x", size: 12 } }
                                    }
                                }
                            }
                        }
                    }
                }
                textarea {
                    placeholder: t("Ask anything…"),
                    rows: "2",
                }
                div { class: "model-row",
                        if has_model {
                            button { class: "icon-btn", title: t("Cheaper model (same provider)"), onclick: move |_| run(model, Command::StepModel(Step::Cheaper)), Icon { name: "chev-down", size: 16 } }
                            Pick {
                                class: "",
                                up: true,
                                title: t("Model for AI Chat. Switching keeps your provider and API key."),
                                placeholder: t("Choose model"),
                                value: current.clone(),
                                items: choices
                                    .iter()
                                    .map(|c| PickItem { id: format!("{}::{}", c.provider, c.model), name: c.name.clone(), hint: c.price.clone(), group: c.provider_name.clone() })
                                    .chain(std::iter::once(PickItem { id: ADD_PROVIDER.to_string(), name: t("Add another provider…"), hint: String::new(), group: t("More") }))
                                    .collect::<Vec<_>>(),
                                onopen: move |_| run(model, Command::FetchModels(None)),
                                onpick: move |v: String| {
                                    if v == ADD_PROVIDER {
                                        run(model, Command::OpenSettings(SettingsSection::AiModels));
                                        model.write().begin_add_model();
                                    } else if let Some((p, m)) = v.split_once("::") {
                                        run(model, Command::UseModel { provider: p.to_string(), model: m.to_string() });
                                    }
                                },
                            }
                            button { class: "icon-btn up", title: t("More capable, more expensive model (same provider)"), onclick: move |_| run(model, Command::StepModel(Step::Pricier)), Icon { name: "chev-down", size: 16 } }
                        } else {
                            button { class: "model-pick", title: t("Connect an AI model"), onclick: move |_| run(model, Command::OpenSettings(SettingsSection::AiModels)),
                                span { {t("No model connected")} }
                            }
                        }
                }
                div { class: "composer-row",
                    button { class: "icon-btn", title: t("Add context"), onclick: move |_| soon(&t("Attachments")), Icon { name: "plus", size: 18 } }
                    button { class: "icon-btn", title: t("Tools"), onclick: move |_| soon(&t("Chat tools")), Icon { name: "terminal", size: 18 } }
                    span { class: "grow" }
                    button { class: "send", title: t("Send"), onclick: move |_| soon(&t("AI Chat")), Icon { name: "send", size: 18 } }
                }
            }
            div { class: "ai-foot", {t("AI Chat can make mistakes. Check important info.")} }
        }
    }
}
