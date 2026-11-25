//! Open: recent folders and a Browse button.

use crate::icons::Icon;
use crate::shell::run;
use dioxus::prelude::*;
use gl_core::{AppModel, Command, t};
use gl_store::now;

#[component]
pub fn OpenPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let rows = model.read().recent_folder_rows(now());
    rsx! {
        div { class: "page open",
            h1 { {t("Recent folders")} }
            div { class: "table",
                div { class: "thead",
                    span { class: "c-name", {t("Name")} }
                    span { class: "c-loc wide", {t("Location")} }
                }
                if rows.is_empty() {
                    div { class: "empty", {t("No recent folders yet. Press Browse to open a file.")} }
                }
                for row in rows {
                    {
                        let path = row.path.clone();
                        rsx! {
                            button { key: "{row.path.display()}", class: "trow", title: "{row.path.display()}", onclick: move |_| run(model, Command::OpenDialog(Some(path.clone()))),
                                span { class: "c-name",
                                    span { class: "folder-ico", Icon { name: "folder", size: 22 } }
                                    span { class: "stem", "{row.stem}" }
                                }
                                span { class: "c-loc wide", "{row.location}" }
                            }
                        }
                    }
                }
            }
            div { class: "page-foot",
                button { class: "btn primary", onclick: move |_| run(model, Command::OpenDialog(None)), {t("Browse")} }
            }
        }
    }
}
