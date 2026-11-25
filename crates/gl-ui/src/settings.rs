//! Settings: General, Region, AI Models, Model Assignment and MCP Servers. General waits for Apply;
//! Region and the AI pages save as soon as something changes.

use crate::icons::Icon;
use crate::pick::{Pick, PickItem};
use crate::shell::run;
use dioxus::prelude::*;
use gl_core::providers::{Protocol, Region};
use gl_core::{AppModel, Command, LANGUAGES, SettingsSection, t, tr};
use gl_store::region::{DECIMALS, LONG_DATES, LONG_TIMES, SHORT_DATES, SHORT_TIMES, THOUSANDS, countries, currencies, currency_info};
use gl_store::{AI_TASKS, Appearance, CurrencyPattern, Length, OpenIn, SpellingDetection, Temperature, ThemeFamily, Weekday};

#[component]
pub fn SettingsPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let section = model.read().section;
    rsx! {
        div { class: "settings",
            div { class: "set-nav",
                h2 { {t("Settings")} }
                for s in SettingsSection::ALL {
                    button {
                        key: "{s.label()}",
                        class: if s == section { "set-item active" } else { "set-item" },
                        onclick: move |_| run(model, Command::OpenSettings(s)),
                        {t(s.label())}
                    }
                }
            }
            div { class: "set-pane",
                match section {
                    SettingsSection::General => rsx! { General {} },
                    SettingsSection::Region => rsx! { RegionPage {} },
                    SettingsSection::AiModels => rsx! { AiModels {} },
                    SettingsSection::ModelAssignment => rsx! { ModelAssignment {} },
                    SettingsSection::McpServers => rsx! { McpServers {} },
                }
            }
        }
    }
}

#[component]
fn Toggle(on: bool, label: String, onchange: EventHandler<bool>) -> Element {
    rsx! {
        button {
            class: if on { "toggle on" } else { "toggle" },
            role: "switch",
            "aria-checked": "{on}",
            "aria-label": "{label}",
            onclick: move |_| onchange.call(!on),
            span { class: "knob" }
        }
    }
}

/// Remounts the page when the model asks for fresh text inputs (reset, provider change): the
/// inputs are uncontrolled, and only a keyed list item is rebuilt reliably by Dioxus.
#[component]
fn General() -> Element {
    let form_gen = use_context::<Signal<AppModel>>().read().form_gen;
    rsx! {
        for g in [form_gen] {
            GeneralBody { key: "{g}" }
        }
    }
}

