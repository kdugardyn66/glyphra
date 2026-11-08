//! Settings › AI providers and AI models: the provider list by region, adding a provider,
//! editing one (with Test connection), and the choice of model per part of the app.

use crate::ai::{AiScreen, TestState, kind_badge, kind_title, status};
use crate::model::AppModel;
use dioxus::prelude::*;
use kd_ai::{KeySource, Protocol, ProviderConfig, ProviderKind, Region, Scope, Service};

fn region_key(r: Region) -> &'static str {
    match r {
        Region::Eu => "ai-region-eu",
        Region::Apac => "ai-region-apac",
        Region::Us => "ai-region-us",
        Region::Local => "ai-region-local",
    }
}

/// Change the provider the open form edits.
fn edit(mut model: Signal<AppModel>, f: impl FnOnce(&mut ProviderConfig)) {
    if let Some(p) = model.write().ai.target() {
        f(p);
    }
}

fn opt(v: String) -> Option<String> {
    Some(v).filter(|x| !x.trim().is_empty())
}

#[component]
fn KindSquare(kind: ProviderKind, size: u32) -> Element {
    let (colour, initials) = kind_badge(kind);
    let font = size * 2 / 5;
    rsx! {
        span {
            class: "kind-sq",
            style: "width:{size}px;height:{size}px;background:{colour};font-size:{font}px;border-radius:{size / 4}px",
            "{initials}"
        }
    }
}

#[component]
fn StatusChip(provider: ProviderConfig) -> Element {
    let model = use_context::<Signal<AppModel>>();
    let (id, var, tone) = status(&provider);
    let text = match var {
        Some(v) => model.read().t_args(id, &[("var", v.into())]),
        None => model.read().t(id),
    };
    let class = match tone {
        1 => "chip-s good",
        2 => "chip-s bad",
        _ => "chip-s",
    };
    rsx! { span { class: "{class}", "{text}" } }
}

#[component]
pub fn ProvidersPane() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let screen = model.read().ai.screen.clone();
    match screen {
        AiScreen::List => rsx! { ProviderList {} },
        AiScreen::AddKind => rsx! { AddKind {} },
        AiScreen::AddForm | AiScreen::Edit(_) => rsx! { ProviderForm {} },
    }
}

#[component]
fn Back(label: String, to: AiScreen) -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    rsx! { button { class: "back-link", onclick: move |_| model.write().ai.show(to.clone()), "‹  {label}" } }
}

