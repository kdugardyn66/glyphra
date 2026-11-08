//! The root component and the platform effects.

use crate::model::{AppModel, Effect};
use crate::{doc, home, icons::Logo, modal, settings, sidebar, tabs};
use dioxus::prelude::*;
use glyphra_i18n::Catalog;
use kd_commands::{Command, menu::command_for_menu_id};
use kd_store::{ThemePref, VERSION};
use std::path::Path;
use std::time::{Duration, Instant};

/// The splash stays at least this long, so it does not flash by.
const SPLASH_MIN: Duration = Duration::from_millis(450);
/// How long the splash takes to fade out.
const SPLASH_FADE: Duration = Duration::from_millis(280);

static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Call first thing in `main`: start-up timings are measured from here.
pub fn mark_process_start() {
    let _ = START.set(Instant::now());
}

/// With `GLYPHRA_TRACE=1` the start-up milestones go to stderr (the start screen must show
/// within 1.5 s of launch).
fn trace(what: &str) {
    if std::env::var_os("GLYPHRA_TRACE").is_some() {
        let ms = START.get().map(|s| s.elapsed().as_millis()).unwrap_or(0);
        eprintln!("[glyphra] {ms:>5} ms  {what}");
    }
}

static LANGUAGE_HOOK: std::sync::OnceLock<fn(&glyphra_i18n::I18n)> = std::sync::OnceLock::new();

/// Register a function the window calls whenever the language in use changes (including once
/// after start-up). The app uses it to relabel the native menu bar.
pub fn set_language_hook(hook: fn(&glyphra_i18n::I18n)) {
    let _ = LANGUAGE_HOOK.set(hook);
}

/// The system language as a BCP 47 tag ("en-US"), or empty.
fn system_language() -> String {
    sys_locale::get_locale().unwrap_or_default()
}

/// Run a command and carry out what only the platform can do.
pub fn run(model: Signal<AppModel>, cmd: Command) {
    let mut m = model;
    let effects = m.write().dispatch(cmd);
    for e in effects {
        exec(model, e);
    }
}

/// Carry out one effect (used by dialogs that produce effects themselves).
pub fn run_effect(model: Signal<AppModel>, effect: Effect) {
    exec(model, effect);
}

fn exec(mut model: Signal<AppModel>, effect: Effect) {
    match effect {
        Effect::Quit => dioxus::desktop::window().close(),
        Effect::OpenFileDialog => {
            spawn(async move {
                let mut dialog = rfd::AsyncFileDialog::new();
                for (label, exts) in kd_store::open_dialog_types() {
                    dialog = dialog.add_filter(label, &exts);
                }
                if let Some(files) = dialog.pick_files().await {
                    for f in files {
                        run(model, Command::OpenPath(f.path().to_path_buf()));
                    }
                }
            });
        }
        Effect::PickFolderDialog => {
            spawn(async move {
                if let Some(dir) = rfd::AsyncFileDialog::new().pick_folder().await {
                    model.write().folder_picked(dir.path().to_path_buf());
                }
            });
        }
        Effect::Reveal(p) => reveal(&p),
        Effect::OpenExternal(p) => {
            if let Err(e) = open::that(&p) {
                model.write().notify_error("error-open", &e.to_string());
            }
        }
        Effect::OpenUrl(u) => {
            // only web addresses: a menu or a document must not start other programs
            if u.starts_with("https://") || u.starts_with("http://") {
                let _ = open::that(u);
            }
        }
        Effect::CopyText(t) => {
            let js = format!("navigator.clipboard.writeText({})", serde_json::to_string(&t).unwrap_or_else(|_| "\"\"".into()));
            document::eval(&js);
            model.write().notify("toast-copied");
        }
        Effect::Trash(p) => {
            let result = trash::delete(&p).map_err(|e| e.to_string());
            model.write().trashed(&p, result);
        }
        Effect::LaunchCraft { id, file } => {
            let found = model.read().installed.iter().find(|a| a.app.id == id).cloned();
            match found {
                Some(app) => {
                    if let Err(e) = app.launch(file.as_deref()) {
                        model.write().notify_error("error-launch", &e.to_string());
                    }
                }
                None => model.write().notify("toast-craft-missing"),
            }
        }
        Effect::FocusSearch => {
            document::eval("setTimeout(() => document.getElementById('search')?.focus(), 30)");
        }
    }
}