#[component]
fn GeneralBody() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let d = m.draft.clone();
    let dirty = m.general_dirty();
    drop(m);
    rsx! {
        div { class: "pane-scroll",
            div { class: "pane-inner",
                div { class: "pane-top",
                    h1 { {t("General")} }
                    button { class: "link", onclick: move |_| model.write().reset_general(), {t("Reset to default")} }
                }
                div { class: "field",
                    label { r#for: "user-name", {t("User Name")} }
                    input { id: "user-name", r#type: "text", initial_value: "{d.user_name}", oninput: move |e| model.write().draft.user_name = e.value() }
                }
                div { class: "field",
                    label { {t("Interface language *")} }
                    select { onchange: move |e| model.write().draft.language = e.value(),
                        for (tag , name) in LANGUAGES {
                            option { key: "{tag}", value: "{tag}", selected: d.language == tag, "{name}" }
                        }
                    }
                }
                div { class: "field",
                    label { {t("Appearance")} }
                    div { class: "appearance",
                        for a in Appearance::ALL {
                            button {
                                key: "{a.label()}",
                                r#type: "button",
                                class: if d.appearance == a { "ap-tile on" } else { "ap-tile" },
                                onclick: move |_| model.write().draft.appearance = a,
                                div { class: "thumb",
                                    if a != Appearance::Dark {
                                        div { class: "half lt",
                                            i {}
                                            i {}
                                        }
                                    }
                                    if a != Appearance::Light {
                                        div { class: "half dk",
                                            i {}
                                            i {}
                                        }
                                    }
                                }
                                span { {t(a.label())} }
                            }
                        }
                    }
                }
                div { class: "field",
                    label { {t("Light theme")} }
                    select {
                        onchange: move |e| {
                            if let Some(t) = ThemeFamily::ALL.into_iter().find(|t| t.light_id() == e.value()) {
                                model.write().draft.light_theme = t;
                            }
                        },
                        for t in ThemeFamily::ALL {
                            option { key: "{t.light_id()}", value: "{t.light_id()}", selected: d.light_theme == t, "{t.light_label()}" }
                        }
                    }
                }
                div { class: "field",
                    label { {t("Dark theme")} }
                    select {
                        onchange: move |e| {
                            if let Some(t) = ThemeFamily::ALL.into_iter().find(|t| t.dark_id() == e.value()) {
                                model.write().draft.dark_theme = t;
                            }
                        },
                        for t in ThemeFamily::ALL {
                            option { key: "{t.dark_id()}", value: "{t.dark_id()}", selected: d.dark_theme == t, "{t.dark_label()}" }
                        }
                    }
                }
                div { class: "field",
                    label { {t("Open file")} }
                    select {
                        onchange: move |e| {
                            if let Some(o) = OpenIn::ALL.into_iter().find(|o| o.label() == e.value()) {
                                model.write().draft.open_file_in = o;
                            }
                        },
                        for o in OpenIn::ALL {
                            option { key: "{o.label()}", value: "{o.label()}", selected: d.open_file_in == o, {t(o.label())} }
                        }
                    }
                    if d.open_file_in == OpenIn::NewWindow {
                        span { class: "note span2", {t("Documents still open in their own tab; separate windows arrive with the editor.")} }
                    }
                }
                div { class: "field",
                    label { {t("Spelling language detection")} }
                    select {
                        onchange: move |e| {
                            if let Some(s) = SpellingDetection::ALL.into_iter().find(|s| s.label() == e.value()) {
                                model.write().draft.spelling = s;
                            }
                        },
                        for s in SpellingDetection::ALL {
                            option { key: "{s.label()}", value: "{s.label()}", selected: d.spelling == s, {t(s.label())} }
                        }
                    }
                }
                div { class: "switch-row",
                    div {
                        div { class: "sw-title", {t("Use graphics acceleration when available *")} }
                        div { class: "sw-desc", {t("Improves performance on most devices. Disable if you experience display issues.")} }
                    }
                    Toggle { on: d.gpu_acceleration, label: t("Use graphics acceleration"), onchange: move |v| model.write().draft.gpu_acceleration = v }
                }
                div { class: "switch-row",
                    div {
                        div { class: "sw-title", {t("Enable AI features *")} }
                        div { class: "sw-desc", {t("Enable AI in the application to add models and use AI-powered functionality")} }
                    }
                    Toggle { on: d.ai_enabled, label: t("Enable AI features"), onchange: move |v| model.write().draft.ai_enabled = v }
                }
            }
        }
        div { class: "pane-foot",
            button { class: "btn primary", disabled: !dirty, onclick: move |_| model.write().apply_general(), {t("Apply")} }
            span { class: "foot-note", {t("* — Option will be applied after the app restart")} }
        }
    }
}

/// Remounts the page when the model asks for fresh text inputs (reset, provider change): the
/// inputs are uncontrolled, and only a keyed list item is rebuilt reliably by Dioxus.
#[component]
fn AiModels() -> Element {
    let form_gen = use_context::<Signal<AppModel>>().read().form_gen;
    rsx! {
        for g in [form_gen] {
            AiModelsBody { key: "{g}" }
        }
    }
}

#[component]
fn AiModelsBody() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let mut menu = use_signal(|| None::<String>);
    let mut show_key = use_signal(|| false);
    let mut other_id = use_signal(|| false);
    let m = model.read();
    let models = m.settings.models.clone();
    let prices: Vec<String> = models.iter().map(|p| m.model_price(p)).collect();
    let default = m.settings.default_model.clone();
    let form = m.add_model.clone();
    let custom = m.custom_form.clone();
    let providers = m.provider_options();
    let groups = m.provider_groups();
    let help = form.as_ref().map(|f| m.provider_help(&f.provider)).unwrap_or_default();
    let hint = form.as_ref().map(|f| m.endpoint_hint(&f.provider)).unwrap_or_default();
    let choices = form.as_ref().map(|f| m.provider_model_labels(&f.provider)).unwrap_or_default();
    let custom_list = m.settings.custom_providers.clone();
    let models_note = form.as_ref().map(|f| m.models_note(&f.provider)).unwrap_or_default();
    drop(m);
    let provider_name = |id: &str| providers.iter().find(|(i, _)| i == id).map(|(_, n)| n.clone()).unwrap_or_else(|| id.to_string());
    let (key_type, key_title, key_icon) = if show_key() { ("text", t("Hide key"), "eye") } else { ("password", t("Show key"), "eye-off") };
    let rows: Vec<_> = models
        .iter()
        .zip(prices)
        .map(|(p, price)| (p.id.clone(), p.name.clone(), provider_name(&p.provider), default.as_deref() == Some(p.id.as_str()), price))
        .collect();
    rsx! {
        div { class: "pane-scroll",
            div { class: "pane-inner wide",
                h1 { {t("AI Models")} }
                p { class: "lead", {t("Connect your own AI source — either via API key from a provider or a locally hosted model")} }
                div { class: "btn-row",
                    button { class: "btn primary", onclick: move |_| model.write().begin_add_model(), {t("Add Model")} }
                    button { class: "btn", onclick: move |_| model.write().begin_custom_provider(), {t("Custom Providers")} }
                }
                if let Some(f) = form {
                    {
                        let valid = f.valid();
                        let has_choices = !choices.is_empty();
                        rsx! {
                            div { class: "card form",
                                h3 { if f.editing.is_some() { {t("Edit AI Model")} } else { {t("Connect new AI Model")} } }
                                div { class: "form-link",
                                    button { class: "link", onclick: move |_| model.write().begin_custom_provider(), {t("Import Custom Provider")} }
                                    span { title: t("Add any OpenAI-compatible server as a provider"), Icon { name: "info", size: 16 } }
                                }
                                div { class: "frow",
                                    label { {t("Provider")} }
                                    select {
                                        onchange: move |e| {
                                            let id = e.value();
                                            model.write().select_provider(&id);
                                            run(model, Command::FetchModels(Some(id)));
                                        },
                                        option { value: "", selected: f.provider.is_empty(), {t("Select provider")} }
                                        for (label , list) in groups.clone() {
                                            optgroup { key: "{label}", label: t(label),
                                                for (id , name) in list {
                                                    option { key: "{id}", value: "{id}", selected: f.provider == id, "{name}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if !help.is_empty() {
                                    p { class: "note provider-help", "{help}" }
                                }
                                div { class: "frow",
                                    label { {t("API key")} }
                                    div { class: "with-btn",
                                        input {
                                            r#type: "{key_type}",
                                            placeholder: if f.connected { t("Saved — leave empty to keep it") } else { t("Enter API key") },
                                            autocomplete: "off",
                                            value: "{f.api_key}",
                                            oninput: move |e| {
                                                if let Some(f) = model.write().add_model.as_mut() {
                                                    f.api_key = e.value();
                                                }
                                            },
                                        }
                                        button { class: "icon-btn", title: "{key_title}", onclick: move |_| show_key.set(!show_key()),
                                            Icon { name: key_icon, size: 18 }
                                        }
                                    }
                                }
                                div { class: "frow",
                                    label { {t("Base URL")} }
                                    input {
                                        r#type: "text",
                                        placeholder: "{hint}",
                                        initial_value: "{f.base_url}",
                                        oninput: move |e| {
                                            if let Some(f) = model.write().add_model.as_mut() {
                                                f.base_url = e.value();
                                            }
                                        },
                                    }
                                }
                                div { class: "frow",
                                    label { {t("Model")} }
                                    if has_choices {
                                        div { class: "field-stack",
                                        Pick {
                                            class: "",
                                            up: false,
                                            title: t("Model"),
                                            placeholder: if other_id() { t("Other — type a model ID…") } else { t("Select model") },
                                            onopen: {
                                                let p = f.provider.clone();
                                                move |_| run(model, Command::FetchModels(Some(p.clone())))
                                            },
                                            value: if other_id() { String::new() } else { f.model.clone() },
                                            items: choices
                                                .iter()
                                                .map(|(id, name, price)| PickItem { id: id.clone(), name: name.clone(), hint: price.clone(), group: String::new() })
                                                .chain([PickItem { id: "__other__".into(), name: "Other — type a model ID…".into(), hint: String::new(), group: String::new() }])
                                                .collect::<Vec<_>>(),
                                            onpick: move |v: String| {
                                                other_id.set(v == "__other__");
                                                if let Some(f) = model.write().add_model.as_mut() {
                                                    f.model = if v == "__other__" { String::new() } else { v };
                                                }
                                            },
                                        }
                                        if other_id() {
                                            input {
                                                r#type: "text",
                                                class: "other-id",
                                                placeholder: t("Model ID, as the provider names it (e.g. a model released today)"),
                                                oninput: move |e| {
                                                    if let Some(f) = model.write().add_model.as_mut() {
                                                        f.model = e.value();
                                                    }
                                                },
                                            }
                                        }
                                        }
                                    } else {
                                        input {
                                            r#type: "text",
                                            placeholder: if f.provider.is_empty() { t("Select provider first") } else { t("Enter model name") },
                                            disabled: f.provider.is_empty(),
                                            initial_value: "{f.model}",
                                            oninput: move |e| {
                                                if let Some(f) = model.write().add_model.as_mut() {
                                                    f.model = e.value();
                                                }
                                            },
                                        }
                                    }
                                }
                                if !f.provider.is_empty() {
                                    div { class: "frow models-note",
                                        label {}
                                        div { class: "field-stack",
                                            if !models_note.is_empty() {
                                                p { class: "note", "{models_note}" }
                                            }
                                            {
                                                let p = f.provider.clone();
                                                rsx! {
                                                    button { class: "link", onclick: move |_| run(model, Command::ReloadModels(p.clone())), {t("Load models from the provider")} }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "frow",
                                    label { {t("Profile name")} }
                                    input {
                                        r#type: "text",
                                        placeholder: t("Enter name"),
                                        initial_value: "{f.profile_name}",
                                        oninput: move |e| {
                                            if let Some(f) = model.write().add_model.as_mut() {
                                                f.profile_name = e.value();
                                            }
                                        },
                                    }
                                }
                                div { class: "form-actions",
                                    button { class: "btn", onclick: move |_| model.write().cancel_add_model(), {t("Cancel")} }
                                    button { class: "btn primary", disabled: !valid, onclick: move |_| {
                                        model.write().submit_add_model();
                                        run(model, Command::FetchModels(None));
                                    }, if f.editing.is_some() { {t("Save")} } else { {t("Add Model")} } }
                                }
                            }
                        }
                    }
                }
                if let Some(c) = custom {
                    div { class: "card form",
                        h3 { {t("Custom Providers")} }
                        p { class: "note", {t("A provider of your own: a company gateway, vLLM, LiteLLM … Choose the API it speaks and where it is based.")} }
                        div { class: "frow",
                            label { {t("Name")} }
                            input {
                                r#type: "text",
                                placeholder: t("Enter name"),
                                initial_value: "{c.name}",
                                oninput: move |e| {
                                    if let Some(f) = model.write().custom_form.as_mut() {
                                        f.name = e.value();
                                    }
                                },
                            }
                        }
                        div { class: "frow",
                            label { {t("Base URL")} }
                            input {
                                r#type: "text",
                                placeholder: "https://…/v1",
                                initial_value: "{c.base_url}",
                                oninput: move |e| {
                                    if let Some(f) = model.write().custom_form.as_mut() {
                                        f.base_url = e.value();
                                    }
                                },
                            }
                        }
                        div { class: "frow",
                            label { {t("API")} }
                            select {
                                onchange: move |e| {
                                    if let Some(f) = model.write().custom_form.as_mut() {
                                        f.protocol = e.value();
                                    }
                                },
                                for p in Protocol::CHAT {
                                    option { key: "{p.id()}", value: "{p.id()}", selected: c.protocol == p.id() || (c.protocol.is_empty() && p == Protocol::OpenAiChat), "{p.label()}" }
                                }
                            }
                        }
                        div { class: "frow",
                            label { {t("Region")} }
                            select {
                                onchange: move |e| {
                                    if let Some(f) = model.write().custom_form.as_mut() {
                                        f.region = e.value();
                                    }
                                },
                                for r in Region::ALL {
                                    option { key: "{r.id()}", value: "{r.id()}", selected: c.region == r.id() || (c.region.is_empty() && r == Region::Local), {t(r.label())} }
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn", onclick: move |_| model.write().cancel_custom_provider(), {t("Cancel")} }
                            button { class: "btn primary", disabled: c.name.trim().is_empty() || c.base_url.trim().is_empty(), onclick: move |_| model.write().submit_custom_provider(), {t("Add Provider")} }
                        }
                        if !custom_list.is_empty() {
                            div { class: "chip-list",
                                for p in custom_list {
                                    {
                                        let name = p.name.clone();
                                        rsx! {
                                            span { key: "{p.name}", class: "pill", title: "{p.base_url} · {p.protocol}",
                                                "{p.name}"
                                                button { class: "icon-btn", title: t("Remove provider"), onclick: move |_| model.write().delete_custom_provider(&name), Icon { name: "x", size: 12 } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if rows.is_empty() {
                    div { class: "empty left", {t("No models connected yet. Press Add Model to connect one.")} }
                }
                for (id , name , provider , is_default , price) in rows {
                    {
                        let (i1, i2, i3, i4) = (id.clone(), id.clone(), id.clone(), id.clone());
                        let open = menu() == Some(id.clone());
                        let initial = provider.chars().next().map(|c| c.to_string()).unwrap_or_default();
                        rsx! {
                            div { key: "{id}", class: "model-card",
                                span { class: "logo-tile", "{initial}" }
                                div { class: "mc-tx",
                                    div { class: "mc-name",
                                        "{name}"
                                        if is_default {
                                            span { class: "tag", {t("Default")} }
                                        }
                                    }
                                    div { class: "mc-sub",
                                        "{provider}"
                                        if !price.is_empty() {
                                            " · {price}"
                                        }
                                    }
                                }
                                button { class: "icon-btn", title: t("More"), onclick: move |_| menu.set(if open { None } else { Some(i1.clone()) }), Icon { name: "more", size: 20 } }
                                if open {
                                    div { class: "backdrop", onclick: move |_| menu.set(None) }
                                    div { class: "popover",
                                        button { onclick: move |_| {
                                                model.write().begin_edit_model(&i4);
                                                run(model, Command::FetchModels(None));
                                                menu.set(None);
                                            },
                                            {t("Edit")}
                                        }
                                        if !is_default {
                                            button { onclick: move |_| {
                                                    model.write().make_default_model(&i2);
                                                    menu.set(None);
                                                },
                                                {t("Make default")}
                                            }
                                        }
                                        button { class: "danger", onclick: move |_| {
                                                model.write().delete_model(&i3);
                                                menu.set(None);
                                            },
                                            {t("Delete")}
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
}

/// Remounts the page when the model asks for fresh text inputs (reset, provider change): the
/// inputs are uncontrolled, and only a keyed list item is rebuilt reliably by Dioxus.
#[component]
fn ModelAssignment() -> Element {
    let form_gen = use_context::<Signal<AppModel>>().read().form_gen;
    rsx! {
        for g in [form_gen] {
            ModelAssignmentBody { key: "{g}" }
        }
    }
}

#[component]
fn ModelAssignmentBody() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let models: Vec<(String, String)> = m.settings.models.iter().map(|p| (p.id.clone(), p.name.clone())).collect();
    let default = m.settings.default_model.clone();
    let assigned: Vec<(&'static str, &'static str, Option<String>)> =
        AI_TASKS.iter().map(|(id, label)| (*id, *label, m.settings.assignments.get(*id).cloned())).collect();
    drop(m);
    rsx! {
        div { class: "pane-scroll",
            div { class: "pane-inner wide",
                h1 { {t("Model Assignment")} }
                p { class: "lead", {t("Assign AI models to different tasks to customize performance and cost.")} }
                if models.is_empty() {
                    div { class: "empty left",
                        {t("No models connected yet. ")}
                        button { class: "link", onclick: move |_| run(model, Command::OpenSettings(SettingsSection::AiModels)), {t("Add one under AI Models.")} }
                    }
                }
                div { class: "frow big",
                    label { class: "strong", {t("Default model")} }
                    select {
                        disabled: models.is_empty(),
                        onchange: move |e| {
                            let v = e.value();
                            model.write().set_default_model(if v.is_empty() { None } else { Some(v) });
                        },
                        if models.is_empty() {
                            option { value: "", {t("No models")} }
                        }
                        for (id , name) in models.clone() {
                            option { key: "{id}", value: "{id}", selected: default.as_deref() == Some(id.as_str()), "{name}" }
                        }
                    }
                }
                h3 { class: "sub", {t("Custom models per task")} }
                p { class: "note", {t("Use a different model for specific tasks instead of the default")} }
                for (task , label , current) in assigned {
                    div { key: "{task}", class: "frow big",
                        label { {t(label)} }
                        select {
                            disabled: models.is_empty(),
                            onchange: move |e| {
                                let v = e.value();
                                model.write().assign_model(task, if v.is_empty() { None } else { Some(v) });
                            },
                            option { value: "", selected: current.is_none(), {t("Default model")} }
                            for (id , name) in models.clone() {
                                option { key: "{id}", value: "{id}", selected: current.as_deref() == Some(id.as_str()), "{name}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Remounts the page when the model asks for fresh text inputs (reset, provider change): the
/// inputs are uncontrolled, and only a keyed list item is rebuilt reliably by Dioxus.
#[component]
fn McpServers() -> Element {
    let form_gen = use_context::<Signal<AppModel>>().read().form_gen;
    rsx! {
        for g in [form_gen] {
            McpServersBody { key: "{g}" }
        }
    }
}

#[component]
fn McpServersBody() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let editor = m.mcp.clone();
    let names = m.mcp_server_names();
    drop(m);
    rsx! {
        div { class: "pane-scroll",
            div { class: "pane-inner wide",
                h1 { {t("MCP Servers")} }
                p { class: "lead", {t("Set up MCP servers and enable their tools for use in AI chat.")} }
                match editor {
                    None => rsx! {
                        div { class: "btn-row",
                            button { class: "btn primary", onclick: move |_| model.write().open_mcp_editor(), {t("Edit configuration")} }
                        }
                        if names.is_empty() {
                            div { class: "empty left", {t("No MCP servers configured.")} }
                        } else {
                            div { class: "chip-list",
                                for n in names {
                                    span { key: "{n}", class: "pill", "{n}" }
                                }
                            }
                        }
                    },
                    Some(e) => {
                        let lines = e.text.lines().count().max(1);
                        rsx! {
                            div { class: "card form",
                                h3 { {t("Edit configuration")} }
                                div { class: "code-head",
                                    span { {t("Enter your JSON configuration:")} }
                                    span { "json" }
                                }
                                div { class: "code",
                                    div { class: "gutter", aria_hidden: "true",
                                        for n in 1..=lines {
                                            div { key: "{n}", "{n}" }
                                        }
                                    }
                                    textarea {
                                        spellcheck: "false",
                                        wrap: "off",
                                        rows: "{lines.max(8)}",
                                        initial_value: "{e.text}",
                                        oninput: move |ev| model.write().set_mcp_text(ev.value()),
                                    }
                                }
                                if let Some(err) = e.error {
                                    div { class: "form-error", role: "alert", "{err}" }
                                }
                                div { class: "form-actions",
                                    button { class: "btn", onclick: move |_| model.write().cancel_mcp_editor(), {t("Cancel")} }
                                    button { class: "btn primary", onclick: move |_| model.write().save_mcp(), {t("Save")} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn symbol_name(s: &str) -> String {
    match s {
        "." => format!(". ({})", t("period")),
        "," => format!(", ({})", t("comma")),
        "'" => format!("' ({})", t("apostrophe")),
        " " => format!("({})", t("space")),
        other => other.to_string(),
    }
}

/// Region: the country and the formats that follow from it, which can be changed by hand.
#[component]
fn RegionPage() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let rs = model.read().settings.region.clone();
    let f = gl_core::region::effective(&rs);
    let sys = gl_core::region::system_country();
    let sys_name = countries().iter().find(|c| c.code == sys).map(|c| c.name.as_str()).unwrap_or(sys.as_str());
    let off = !rs.manual;
    let now = gl_core::region::now();
    let eg = |p: &str| tr("Example: {sample}", &[("sample", &gl_core::region::pattern(p, now))]);
    let with_current = |list: &[&str], current: &str| -> Vec<String> {
        let mut v: Vec<String> = list.iter().map(|p| p.to_string()).collect();
        if !v.iter().any(|p| p == current) {
            v.insert(0, current.to_string());
        }
        v
    };
    let short_times = with_current(&SHORT_TIMES, &f.short_time);
    let long_times = with_current(&LONG_TIMES, &f.long_time);
    rsx! {
        div { class: "pane-scroll",
            div { class: "pane-inner",
                div { class: "pane-top",
                    h1 { {t("Region")} }
                    button { class: "link", onclick: move |_| model.write().reset_region(), {t("Reset to default")} }
                }
                div { class: "field",
                    label { {t("Region")} }
                    select { onchange: move |e| model.write().set_region(&e.value()),
                        option { value: "system", selected: rs.region == "system", {tr("System ({country})", &[("country", sys_name)])} }
                        for c in countries() {
                            option { key: "{c.code}", value: "{c.code}", selected: rs.region == c.code, "{c.name}" }
                        }
                    }
                    span { class: "note span2", {t("Formats follow the chosen region. Tick “Change settings manually” to adjust them yourself.")} }
                }
                div { class: "check-row",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: rs.manual,
                            onchange: move |e| model.write().set_region_manual(e.checked()),
                        }
                        span { {t("Change settings manually")} }
                    }
                }
                div { class: "field",
                    label { {t("Unit of temperature")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            if let Some(u) = Temperature::ALL.into_iter().find(|u| u.label() == e.value()) {
                                model.write().edit_region(|r| r.temperature = u);
                            }
                        },
                        for u in Temperature::ALL {
                            option { key: "{u.label()}", value: "{u.label()}", selected: f.temperature == u, {t(u.label())} }
                        }
                    }
                }
                div { class: "field",
                    label { {t("Unit of measurement")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            if let Some(u) = Length::ALL.into_iter().find(|u| u.label() == e.value()) {
                                model.write().edit_region(|r| r.length = u);
                            }
                        },
                        for u in Length::ALL {
                            option { key: "{u.label()}", value: "{u.label()}", selected: f.length == u, {t(u.label())} }
                        }
                    }
                }
                div { class: "field",
                    label { {t("First day of the week")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            if let Some(d) = Weekday::ALL.into_iter().find(|d| d.index().to_string() == e.value()) {
                                model.write().edit_region(|r| r.first_day = d);
                            }
                        },
                        for d in Weekday::ALL {
                            option { key: "{d.index()}", value: "{d.index()}", selected: f.first_day == d, "{gl_core::region::weekday_name(d.index())}" }
                        }
                    }
                }
                datalist { id: "dl-short-date",
                    for p in SHORT_DATES {
                        option { key: "{p}", value: "{p}" }
                    }
                }
                datalist { id: "dl-long-date",
                    for p in LONG_DATES {
                        option { key: "{p}", value: "{p}" }
                    }
                }
                div { class: "field",
                    label { {t("Short date format")} }
                    input {
                        r#type: "text",
                        list: "dl-short-date",
                        disabled: off,
                        value: "{f.short_date}",
                        onchange: move |e| {
                            let v = e.value().trim().to_string();
                            if !v.is_empty() {
                                model.write().edit_region(|r| r.short_date = v);
                            }
                        },
                    }
                    span { class: "note span2", {eg(&f.short_date)} }
                }
                div { class: "field",
                    label { {t("Long date format")} }
                    input {
                        r#type: "text",
                        list: "dl-long-date",
                        disabled: off,
                        value: "{f.long_date}",
                        onchange: move |e| {
                            let v = e.value().trim().to_string();
                            if !v.is_empty() {
                                model.write().edit_region(|r| r.long_date = v);
                            }
                        },
                    }
                    span { class: "note span2", {eg(&f.long_date)} }
                }
                div { class: "field",
                    label { {t("Short time format")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            let v = e.value();
                            model.write().edit_region(|r| r.short_time = v);
                        },
                        for p in short_times {
                            option { key: "{p}", value: "{p}", selected: f.short_time == p, "{p}" }
                        }
                    }
                    span { class: "note span2", {eg(&f.short_time)} }
                }
                div { class: "field",
                    label { {t("Long time format")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            let v = e.value();
                            model.write().edit_region(|r| r.long_time = v);
                        },
                        for p in long_times {
                            option { key: "{p}", value: "{p}", selected: f.long_time == p, "{p}" }
                        }
                    }
                    span { class: "note span2", {eg(&f.long_time)} }
                }
                h3 { class: "subhead", {t("Numbering")} }
                div { class: "field",
                    label { {t("Decimal symbol")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            let v = e.value();
                            model.write().edit_region(|r| {
                                if r.thousands == v {
                                    r.thousands = if v == "." { ",".into() } else { ".".into() };
                                }
                                r.decimal = v;
                            });
                        },
                        for s in DECIMALS {
                            option { key: "{s}", value: "{s}", selected: f.decimal == s, {symbol_name(s)} }
                        }
                    }
                }
                div { class: "field",
                    label { {t("Thousands separator")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            let v = e.value();
                            model.write().edit_region(|r| {
                                if r.decimal == v {
                                    r.decimal = if v == "." { ",".into() } else { ".".into() };
                                }
                                r.thousands = v;
                            });
                        },
                        for s in THOUSANDS {
                            option { key: "{s}", value: "{s}", selected: f.thousands == s, {symbol_name(s)} }
                        }
                    }
                }
                div { class: "field",
                    label { {t("Number")} }
                    div { class: "sample", "{gl_core::region::number(&f, 9_999_999.99, 2)}" }
                }
                h3 { class: "subhead", {t("Currency")} }
                div { class: "field",
                    label { {t("Symbol")} }
                    input {
                        r#type: "text",
                        disabled: off,
                        maxlength: "6",
                        value: "{f.currency_symbol}",
                        onchange: move |e| {
                            let v = e.value().trim().to_string();
                            if !v.is_empty() {
                                model.write().edit_region(|r| r.currency_symbol = v);
                            }
                        },
                    }
                }
                div { class: "field",
                    label { {t("ISO code")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            let v = e.value();
                            model.write().edit_region(|r| {
                                if let Some(c) = currency_info(&v) {
                                    r.currency_symbol = c.symbol.clone();
                                    r.currency_decimals = c.decimals;
                                }
                                r.currency_code = v;
                            });
                        },
                        for c in currencies() {
                            option { key: "{c.code}", value: "{c.code}", selected: f.currency_code == c.code, "{c.code} – {c.name}" }
                        }
                    }
                }
                div { class: "field",
                    label { {t("Text")} }
                    select {
                        disabled: off,
                        onchange: move |e| {
                            if let Some(p) = CurrencyPattern::ALL.into_iter().find(|p| p.label(&f.currency_symbol) == e.value()) {
                                model.write().edit_region(|r| r.currency_pattern = p);
                            }
                        },
                        for p in CurrencyPattern::ALL {
                            option {
                                key: "{p.label(&f.currency_symbol)}",
                                value: "{p.label(&f.currency_symbol)}",
                                selected: f.currency_pattern == p,
                                "{p.label(&f.currency_symbol)}"
                            }
                        }
                    }
                    span { class: "note span2",
                        {tr("Example: {sample}", &[("sample", &format!("{}   /   {}", gl_core::region::currency(&f, 1234.56), gl_core::region::currency(&f, -1234.56)))])}
                    }
                }
            }
        }
    }
}
