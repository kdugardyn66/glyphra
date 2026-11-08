//! The body of a document tab. In 0.1.0 Markdown files open as a read-only preview;
//! PDF and Drawing files wait for their modules.

use crate::app::run;
use crate::model::{AppModel, TabBody};
use dioxus::prelude::*;
use kd_commands::Command;

#[component]
pub fn DocTab() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let Some(tab) = m.tabs.get(m.active).cloned() else {
        return rsx! {};
    };
    let ro = m.t("doc-readonly");
    let path = tab.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
    match tab.body {
        TabBody::Markdown { html } => rsx! {
            main { class: "main",
                div { class: "doc",
                    span { class: "ro", title: "{path}", "{ro}" }
                    div { class: "md selectable", dangerous_inner_html: "{html}" }
                }
            }
        },
        TabBody::Soon(module) => {
            let name = m.t(&format!("module-{}", module.id()));
            let title = m.t_args("soon-title", &[("module", name.into())]);
            let text = m.t_args("soon-text", &[("release", module.release().into())]);
            let reveal = m.t("row-menu-reveal");
            let group = module.group().id();
            rsx! {
                main { class: "main",
                    div { class: "soon",
                        div { class: "lead", style: "background:var(--g-{group})", "{module.badge()}" }
                        h2 { "{title}" }
                        p { "{text}" }
                        p { class: "selectable", style: "color:var(--text-faint);font-size:12.5px", "{path}" }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                if let Some(p) = tab.path.clone() {
                                    run(model, Command::RevealInFolder(p));
                                }
                            },
                            "{reveal}"
                        }
                    }
                }
            }
        }
        TabBody::Unreadable(why) => {
            let title = m.t("doc-unreadable");
            rsx! {
                main { class: "main",
                    div { class: "soon",
                        h2 { "{title}" }
                        p { class: "selectable", "{why}" }
                    }
                }
            }
        }
        TabBody::Home => rsx! {},
    }
}