fn reveal(p: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(p).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(format!("/select,{}", p.display())).spawn();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = open::that(p.parent().unwrap_or(p));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Loading,
    Leaving,
    Done,
}

/// The whole app: the splash while settings and the library load, then the window.
#[component]
pub fn Root() -> Element {
    // A cheap placeholder (built-in languages only) so the splash can already speak.
    let mut model = use_context_provider(|| Signal::new(AppModel::placeholder(Catalog::load(None).0, system_language())));
    let mut phase = use_signal(|| Phase::Loading);

    use_future(move || async move {
        trace("splash is on screen");
        let started = Instant::now();
        let loaded = tokio::task::spawn_blocking(|| AppModel::boot(system_language())).await;
        let left = SPLASH_MIN.saturating_sub(started.elapsed());
        if !left.is_zero() {
            tokio::time::sleep(left).await;
        }
        if let Ok(real) = loaded {
            model.set(real);
        }
        phase.set(Phase::Leaving);
        trace("start screen shown (splash fading)");
        tokio::time::sleep(SPLASH_FADE).await;
        phase.set(Phase::Done);
    });

    // Tell the native menu bar when the language changes.
    let tag = use_memo(move || model.read().i18n.info().tag.clone());
    use_effect(move || {
        let _ = tag();
        if let Some(hook) = LANGUAGE_HOOK.get() {
            hook(&model.peek().i18n);
        }
    });

    // Native menu items arrive here as command ids.
    dioxus::desktop::use_muda_event_handler(move |event| {
        if let Some(cmd) = command_for_menu_id(&event.id.0) {
            run(model, cmd);
        }
    });

    let (theme, dir, font) = {
        let m = model.read();
        let theme = match m.state.settings.theme {
            ThemePref::System => "system",
            ThemePref::Light => "light",
            ThemePref::Dark => "dark",
        };
        let info = m.i18n.info();
        (theme, info.direction.html(), info.font.clone().unwrap_or_default())
    };
    let style = if font.is_empty() { String::new() } else { format!("font-family:{font}") };
    let phase_now = phase();

    rsx! {
        style { dangerous_inner_html: crate::CSS }
        div { class: "app", "data-theme": "{theme}", dir: "{dir}", style: "{style}",
            if phase_now != Phase::Loading {
                Window {}
            }
            if phase_now != Phase::Done {
                Splash { leaving: phase_now == Phase::Leaving }
            }
        }
    }
}

#[component]
fn Splash(leaving: bool) -> Element {
    let model = use_context::<Signal<AppModel>>();
    let tagline = model.read().t("splash-tagline");
    rsx! {
        div { class: if leaving { "splash leaving" } else { "splash" },
            Logo { class: "logo" }
            div { class: "name", "Glyphra" }
            div { class: "tag", "{tagline}" }
            if !leaving {
                div { class: "bar" }
            }
            div { class: "ver", "{VERSION}" }
        }
    }
}

/// The window: tab strip, then the sidebar and the active tab.
#[component]
fn Window() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let (active, toast) = {
        let m = model.read();
        (m.active, m.toast.clone())
    };
    rsx! {
        tabs::TabStrip {}
        div { class: "body",
            if active == 0 {
                sidebar::Sidebar {}
                home::HomeTab {}
            } else {
                doc::DocTab {}
            }
        }
        modal::Dialogs {}
        settings::SettingsDialog {}
        if let Some(t) = toast {
            modal::ToastView { id: t.id, text: t.text, error: t.error }
        }
    }
}
