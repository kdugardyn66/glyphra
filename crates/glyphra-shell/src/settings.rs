//! The Settings dialog: General, Keyboard, About and the licences.

use crate::icons::Icon;
use crate::model::AppModel;
use dioxus::prelude::*;
use glyphra_i18n::Direction;
use kd_commands::{Command, Keymap, Os, SettingsSection, bindable_commands, display_sequence};
use kd_store::{SOURCE_REPO, ThemePref, VERSION};

const MIT: &str = include_str!("../../../LICENSE-MIT");
const APACHE: &str = include_str!("../../../LICENSE-APACHE");

#[component]
pub fn SettingsDialog() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let Some(section) = m.settings else { return rsx! {} };
    let title = m.t("settings-title");
    let items: Vec<(SettingsSection, String)> =
        SettingsSection::ALL.into_iter().map(|s| (s, m.t(&format!("settings-{}", s.id())))).collect();
    let heading = m.t(&format!("settings-{}", section.id()));
    drop(m);
    rsx! {
        div { class: "scrim", onclick: move |_| model.write().settings = None,
            div {
                class: "settings",
                onclick: move |e| e.stop_propagation(),
                onkeydown: move |e| {
                    if e.key() == Key::Escape {
                        model.write().settings = None;
                    }
                },
                div { class: "nav-col",
                    h3 { "{title}" }
                    for (s , label) in items {
                        button {
                            key: "{s.id()}",
                            class: if s == section { "nav active" } else { "nav" },
                            onclick: move |_| model.write().settings = Some(s),
                            span { class: "t", "{label}" }
                        }
                    }
                }
                div { class: "pane",
                    h2 { "{heading}" }
                    match section {
                        SettingsSection::General => rsx! { General {} },
                        SettingsSection::Keyboard => rsx! { KeyboardPane {} },
                        SettingsSection::About => rsx! { About {} },
                        SettingsSection::LicenseMit => rsx! { div { class: "license selectable", "{MIT}" } },
                        SettingsSection::LicenseApache => rsx! { div { class: "license selectable", "{APACHE}" } },
                    }
                }
            }
        }
    }
}

#[component]
fn General() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let s = &m.state.settings;
    let (name, theme, language, docs) = (s.display_name.clone(), s.theme, s.language.clone(), s.documents_dir.display().to_string());
    let t = |id: &str| m.t(id);
    let (l_name, n_name, l_theme, l_lang, n_lang, l_docs, sys_default) = (
        t("settings-name"),
        t("settings-name-note"),
        t("settings-theme"),
        t("settings-language"),
        t("settings-language-note"),
        t("settings-docs-folder"),
        t("settings-language-system"),
    );
    let themes: Vec<(ThemePref, String, &'static str)> = vec![
        (ThemePref::System, t("theme-system"), "monitor"),
        (ThemePref::Light, t("theme-light"), "sun"),
        (ThemePref::Dark, t("theme-dark"), "moon"),
    ];
    let locales = m.catalog.locales();
    let rtl_note = m.i18n.info().direction == Direction::Rtl;
    drop(m);
    let _ = rtl_note;
    rsx! {
        div { class: "field",
            label { "{l_name}" }
            input { r#type: "text", value: "{name}", oninput: move |e| model.write().set_display_name(e.value()) }
            span { class: "note", "{n_name}" }
        }
        div { class: "field",
            label { "{l_theme}" }
            div { class: "seg",
                for (pref , label , icon) in themes {
                    button { key: "{label}", class: if theme == pref { "on" } else { "" }, onclick: move |_| model.write().choose_theme(pref),
                        span { style: "display:inline-flex;align-items:center;gap:6px", Icon { name: icon, size: 15 } "{label}" }
                    }
                }
            }
        }
        div { class: "field",
            label { "{l_lang}" }
            select { value: "{language}", onchange: move |e| model.write().choose_language(&e.value()),
                option { value: "", selected: language.is_empty(), "{sys_default}" }
                for l in locales {
                    option { key: "{l.tag}", value: "{l.tag}", selected: language == l.tag, "{l.native_name}" }
                }
            }
            span { class: "note", "{n_lang}" }
        }
        div { class: "field",
            label { "{l_docs}" }
            span { class: "selectable", "{docs}" }
        }
    }
}

#[component]
fn KeyboardPane() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let os = Os::current();
    let keymap = Keymap::defaults(os);
    let rows: Vec<(String, String)> = bindable_commands()
        .into_iter()
        .filter_map(|(c, _)| keymap.keys_for(&c).map(|k| (m.t(&c.label_key()), display_sequence(k, os))))
        .collect();
    let note = m.t("settings-keyboard-note");
    drop(m);
    let _ = Command::Refresh;
    rsx! {
        p { style: "color:var(--text-muted);margin-top:0", "{note}" }
        div { class: "kv",
            for (label , keys) in rows {
                span { key: "{label}{keys}", class: "k", "{label}" }
                kbd { "{keys}" }
            }
        }
    }
}

#[component]
fn About() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let (l_ver, l_cfg, l_src, l_packs, n_packs) =
        (m.t("about-version"), m.t("about-config"), m.t("about-source"), m.t("about-language-packs"), m.t("about-language-packs-note"));
    let tagline = m.t("splash-tagline");
    let cfg = m.store.dir().display().to_string();
    let packs = m.store.locales_dir().display().to_string();
    drop(m);
    rsx! {
        p { style: "font-size:17px;margin-top:0", "Glyphra — {tagline}" }
        div { class: "field",
            label { "{l_ver}" }
            span { "{VERSION}" }
        }
        div { class: "field",
            label { "{l_src}" }
            span { class: "selectable", "{SOURCE_REPO}" }
        }
        div { class: "field",
            label { "{l_cfg}" }
            span { class: "selectable", "{cfg}" }
        }
        div { class: "field",
            label { "{l_packs}" }
            span { class: "selectable", "{packs}" }
            span { class: "note", "{n_packs}" }
        }
    }
}
