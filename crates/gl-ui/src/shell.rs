//! The root component: title bar with tabs, the left menu bar, the page in front, the AI panel
//! on the right and the notice bar.

use crate::icons::{Icon, Logo};
use crate::{ai_panel, ai_tools, collaboration, doc, home, open, settings, templates};
use dioxus::prelude::*;
use gl_core::{AppModel, Command, Effect, NoticeKind, Page, SettingsSection, command_for_menu_id, t};
use gl_store::Store;
use std::time::Duration;

/// Run a command and carry out what only the platform can do.
pub fn run(mut model: Signal<AppModel>, cmd: Command) {
    let effects = model.write().dispatch(cmd);
    for e in effects {
        exec(model, e);
    }
}

fn exec(mut model: Signal<AppModel>, effect: Effect) {
    match effect {
        Effect::Quit => dioxus::desktop::window().close(),
        Effect::FetchModels { provider, request } => {
            // not tied to the component that asked: forms are rebuilt while the answer is on its way
            dioxus::core::spawn_forever(async move {
                let result = tokio::task::spawn_blocking(move || fetch_text(&request)).await.unwrap_or_else(|e| Err(e.to_string()));
                run(model, Command::ModelsFetched { provider, result });
            });
        }
        Effect::Reveal(path) => {
            if let Err(e) = reveal(&path) {
                model.write().notify(
                    gl_core::NoticeKind::Error,
                    gl_core::tr("Could not open the file manager: {error}", &[("error", &e.to_string())]),
                );
            }
        }
        Effect::OpenFileDialog { start } => {
            spawn(async move {
                let dialog = rfd::AsyncFileDialog::new().set_directory(&start).add_filter(t("Markdown and text"), &gl_core::OPENABLE);
                if let Some(files) = dialog.pick_files().await {
                    for f in files {
                        run(model, Command::OpenPath(f.path().to_path_buf()));
                    }
                }
            });
        }
    }
}

/// Show a file in the system's file manager (selected where the system can do that).
fn reveal(path: &std::path::Path) -> std::io::Result<()> {
    use std::process::Command;
    let mut cmd = if cfg!(target_os = "macos") {
        let mut c = Command::new("open");
        if path.is_dir() {
            c.arg(path)
        } else {
            c.arg("-R").arg(path)
        };
        c
    } else if cfg!(target_os = "windows") {
        let mut c = Command::new("explorer");
        if path.is_dir() {
            c.arg(path)
        } else {
            c.arg(format!("/select,{}", path.display()))
        };
        c
    } else {
        // no standard way to select a file: open its folder
        let mut c = Command::new("xdg-open");
        c.arg(if path.is_dir() { path } else { path.parent().unwrap_or(path) });
        c
    };
    cmd.spawn().map(|_| ())
}

/// The most that is read from a provider's answer.
const MAX_ANSWER: u64 = 2 * 1024 * 1024;

/// Ask a provider for its models: one GET, never redirected (a redirect could carry the API key
/// to another server), at most `MAX_ANSWER` bytes read.
fn fetch_text(request: &gl_core::models_api::Request) -> Result<String, String> {
    use std::io::Read;
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(20)).redirects(0).build();
    let mut call = agent.get(&request.url);
    for (name, value) in &request.headers {
        call = call.set(name, value);
    }
    // the address (which may hold the key as a query) is not part of the error
    let resp = call.call().map_err(|e| match e {
        ureq::Error::Status(code, _) => format!("status code {code}"),
        ureq::Error::Transport(t) => t.kind().to_string(),
    })?;
    let mut text = String::new();
    resp.into_reader().take(MAX_ANSWER + 1).read_to_string(&mut text).map_err(|e| e.to_string())?;
    if text.len() as u64 > MAX_ANSWER {
        return Err("The answer is too large.".into());
    }
    Ok(text)
}

#[component]
pub fn Root() -> Element {
    let model = use_context_provider(|| Signal::new(AppModel::new(Store::platform())));

    // Native menu items arrive here as ids.
    dioxus::desktop::use_muda_event_handler(move |event| {
        if let Some(cmd) = command_for_menu_id(&event.id.0) {
            run(model, cmd);
        }
    });

    let m = model.read();
    let (theme, light_theme, dark_theme) = m.settings.theme_attrs();
    let class = if cfg!(target_os = "macos") { "app mac" } else { "app" };
    let panel = m.ai_panel_visible();
    let in_doc = m.active_tab.is_some();
    let page = m.page;
    drop(m);

    rsx! {
        style { dangerous_inner_html: crate::CSS }
        div { class: "{class}", "data-theme": "{theme}", "data-light": "{light_theme}", "data-dark": "{dark_theme}",
            TitleBar {}
            div { class: "body",
                Rail {}
                main { class: "content",
                    if in_doc {
                        doc::DocView {}
                    } else {
                        match page {
                            Page::Home => rsx! { home::HomePage {} },
                            Page::Open => rsx! { open::OpenPage {} },
                            Page::Templates => rsx! { templates::TemplatesPage {} },
                            Page::Collaboration => rsx! { collaboration::CollaborationPage {} },
                            Page::AiTools => rsx! { ai_tools::AiToolsPage {} },
                            Page::Settings => rsx! { settings::SettingsPage {} },
                        }
                    }
                }
                if panel {
                    ai_panel::AiPanel {}
                }
            }
            NoticeBar {}
        }
    }
}

