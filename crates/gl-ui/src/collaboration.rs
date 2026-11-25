//! Collaboration: a placeholder until the server part exists.

use crate::icons::Icon;
use dioxus::prelude::*;
use gl_core::t;

#[component]
pub fn CollaborationPage() -> Element {
    rsx! {
        div { class: "page placeholder",
            div { class: "ph-ico", Icon { name: "collaboration", size: 44 } }
            h1 { {t("Collaboration")} }
            p { {t("Work on documents together with your team: shared folders, comments and chat, on your own server.")} }
            span { class: "soon", {t("Coming in a later version")} }
        }
    }
}
