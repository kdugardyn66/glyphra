//! A drop-down list whose closed state shows only the chosen name; the list itself shows each
//! entry with its price (a native `<select>` cannot show different text in the two places).

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct PickItem {
    pub id: String,
    pub name: String,
    /// Shown in the open list only.
    pub hint: String,
    /// Entries with the same group are listed under a heading.
    pub group: String,
}

#[component]
pub fn Pick(
    items: Vec<PickItem>,
    value: String,
    placeholder: String,
    up: bool,
    class: String,
    title: String,
    onpick: EventHandler<String>,
    /// Called when the list is opened.
    #[props(default)]
    onopen: EventHandler<()>,
) -> Element {
    let mut open = use_signal(|| false);
    let shown = items.iter().find(|i| i.id == value).map(|i| i.name.clone()).unwrap_or(placeholder);
    let mut groups: Vec<String> = Vec::new();
    for i in &items {
        if !groups.contains(&i.group) {
            groups.push(i.group.clone());
        }
    }
    rsx! {
        div { class: "pick {class}",
            button { class: "pick-btn", title: "{title}", r#type: "button", onclick: move |_| {
                    open.toggle();
                    if open() {
                        onopen.call(());
                    }
                },
                span { class: "pick-name", "{shown}" }
                span { class: "pick-caret", "▾" }
            }
            if open() {
                div { class: "pick-backdrop", onclick: move |_| open.set(false) }
                div { class: if up { "pick-list up" } else { "pick-list" },
                    for g in groups {
                        if !g.is_empty() {
                            div { key: "{g}", class: "pick-group", "{g}" }
                        }
                        for i in items.iter().filter(|i| i.group == g) {
                            button {
                                key: "{i.id}",
                                r#type: "button",
                                class: if i.id == value { "pick-item on" } else { "pick-item" },
                                onclick: {
                                    let id = i.id.clone();
                                    move |_| {
                                        open.set(false);
                                        onpick.call(id.clone());
                                    }
                                },
                                span { class: "pick-name", "{i.name}" }
                                span { class: "pick-hint", "{i.hint}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
