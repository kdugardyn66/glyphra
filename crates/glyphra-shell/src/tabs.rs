//! The tab strip: Home, one tab per open document, and the theme switch.

use crate::app::run;
use crate::icons::Icon;
use crate::model::AppModel;
use dioxus::prelude::*;
use kd_commands::Command;
use kd_store::{DocKind, ThemePref};

#[component]
pub fn TabStrip() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let home_label = m.t("tab-home");
    let theme_label = m.t("tab-theme");
    let theme_icon = match m.state.settings.theme {
        ThemePref::System => "monitor",
        ThemePref::Light => "sun",
        ThemePref::Dark => "moon",
    };
    let active = m.active;
    let tabs: Vec<(usize, String, String, Option<&'static str>)> = m
        .tabs
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, t)| {
            let group = t.path.as_deref().and_then(DocKind::from_path).map(|k| (k.badge(), k.group.id()));
            (i, t.title.clone(), group.map(|g| g.0).unwrap_or("").to_string(), group.map(|g| g.1))
        })
        .collect();
    rsx! {
        div { class: "tabstrip",
            button {
                class: if active == 0 { "tab active" } else { "tab" },
                onclick: move |_| run(model, Command::ActivateTab(0)),
                Icon { name: "home", size: 16 }
                span { class: "t", "{home_label}" }
            }
            for (i, title, badge, group) in tabs {
                button {
                    key: "{i}-{title}",
                    class: if active == i { "tab active" } else { "tab" },
                    title: "{title}",
                    onclick: move |_| run(model, Command::ActivateTab(i)),
                    onauxclick: move |_| run(model, Command::CloseTab(i)),
                    span { class: "badge-sm", style: "background:var(--g-{group.unwrap_or(\"office\")})", "{badge}" }
                    span { class: "t", "{title}" }
                    span {
                        class: "x",
                        onclick: move |e| {
                            e.stop_propagation();
                            run(model, Command::CloseTab(i));
                        },
                        Icon { name: "x", size: 12 }
                    }
                }
            }
            div { class: "grow" }
            button { class: "icon-btn", title: "{theme_label}", onclick: move |_| run(model, Command::CycleTheme),
                Icon { name: theme_icon, size: 17 }
            }
        }
    }
}
