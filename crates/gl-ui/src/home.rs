//! Home: "Create new" cards and the table of recent files.

use crate::icons::Icon;
use crate::shell::run;
use dioxus::prelude::*;
use gl_core::{AppModel, Command, t, tr, tx};
use gl_store::now;
use gl_templates::Kind;

/// (what it makes, label, badge, CSS class of the picture)
const CARDS: [(Kind, &str, &str, &str); 2] =
    [(Kind::Document, tx("Document"), "MD", "doc"), (Kind::Presentation, tx("Presentation"), "SLIDES", "slides")];

#[component]
pub fn HomePage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let rows = model.read().recent_file_rows(now(), 30);
    let mut selected = use_signal(|| None::<std::path::PathBuf>);
    let missing = rows.iter().filter(|r| !r.exists).count();
    let empty = rows.is_empty();
    rsx! {
        div { class: "page home",
            div { class: "create",
                h1 { {t("Create new")} }
                div { class: "cards",
                    for (kind , label , badge , art) in CARDS {
                        button { key: "{label}", class: "doc-card", onclick: move |_| run(model, Command::NewDocument(kind)),
                            div { class: "art {art}",
                                span { class: "badge", "{badge}" }
                                div { class: "mini",
                                    for i in 0..16 {
                                        span { key: "{i}", class: "bar" }
                                    }
                                }
                            }
                            span { class: "card-label", {t(label)} }
                        }
                    }
                }
            }
            div { class: "list-tools",
                button { class: "btn", disabled: missing == 0, title: t("Remove the files that no longer exist from the list"), onclick: move |_| run(model, Command::CleanupRecent), {t("Cleanup")} }
                button { class: "btn", disabled: empty, title: t("Remove everything from the list (the files stay on disk)"), onclick: move |_| run(model, Command::ClearRecent), {t("Clear all")} }
            }
            div { class: "table",
                div { class: "thead",
                    span { class: "c-name", {t("Name")} }
                    span { class: "c-loc", {t("Location")} }
                    span { class: "c-when", {t("Last opened")} }
                    span { class: "c-tools" }
                }
                if rows.is_empty() {
                    div { class: "empty", {t("Nothing opened yet. Create a document above, pick a template, or open a file.")} }
                }
                for row in rows {
                    {
                        let is_selected = selected().as_ref() == Some(&row.path);
                        let class = format!("trow{}{}", if is_selected { " selected" } else { "" }, if row.exists { "" } else { " missing" });
                        let (p_sel, p_open, p_key) = (row.path.clone(), row.path.clone(), row.path.clone());
                        let (p_chat, p_dir, p_pin, p_del) = (row.path.clone(), row.path.clone(), row.path.clone(), row.path.clone());
                        let (exists, pinned) = (row.exists, row.pinned);
                        let pin_title = if pinned { t("Unpin") } else { t("Pin") };
                        let title = if exists { row.path.display().to_string() } else { tr("{path} (no longer exists)", &[("path", &row.path.display().to_string())]) };
                        rsx! {
                            div {
                                key: "{row.path.display()}",
                                class: "{class}",
                                role: "row",
                                tabindex: "0",
                                title: "{title}",
                                onclick: move |_| selected.set(Some(p_sel.clone())),
                                ondoubleclick: move |_| run(model, Command::OpenPath(p_open.clone())),
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter {
                                        run(model, Command::OpenPath(p_key.clone()));
                                    }
                                },
                                span { class: "c-name",
                                    span { class: "file-ico" }
                                    span { class: "stem", "{row.stem}" }
                                    span { class: "ext", "{row.ext}" }
                                    if pinned {
                                        span { class: "pinned-mark", title: t("Pinned"), Icon { name: "pin", size: 14 } }
                                    }
                                }
                                span { class: "c-loc", "{row.location}" }
                                span { class: "c-when", "{row.when}" }
                                span { class: "c-tools",
                                    button {
                                        class: "icon-btn",
                                        title: t("Open in Chat"),
                                        disabled: !exists,
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            run(model, Command::OpenInChat(p_chat.clone()));
                                        },
                                        ondoubleclick: |e| e.stop_propagation(),
                                        Icon { name: "chat", size: 18 }
                                    }
                                    button {
                                        class: "icon-btn",
                                        title: t("Show in Folder"),
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            run(model, Command::RevealPath(p_dir.clone()));
                                        },
                                        ondoubleclick: |e| e.stop_propagation(),
                                        Icon { name: "folder-open", size: 18 }
                                    }
                                    button {
                                        class: "icon-btn",
                                        title: "{pin_title}",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            run(model, Command::TogglePin(p_pin.clone()));
                                        },
                                        ondoubleclick: |e| e.stop_propagation(),
                                        Icon { name: "pin", size: 18 }
                                    }
                                    button {
                                        class: "icon-btn",
                                        title: t("Remove from list"),
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            run(model, Command::ForgetRecent(p_del.clone()));
                                        },
                                        ondoubleclick: |e| e.stop_propagation(),
                                        Icon { name: "minus-circle", size: 18 }
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