#[component]
fn ProviderList() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let (sub, add, none) = (m.t("ai-providers-sub"), m.t("ai-add-provider"), m.t("ai-no-providers"));
    let groups: Vec<(String, Vec<ProviderConfig>)> = Region::ALL
        .iter()
        .map(|r| (m.t(region_key(*r)), m.ai.draft.providers.iter().filter(|p| p.region() == *r).cloned().collect::<Vec<_>>()))
        .filter(|(_, v)| !v.is_empty())
        .collect();
    let empty = m.ai.draft.providers.is_empty();
    drop(m);
    rsx! {
        div { class: "ai-set",
            div { class: "ai-set-head",
                p { class: "lead", "{sub}" }
                button { class: "btn primary", onclick: move |_| model.write().ai.show(AiScreen::AddKind), "{add}" }
            }
            if empty {
                p { class: "lead", "{none}" }
            }
            for (label , rows) in groups {
                div { key: "{label}", class: "region-label", "{label}" }
                for p in rows {
                    {
                        let id = p.id.clone();
                        let sub = match p.kind {
                            ProviderKind::Custom | ProviderKind::OpenAiCompatible => p.base_url(),
                            k => k.blurb().to_string(),
                        };
                        rsx! {
                            button { key: "{p.id}", class: "prov-row", onclick: move |_| model.write().ai.show(AiScreen::Edit(id.clone())),
                                KindSquare { kind: p.kind, size: 38 }
                                span { class: "pr-text",
                                    span { class: "pr-title", "{p.display_name()}" }
                                    span { class: "pr-sub", "{sub}" }
                                }
                                StatusChip { provider: p.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AddKind() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let (back, head, sub) = (m.t("ai-providers"), m.t("ai-add-title"), m.t("ai-add-sub"));
    let groups: Vec<(String, Vec<ProviderKind>)> =
        Region::ALL.iter().map(|r| (m.t(region_key(*r)), ProviderKind::in_region(*r))).filter(|(_, v)| !v.is_empty()).collect();
    drop(m);
    rsx! {
        div { class: "ai-set",
            Back { label: back, to: AiScreen::List }
            h3 { "{head}" }
            p { class: "lead", "{sub}" }
            for (label , kinds) in groups {
                div { key: "{label}", class: "region-label", "{label}" }
                for kind in kinds {
                    button { key: "{kind_title(kind)}", class: "prov-row", onclick: move |_| model.write().ai.choose_kind(kind),
                        KindSquare { kind, size: 32 }
                        span { class: "pr-text",
                            span { class: "pr-title", "{kind_title(kind)}" }
                            span { class: "pr-sub", "{kind.blurb()}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProviderForm() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let Some(p) = m.ai.target_ref().cloned() else { return rsx! { Back { label: m.t("ai-providers"), to: AiScreen::List } } };
    let adding = m.ai.screen == AiScreen::AddForm;
    let visible = m.ai.key_visible;
    let t = |k: &str| m.t(k);
    let models_text = p.models.join(", ");
    let offered = m.ai.offered(&p);
    let test: Option<(String, &'static str)> = m.ai.tests.get(&p.id).map(|s| match s {
        TestState::Running(_) => (t("ai-test-running"), "dim"),
        TestState::Ok(list, _) => (m.t_args("ai-test-ok", &[("n", list.len().into())]), "good"),
        TestState::Err(e) => (e.clone(), "bad"),
    });
    let account_note = match m.ai.tests.get(&p.id) {
        Some(TestState::Ok(_, note)) => note.clone(),
        _ => None,
    };
    let key_note = match p.key_source() {
        KeySource::Stored => t("ai-key-stored"),
        KeySource::Env(v) => m.t_args("ai-key-env", &[("var", v.into())]),
        KeySource::None => p.kind.key_env().first().map(|v| m.t_args("ai-key-env-hint", &[("var", (*v).into())])).unwrap_or_default(),
    };
    let heading = if adding { m.t_args("ai-add-named", &[("name", p.kind.label().into())]) } else { p.display_name() };
    let protocols: Vec<(usize, &'static str)> = Protocol::CHAT.iter().enumerate().map(|(i, x)| (i, x.label())).collect();
    let regions: Vec<(usize, String)> = Region::ALL.iter().enumerate().map(|(i, r)| (i, m.t(region_key(*r)))).collect();
    let proto_now = Protocol::CHAT.iter().position(|x| *x == p.protocol()).unwrap_or(0);
    let region_now = Region::ALL.iter().position(|r| *r == p.region()).unwrap_or(0);
    let default_model = p.default_model.clone().unwrap_or_default();
    let (
        l_back,
        l_enabled,
        l_name,
        l_url,
        l_proto,
        l_region,
        l_key,
        l_model,
        l_auto,
        l_models,
        l_models_ph,
        l_test,
        l_add,
        l_cancel,
        l_remove,
    ) = (
        t(if adding { "ai-choose-another" } else { "ai-providers" }),
        t("ai-enabled"),
        t("ai-name"),
        t("ai-endpoint"),
        t("ai-api"),
        t("ai-region"),
        t("ai-key"),
        t("ai-default-model-of-provider"),
        t("ai-model-auto"),
        t("ai-models-list"),
        t("ai-models-list-ph"),
        t("ai-test"),
        t("ai-add"),
        t("ai-cancel"),
        t("ai-remove"),
    );
    let (l_show, l_hide) = (t("ai-key-show"), t("ai-key-hide"));
    drop(m);
    let chat = p.service() == Service::Chat;
    let needs_key = p.kind.needs_key() || matches!(p.kind, ProviderKind::OpenAiCompatible | ProviderKind::Custom);
    let back_to = if adding { AiScreen::AddKind } else { AiScreen::List };
    let id = p.id.clone();
    rsx! {
        div { class: "ai-set",
            Back { label: l_back, to: back_to }
            h3 { "{heading}" }
            div { class: "form-head",
                KindSquare { kind: p.kind, size: 38 }
                span { class: "fh-text",
                    strong { "{p.kind.vendor()}" }
                    StatusChip { provider: p.clone() }
                }
            }
            p { class: "help", "{p.kind.help()}" }
            label { class: "check",
                input { r#type: "checkbox", checked: p.enabled, onchange: move |e| edit(model, |x| x.enabled = e.checked()) }
                " {l_enabled}"
            }
            if p.kind == ProviderKind::Custom {
                div { class: "frow",
                    label { "{l_name}" }
                    input {
                        r#type: "text",
                        value: "{p.name.clone().unwrap_or_default()}",
                        oninput: move |e| edit(model, |x| x.name = opt(e.value())),
                    }
                }
            }
            if p.kind.endpoint_editable() {
                div { class: "frow",
                    label { "{l_url}" }
                    input {
                        r#type: "text",
                        value: "{p.base_url.clone().unwrap_or_default()}",
                        placeholder: "{p.kind.default_base_url()}",
                        oninput: move |e| edit(model, |x| x.base_url = opt(e.value())),
                    }
                }
            }
            if p.kind == ProviderKind::Custom {
                div { class: "frow",
                    label { "{l_proto}" }
                    select { onchange: move |e| edit(model, |x| x.protocol = e.value().parse::<usize>().ok().and_then(|i| Protocol::CHAT.get(i).copied())),
                        for (i , label) in protocols {
                            option { key: "{i}", value: "{i}", selected: i == proto_now, "{label}" }
                        }
                    }
                }
                div { class: "frow",
                    label { "{l_region}" }
                    select { onchange: move |e| edit(model, |x| x.region = e.value().parse::<usize>().ok().and_then(|i| Region::ALL.get(i).copied())),
                        for (i , label) in regions {
                            option { key: "{i}", value: "{i}", selected: i == region_now, "{label}" }
                        }
                    }
                }
            }
            if needs_key {
                div { class: "frow",
                    label { "{l_key}" }
                    div { class: "keyrow",
                        input {
                            r#type: if visible { "text" } else { "password" },
                            autocomplete: "off",
                            value: "{p.api_key.clone().unwrap_or_default()}",
                            oninput: move |e| edit(model, |x| x.api_key = opt(e.value())),
                        }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                let v = model.read().ai.key_visible;
                                model.write().ai.key_visible = !v;
                            },
                            if visible {
                                "{l_hide}"
                            } else {
                                "{l_show}"
                            }
                        }
                    }
                    if !key_note.is_empty() {
                        span { class: "note", "{key_note}" }
                    }
                }
            }
            if chat {
                div { class: "frow",
                    label { "{l_model}" }
                    select { onchange: move |e| edit(model, |x| x.default_model = opt(e.value())),
                        option { value: "", selected: default_model.is_empty(), "{l_auto}" }
                        if !default_model.is_empty() && !offered.contains(&default_model) {
                            option { value: "{default_model}", selected: true, "{default_model}" }
                        }
                        for m in offered {
                            option { key: "{m}", value: "{m}", selected: m == default_model, "{m}" }
                        }
                    }
                }
                if p.kind.lists_own_models() {
                    div { class: "frow",
                        label { "{l_models}" }
                        input {
                            r#type: "text",
                            value: "{models_text}",
                            placeholder: "{l_models_ph}",
                            oninput: move |e| edit(model, |x| x.models = e.value().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()),
                        }
                    }
                }
            }
            div { class: "test-row",
                button { class: "btn", onclick: move |_| model.write().ai.test_target(), "{l_test}" }
                if let Some((text , tone)) = test {
                    span { class: "test-{tone}", "{text}" }
                }
            }
            if let Some(n) = account_note {
                span { class: "note", "{n}" }
            }
            div { class: "tool-actions",
                if adding {
                    button { class: "btn primary", onclick: move |_| model.write().ai.add_form_provider(), "{l_add}" }
                    button { class: "btn", onclick: move |_| model.write().ai.show(AiScreen::List), "{l_cancel}" }
                } else {
                    button { class: "btn danger", onclick: move |_| model.write().ai.remove_provider(&id), "{l_remove}" }
                }
            }
        }
    }
}

#[component]
pub fn ModelsPane() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let t = |id: &str| m.t(id);
    let ai = &m.ai;
    let choices = ai.choices(&ai.draft);
    let default_model = ai.draft.default_model.as_ref().map(|x| x.to_string()).unwrap_or_default();
    let scopes: Vec<(String, String, String)> = Scope::ALL
        .iter()
        .map(|s| {
            (s.id().to_string(), t(&format!("ai-scope-{}", s.id())), ai.draft.models.get(s.id()).map(|x| x.to_string()).unwrap_or_default())
        })
        .collect();
    let (n_models, l_default, l_use_default, l_none) =
        (t("ai-models-note"), t("ai-default-model"), t("ai-use-default"), t("ai-models-none"));
    let any = !choices.is_empty();
    drop(m);
    rsx! {
        div { class: "ai-set",
            p { class: "lead", "{n_models}" }
            if !any {
                p { class: "lead", "{l_none}" }
            }
            div { class: "frow",
                label { "{l_default}" }
                select { onchange: move |e| model.write().ai.set_default_model(&e.value()),
                    option { value: "", selected: default_model.is_empty(), "—" }
                    for c in choices.clone() {
                        option { key: "{c.value}", value: "{c.value}", selected: default_model == c.value, "{c.label}" }
                    }
                }
            }
            for (id , label , value) in scopes {
                div { key: "{id}", class: "frow",
                    label { "{label}" }
                    select {
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
}