/// The strip above the page: the Home tab, then one tab per open document. On macOS the
/// window has no native title bar, so the traffic lights sit at the left of this strip and an
/// empty part of it moves the window.
#[component]
fn TitleBar() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let tabs: Vec<(usize, String)> = m.tabs.iter().enumerate().map(|(i, t)| (i, t.title.clone())).collect();
    let active = m.active_tab;
    drop(m);
    rsx! {
        div {
            class: "titlebar",
            onmousedown: move |_| {
                if cfg!(target_os = "macos") {
                    dioxus::desktop::window().drag();
                }
            },
            div { class: "traffic-space" }
            button {
                class: if active.is_none() { "tab home active" } else { "tab home" },
                onmousedown: move |e| e.stop_propagation(),
                onclick: move |_| run(model, Command::ShowPage(Page::Home)),
                Logo { class: "tab-logo" }
                span { "GLYPHRA" }
            }
            for (i , title) in tabs {
                div { key: "{i}", class: if active == Some(i) { "tab doc active" } else { "tab doc" }, onmousedown: move |e| e.stop_propagation(),
                    button { class: "tab-title", title: "{title}", onclick: move |_| run(model, Command::ActivateTab(i)), "{title}" }
                    button { class: "tab-close", title: t("Close"), onclick: move |_| run(model, Command::CloseTab(i)), Icon { name: "x", size: 12 } }
                }
            }
        }
    }
}

/// The left menu bar: AI Chat, then Home, Open, Templates, Collaboration, AI Tools, and
/// Settings at the bottom.
#[component]
fn Rail() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let page = if m.active_tab.is_some() { None } else { Some(m.page) };
    let panel = m.ai_panel_visible();
    let ai_on = m.settings.ai_enabled;
    drop(m);
    let on = move |p: Page| {
        if page == Some(p) { "rail-btn active" } else { "rail-btn" }
    };
    rsx! {
        nav { class: "rail",
            button {
                class: if panel { "rail-btn active" } else if ai_on { "rail-btn" } else { "rail-btn off" },
                title: t("Show or hide the AI panel"),
                onclick: move |_| {
                    if ai_on {
                        run(model, Command::ToggleAiPanel);
                    } else {
                        let mut m = model;
                        m.write().notify(NoticeKind::Info, t("AI features are off. Turn them on in Settings › General (applies after a restart)."));
                    }
                },
                Icon { name: "chat", size: 26 }
                span { {t("AI Chat")} }
            }
            div { class: "rail-sep" }
            button { class: on(Page::Home), onclick: move |_| run(model, Command::ShowPage(Page::Home)),
                Icon { name: "home", size: 26 }
                span { {t("Home")} }
            }
            button { class: on(Page::Open), onclick: move |_| run(model, Command::ShowPage(Page::Open)),
                Icon { name: "folder-open", size: 26 }
                span { {t("Open")} }
            }
            button { class: on(Page::Templates), onclick: move |_| run(model, Command::ShowPage(Page::Templates)),
                Icon { name: "templates", size: 26 }
                span { {t("Templates")} }
            }
            button { class: on(Page::Collaboration), onclick: move |_| run(model, Command::ShowPage(Page::Collaboration)),
                Icon { name: "collaboration", size: 26 }
                span { {t("Collaboration")} }
            }
            button { class: on(Page::AiTools), onclick: move |_| run(model, Command::ShowPage(Page::AiTools)),
                Icon { name: "sparkles", size: 26 }
                span { {t("AI Tools")} }
            }
            div { class: "rail-grow" }
            button { class: on(Page::Settings), onclick: move |_| run(model, Command::OpenSettings(SettingsSection::General)),
                Icon { name: "settings", size: 26 }
                span { {t("Settings")} }
            }
        }
    }
}

/// Errors and information, at the top of the window. Gone after a few seconds, or on click.
#[component]
fn NoticeBar() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    // a memo, so the timer starts only when the notice itself changes
    let notice_memo = use_memo(move || model.read().notice.clone());
    let notice = notice_memo();
    use_effect(move || {
        if let Some(n) = notice_memo() {
            spawn(async move {
                tokio::time::sleep(Duration::from_secs(8)).await;
                // only dismiss the notice that was on screen when the timer started
                if model.peek().notice.as_ref() == Some(&n) {
                    model.write().dismiss_notice();
                }
            });
        }
    });
    let Some(n) = notice else { return rsx! {} };
    let class = if n.kind == NoticeKind::Error { "notice error" } else { "notice" };
    rsx! {
        div { class: "{class}", role: "alert",
            span { class: "msg selectable", "{n.text}" }
            button { class: "icon-btn", title: t("Dismiss"), onclick: move |_| model.write().dismiss_notice(), Icon { name: "x", size: 14 } }
        }
    }
}
