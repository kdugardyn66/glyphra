//! The Settings dialog: General, Keyboard, About and the licences.

use crate::app::run_effect;
use crate::icons::Icon;
use crate::model::{AppModel, Effect};
use dioxus::prelude::*;
use glyphra_i18n::Direction;
use kd_ai::{KeySource, ProviderKind, Scope, Service};
use kd_commands::{Command, Keymap, Os, SettingsSection, bindable_commands, display_sequence};
use kd_store::{OpenIn, SOURCE_REPO, Store, ThemePref, VERSION};

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
    let (l_apply, l_close) = (m.t("settings-apply"), m.t("settings-close"));
    let dirty = m.settings_dirty();
    drop(m);
    // The dialog is modal: the scrim swallows clicks (it does not close the dialog) and only
    // Apply, Close or Escape leave it.
    rsx! {
        div { class: "scrim", onclick: move |e| e.stop_propagation(),
            div {
                class: "settings",
                tabindex: "0",
                onclick: move |e| e.stop_propagation(),
                onkeydown: move |e| {
                    if e.key() == Key::Escape {
                        model.write().close_settings();
                    }
                },
                div { class: "nav-col",
                    h3 { "{title}" }
                    for (s , label) in items {
                        button {
                            key: "{s.id()}",
                            class: if s == section { "nav active" } else { "nav" },
                            onclick: move |_| model.write().open_settings(s),
                            span { class: "t", "{label}" }
                        }
                    }
                }
                div { class: "pane",
                    div { class: "pane-body",
                        h2 { "{heading}" }
                        match section {
                            SettingsSection::General => rsx! { General {} },
                            SettingsSection::Ai => rsx! { AiPane {} },
                            SettingsSection::Keyboard => rsx! { KeyboardPane {} },
                            SettingsSection::About => rsx! { About {} },
                            SettingsSection::LicenseMit => rsx! { div { class: "license selectable", "{MIT}" } },
                            SettingsSection::LicenseApache => rsx! { div { class: "license selectable", "{APACHE}" } },
                        }
                    }
                    div { class: "foot",
                        button {
                            class: "btn primary",
                            disabled: !dirty,
                            onclick: move |_| {
                                let ok = model.write().apply_settings();
                                if ok {
                                    model.write().close_settings();
                                }
                            },
                            "{l_apply}"
                        }
                        button { class: "btn", onclick: move |_| model.write().close_settings(), "{l_close}" }
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
    let s = &m.draft;
    let (name, theme, language, docs, open_in) =
        (s.display_name.clone(), s.theme, s.language.clone(), s.documents_dir.display().to_string(), s.open_files_in);
    let is_default = s.documents_dir == Store::default_documents_dir();
    let t = |id: &str| m.t(id);
    let (l_choose, l_default, n_docs, l_open, n_open) =
        (t("settings-docs-choose"), t("settings-docs-default"), t("settings-docs-note"), t("settings-open-in"), t("settings-open-note"));
    let opens: Vec<(OpenIn, String)> = vec![(OpenIn::Tab, t("settings-open-tabs")), (OpenIn::Window, t("settings-open-windows"))];
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
            input { r#type: "text", value: "{name}", oninput: move |e| model.write().draft.display_name = e.value() }
            span { class: "note", "{n_name}" }
        }
        div { class: "field",
            label { "{l_theme}" }
            div { class: "seg",
                for (pref , label , icon) in themes {
                    button { key: "{label}", class: if theme == pref { "on" } else { "" }, onclick: move |_| model.write().draft.theme = pref,
                        span { style: "display:inline-flex;align-items:center;gap:6px", Icon { name: icon, size: 15 } "{label}" }
                    }
                }
            }
        }
        div { class: "field",
            label { "{l_lang}" }
            select { value: "{language}", onchange: move |e| model.write().draft.language = e.value(),
                option { value: "", selected: language.is_empty(), "{sys_default}" }
                for l in locales {
                    option { key: "{l.tag}", value: "{l.tag}", selected: language == l.tag, "{l.native_name}" }
                }
            }
            span { class: "note", "{n_lang}" }
        }
        div { class: "field",
            label { "{l_docs}" }
            div { class: "pathrow",
                span { class: "path selectable", title: "{docs}", "{docs}" }
                button { class: "btn", onclick: move |_| run_effect(model, Effect::PickDocumentsFolderDialog), "{l_choose}" }
                button {
                    class: "btn",
                    disabled: is_default,
                    onclick: move |_| model.write().draft_documents_dir(Store::default_documents_dir()),
                    "{l_default}"
                }
            }
            span { class: "note", "{n_docs}" }
        }
        div { class: "field",
            label { "{l_open}" }
            div { class: "seg",
                for (choice , label) in opens {
                    button {
                        key: "{label}",
                        class: if open_in == choice { "on" } else { "" },
                        onclick: move |_| model.write().draft.open_files_in = choice,
                        "{label}"
                    }
                }
            }
            span { class: "note", "{n_open}" }
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

#[component]
fn AiPane() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let t = |id: &str| m.t(id);
    let ai = &m.ai;
    let sel = ai.selected.clone();
    let providers: Vec<(String, String, bool)> =
        ai.draft.providers.iter().map(|p| (p.id.clone(), p.display_name(), p.usable_for_chat())).collect();
    let current = ai.draft.provider(&sel).cloned();
    let kinds: Vec<(&'static str, &'static str)> =
        ProviderKind::ALL.into_iter().filter(|k| k.service() == Service::Chat).map(|k| (k.default_id(), k.label())).collect();
    let choices = ai.choices(&ai.draft);
    let default_model = ai.draft.default_model.as_ref().map(|x| x.to_string()).unwrap_or_default();
    let scopes: Vec<(String, String, String)> = Scope::ALL
        .iter()
        .map(|s| {
            (s.id().to_string(), t(&format!("ai-scope-{}", s.id())), ai.draft.models.get(s.id()).map(|x| x.to_string()).unwrap_or_default())
        })
        .collect();
    let offered: Vec<String> = current
        .as_ref()
        .map(|p| {
            let mut v = p.models.clone();
            v.extend(ai.catalog.for_kind(p.kind).iter().map(|i| i.id.clone()));
            v
        })
        .unwrap_or_default();
    let key_note = current.as_ref().map(|p| match p.key_source() {
        KeySource::Stored => t("ai-key-stored"),
        KeySource::Env(v) => m.t_args("ai-key-env", &[("var", v.into())]),
        KeySource::None if p.kind.needs_key() => t("ai-key-none"),
        KeySource::None => t("ai-key-not-needed"),
    });
    let (h_prov, n_prov, l_add, l_add_btn, l_enabled, l_name, l_key, l_url, l_model, l_remove, l_ready, l_notready) = (
        t("ai-providers"),
        t("ai-providers-note"),
        t("ai-add-provider"),
        t("ai-add"),
        t("ai-enabled"),
        t("ai-name"),
        t("ai-key"),
        t("ai-url"),
        t("ai-default-model-of-provider"),
        t("ai-remove"),
        t("ai-provider-ready"),
        t("ai-provider-not-ready"),
    );
    let (h_models, n_models, l_default, l_use_default) = (t("ai-models"), t("ai-models-note"), t("ai-default-model"), t("ai-use-default"));
    drop(m);
    let mut add_kind = use_signal(String::new);
    let kind_for_add = kinds.first().map(|k| k.0.to_string()).unwrap_or_default();
    let adding = if add_kind.read().is_empty() { kind_for_add } else { add_kind.read().clone() };
    let adding_btn = adding.clone();
    rsx! {
        div { class: "field row",
            label { "{h_prov}" }
            span { class: "note", "{n_prov}" }
            div { class: "provlist",
                for (id , name , ok) in providers {
                    button {
                        key: "{id}",
                        class: if id == sel { "prov on" } else { "prov" },
                        onclick: {
                            let id = id.clone();
                            move |_| model.write().ai.selected = id.clone()
                        },
                        span { class: if ok { "dot ok" } else { "dot" } }
                        "{name}"
                    }
                }
            }
            div { class: "pathrow",
                span { class: "note", "{l_add}" }
                select { value: "{adding}", onchange: move |e| add_kind.set(e.value()),
                    for (id , label) in kinds {
                        option { key: "{id}", value: "{id}", selected: adding == id, "{label}" }
                    }
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        let kind = ProviderKind::ALL.into_iter().find(|k| k.default_id() == adding_btn);
                        if let Some(k) = kind {
                            model.write().ai.add_provider(k);
                        }
                    },
                    "{l_add_btn}"
                }
            }
        }
        if let Some(p) = current {
            div { class: "prov-detail",
                div { class: "prov-title",
                    strong { "{p.display_name()}" }
                    span { class: if p.usable_for_chat() { "badge ok" } else { "badge" }, if p.usable_for_chat() { "{l_ready}" } else { "{l_notready}" } }
                }
                div { class: "field row",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: p.enabled,
                            onchange: {
                                let id = p.id.clone();
                                move |e| {
                                    if let Some(x) = model.write().ai.draft.provider_mut(&id) {
                                        x.enabled = e.checked();
                                    }
                                }
                            },
                        }
                        " {l_enabled}"
                    }
                }
                div { class: "field",
                    label { "{l_name}" }
                    input {
                        r#type: "text",
                        value: "{p.name.clone().unwrap_or_default()}",
                        placeholder: "{p.kind.label()}",
                        oninput: {
                            let id = p.id.clone();
                            move |e| {
                                if let Some(x) = model.write().ai.draft.provider_mut(&id) {
                                    x.name = Some(e.value()).filter(|v| !v.trim().is_empty());
                                }
                            }
                        },
                    }
                }
                if p.kind.needs_key() {
                    div { class: "field",
                        label { "{l_key}" }
                        input {
                            r#type: "password",
                            autocomplete: "off",
                            value: "{p.api_key.clone().unwrap_or_default()}",
                            oninput: {
                                let id = p.id.clone();
                                move |e| {
                                    if let Some(x) = model.write().ai.draft.provider_mut(&id) {
                                        x.api_key = Some(e.value()).filter(|v| !v.trim().is_empty());
                                    }
                                }
                            },
                        }
                        if let Some(n) = key_note {
                            span { class: "note", "{n}" }
                        }
                    }
                }
                div { class: "field",
                    label { "{l_url}" }
                    input {
                        r#type: "text",
                        value: "{p.base_url.clone().unwrap_or_default()}",
                        placeholder: "{p.kind.default_base_url()}",
                        oninput: {
                            let id = p.id.clone();
                            move |e| {
                                if let Some(x) = model.write().ai.draft.provider_mut(&id) {
                                    x.base_url = Some(e.value()).filter(|v| !v.trim().is_empty());
                                }
                            }
                        },
                    }
                }
                div { class: "field",
                    label { "{l_model}" }
                    input {
                        r#type: "text",
                        list: "offered-models",
                        value: "{p.default_model.clone().unwrap_or_default()}",
                        oninput: {
                            let id = p.id.clone();
                            move |e| {
                                if let Some(x) = model.write().ai.draft.provider_mut(&id) {
                                    x.default_model = Some(e.value()).filter(|v| !v.trim().is_empty());
                                }
                            }
                        },
                    }
                    datalist { id: "offered-models",
                        for m in offered {
                            option { key: "{m}", value: "{m}" }
                        }
                    }
                }
                div { class: "tool-actions",
                    button {
                        class: "btn danger",
                        onclick: {
                            let id = p.id.clone();
                            move |_| model.write().ai.remove_provider(&id)
                        },
                        "{l_remove}"
                    }
                }
            }
        }
        div { class: "field row",
            label { "{h_models}" }
            span { class: "note", "{n_models}" }
        }
        div { class: "field",
            label { "{l_default}" }
            select { value: "{default_model}", onchange: move |e| model.write().ai.set_default_model(&e.value()),
                option { value: "", selected: default_model.is_empty(), "—" }
                for c in choices.clone() {
                    option { key: "{c.value}", value: "{c.value}", selected: default_model == c.value, "{c.label}" }
                }
            }
        }
        for (id , label , value) in scopes {
            div { key: "{id}", class: "field",
                label { "{label}" }
                select {
                    value: "{value}",
                    onchange: {
                        let id = id.clone();
                        move |e| model.write().ai.set_scope_model(&id, &e.value())
                    },
                    option { value: "", selected: value.is_empty(), "{l_use_default}" }
                    for c in choices.clone() {
                        option { key: "{c.value}", value: "{c.value}", selected: value == c.value, "{c.label}" }
                    }
                }
            }
        }
    }
}
