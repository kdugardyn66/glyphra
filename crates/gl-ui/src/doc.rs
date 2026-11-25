//! A document in front: a read-only page until the editor arrives in version 0.2.0.

use crate::icons::Icon;
use dioxus::prelude::*;
use gl_core::{AppModel, preview};

#[component]
pub fn DocView() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let Some(tab) = m.active_tab.and_then(|i| m.tabs.get(i)) else {
        return rsx! {};
    };
    let (path, html) = (tab.path.display().to_string(), preview::to_html(&tab.content));
    drop(m);
    rsx! {
        div { class: "page doc-view",
            div { class: "banner",
                Icon { name: "info", size: 16 }
                span { class: "selectable", "Read-only preview of {path}. Editing arrives in version 0.2.0." }
            }
            div { class: "sheet",
                article { class: "paper md selectable", dangerous_inner_html: "{html}" }
            }
        }
    }
}
