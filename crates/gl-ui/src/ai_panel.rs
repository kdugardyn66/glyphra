//! The AI Chat panel on the right: the conversation with the model you connected, the prompts
//! above the message box, history, files you attach, and the tools of the AI Tools catalogue.

use crate::ai_tools::SECTIONS;
use crate::icons::Icon;
use crate::pick::{Pick, PickItem};
use crate::shell::run;
use dioxus::prelude::*;
use gl_core::chat::{self, Role};
use gl_core::{AppModel, Command, NoticeKind, SettingsSection, Step, t, tr};

/// The last entry of the model list: connect another model or provider.
const ADD_PROVIDER: &str = "::add";

/// Which pop-up menu is open.
#[derive(Clone, Copy, PartialEq)]
enum Menu {
    History,
    Context,
    Tools,
}

fn unix_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Put text on the clipboard.
fn copy(text: String) {
    let ev = document::eval("navigator.clipboard.writeText(await dioxus.recv());");
    let _ = ev.send(text);
}

#[component]
pub fn AiPanel() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let mut menu = use_signal(|| None::<Menu>);
    let mut draft = use_signal(String::new);
    let (choices, has_model, chat, busy, error, expanded, use_doc, doc_name, files, chats) = {
        let m = model.read();
        (
            m.model_choices(),
            m.settings.model_for("chat").is_some(),
            m.chat_current.as_ref().and_then(|id| m.chats.iter().find(|c| c.id == *id)).cloned(),
            m.chat_busy.is_some(),
            m.chat_error.clone(),
            m.chat_expanded,
            m.chat_use_document,
            m.active_tab.and_then(|i| m.tabs.get(i)).map(|t| t.title.clone()),
            m.chat_files.clone(),
            m.chats.clone(),
        )
    };
    let current = choices.iter().find(|c| c.current).map(|c| format!("{}::{}", c.provider, c.model)).unwrap_or_default();
    let messages = chat.as_ref().map(|c| c.messages.clone()).unwrap_or_default();
    let in_chat = !messages.is_empty();
    let mut send = move || {
        let text = draft();
        if !text.trim().is_empty() && !model.read().chat_busy.is_some() {
            run(model, Command::SendChat(text));
            draft.set(String::new());
        }
    };
    let can_send = !busy && !draft().trim().is_empty();
    let now = unix_now();
    rsx! {
        aside { class: if expanded { "ai-panel wide" } else { "ai-panel" },
            div { class: "ai-head",
                h2 { {t("AI Chat")} }
                div { class: "ai-tools-row",
                    if in_chat {
                        button { class: "icon-btn", title: t("New chat"), onclick: move |_| run(model, Command::NewChat), Icon { name: "plus", size: 20 } }
                    }
                    button {
                        class: "icon-btn",
                        title: t("History"),
                        onclick: move |_| menu.set(if menu() == Some(Menu::History) { None } else { Some(Menu::History) }),
                        Icon { name: "history", size: 20 }
                    }
                    button { class: "icon-btn", title: t("AI settings"), onclick: move |_| run(model, Command::OpenSettings(SettingsSection::AiModels)), Icon { name: "sliders", size: 20 } }
                    button { class: "icon-btn", title: if expanded { t("Make the chat narrower") } else { t("Expand") }, onclick: move |_| run(model, Command::ToggleChatExpanded), Icon { name: "expand", size: 20 } }
                    button { class: "icon-btn", title: t("Close"), onclick: move |_| run(model, Command::ToggleAiPanel), Icon { name: "x", size: 20 } }
                }
            }
            if menu() == Some(Menu::History) {
                div { class: "menu-backdrop", onclick: move |_| menu.set(None) }
                div { class: "chat-menu history",
                    if chats.is_empty() {
                        p { class: "note", {t("No chats yet.")} }
                    }
                    for c in chats.iter() {
                        {
                            let (id_open, id_del) = (c.id.clone(), c.id.clone());
                            let active = chat.as_ref().is_some_and(|x| x.id == c.id);
                            rsx! {
                                div { key: "{c.id}", class: if active { "menu-row on" } else { "menu-row" },
                                    button {
                                        class: "menu-main",
                                        onclick: move |_| {
                                            run(model, Command::OpenChat(id_open.clone()));
                                            menu.set(None);
                                        },
                                        span { class: "menu-title", "{c.title}" }
                                        span { class: "menu-hint", {gl_core::i18n::time_ago(c.updated, now)} }
                                    }
                                    button { class: "icon-btn", title: t("Delete chat"), onclick: move |_| run(model, Command::DeleteChat(id_del.clone())), Icon { name: "trash", size: 16 } }
                                }
                            }
                        }
                    }
                    if !chats.is_empty() {
                        div { class: "menu-sep" }
                        button {
                            class: "menu-item",
                            onclick: move |_| {
                                run(model, Command::ClearChats);
                                menu.set(None);
                            },
                            {t("Delete all chats")}
                        }
                    }
                }
            }
            if in_chat {
                div { class: "thread selectable",
                    for (i , m) in messages.iter().enumerate() {
                        {
                            let text = m.text.clone();
                            let shown = m.shown.clone().unwrap_or_else(|| m.text.clone());
                            let is_user = m.role == Role::User;
                            rsx! {
                                div { key: "{i}", class: if is_user { "msg user" } else { "msg bot" },
                                    div { class: "msg-text", "{shown}" }
                                    if !is_user {
                                        button { class: "icon-btn msg-copy", title: t("Copy"), onclick: move |_| copy(text.clone()), Icon { name: "copy", size: 15 } }
                                    }
                                }
                            }
                        }
                    }
                    if busy {
                        div { class: "msg bot thinking", onmounted: move |e| async move { let _ = e.scroll_to(ScrollBehavior::Smooth).await; }, {t("Thinking…")} }
                    }
                    if let Some(e) = error {
                        div { class: "msg error", onmounted: move |e| async move { let _ = e.scroll_to(ScrollBehavior::Smooth).await; },
                            span { "{e}" }
                            button { class: "link", onclick: move |_| run(model, Command::RetryChat), {t("Try again")} }
                        }
                    }
                }
            } else {
                div { class: "ai-hero",
                    svg { class: "hero", view_box: "0 0 320 190",
                        rect { x: "34", y: "30", width: "86", height: "112", rx: "8", class: "pg back" }
                        rect { x: "200", y: "30", width: "86", height: "112", rx: "8", class: "pg back" }
                        rect { x: "98", y: "8", width: "124", height: "148", rx: "8", class: "pg front" }
                        for i in 0..7 {
                            rect { key: "{i}", x: "114", y: "{26 + i * 15}", width: "92", height: "7", rx: "2", class: "ln" }
                        }
                        path { d: "M128 112 Q142 132 158 136 Q142 142 134 160 Q126 142 108 136 Q124 132 128 112Z", class: "spark" }
                    }
                    p { {t("Your AI Chat for documents")} }
                }
                div { class: "chips",
                    for (i , chip) in chat::CHIPS.iter().enumerate() {
                        button { key: "{chip.label}", class: "chip", disabled: busy, onclick: move |_| run(model, Command::ChatChip(i)), {t(chip.label)} }
                    }
                }
            }
            div { class: "composer",
                if menu() == Some(Menu::Context) {
                    div { class: "menu-backdrop", onclick: move |_| menu.set(None) }
                    div { class: "chat-menu up",
                        button {
                            class: "menu-item",
                            disabled: doc_name.is_none(),
                            onclick: move |_| run(model, Command::ToggleChatDocument),
                            span { class: "check", if use_doc && doc_name.is_some() { "✓" } }
                            {t("Include the current document")}
                        }
                        button {
                            class: "menu-item",
                            onclick: move |_| {
                                run(model, Command::ChooseChatFile);
                                menu.set(None);
                            },
                            span { class: "check" }
                            {t("Attach a file…")}
                        }
                    }
                }
                if menu() == Some(Menu::Tools) {
                    div { class: "menu-backdrop", onclick: move |_| menu.set(None) }
                    div { class: "chat-menu up tools",
                        for sec in SECTIONS {
                            div { key: "{sec.title}", class: "menu-group", {t(sec.title)} }
                            for (title , desc) in sec.tools.iter().copied().filter(|(title, _)| chat::tool(title).is_some()) {
                                button {
                                    key: "{title}",
                                    class: "menu-item",
                                    title: t(desc),
                                    disabled: busy,
                                    onclick: move |_| {
                                        run(model, Command::RunChatTool(title.to_string()));
                                        menu.set(None);
                                    },
                                    {t(title)}
                                }
                            }
                        }
                    }
                }
                if doc_name.is_some() && use_doc || !files.is_empty() {
                    div { class: "attached",
                        if let (Some(name), true) = (doc_name.clone(), use_doc) {
                            span { class: "pill", title: t("The document you are looking at is sent with your messages"),
                                span { class: "pill-name", "{name}" }
                                button { class: "icon-btn", title: t("Do not send the current document"), onclick: move |_| run(model, Command::ToggleChatDocument), Icon { name: "x", size: 12 } }
                            }
                        }
                        for f in files {
                            {
                                let path = f.clone();
                                let name = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                rsx! {
                                    span { key: "{f.display()}", class: "pill", title: "{f.display()}",
                                        span { class: "pill-name", "{name}" }
                                        button { class: "icon-btn", title: t("Remove attachment"), onclick: move |_| run(model, Command::DetachFromChat(path.clone())), Icon { name: "x", size: 12 } }
                                    }
                                }
                            }
                        }
                    }
                }
                textarea {
                    placeholder: t("Ask anything…"),
                    rows: "2",
                    value: "{draft}",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter && !e.modifiers().shift() {
                            e.prevent_default();
                            send();
                        }
                    },
                }
                div { class: "model-row",
                    if has_model {
                        button { class: "icon-btn", title: t("Cheaper model (same provider)"), onclick: move |_| run(model, Command::StepModel(Step::Cheaper)), Icon { name: "chev-down", size: 16 } }
                        Pick {
                            class: "",
                            up: true,
                            title: t("Model for AI Chat. Switching keeps your provider and API key."),
                            placeholder: t("Choose model"),
                            value: current.clone(),
                            items: choices
                                .iter()
                                .map(|c| PickItem { id: format!("{}::{}", c.provider, c.model), name: c.name.clone(), hint: c.price.clone(), group: c.provider_name.clone() })
                                .chain(std::iter::once(PickItem { id: ADD_PROVIDER.to_string(), name: t("Add another provider…"), hint: String::new(), group: t("More") }))
                                .collect::<Vec<_>>(),
                            onopen: move |_| run(model, Command::FetchModels(None)),
                            onpick: move |v: String| {
                                if v == ADD_PROVIDER {
                                    run(model, Command::OpenSettings(SettingsSection::AiModels));
                                    model.write().begin_add_model();
                                } else if let Some((p, m)) = v.split_once("::") {
                                    run(model, Command::UseModel { provider: p.to_string(), model: m.to_string() });
                                }
                            },
                        }
                        button { class: "icon-btn up", title: t("More capable, more expensive model (same provider)"), onclick: move |_| run(model, Command::StepModel(Step::Pricier)), Icon { name: "chev-down", size: 16 } }
                    } else {
                        button { class: "model-pick", title: t("Connect an AI model"), onclick: move |_| run(model, Command::OpenSettings(SettingsSection::AiModels)),
                            span { {t("No model connected")} }
                        }
                    }
                }
                div { class: "composer-row",
                    button {
                        class: "icon-btn",
                        title: t("Add context"),
                        onclick: move |_| menu.set(if menu() == Some(Menu::Context) { None } else { Some(Menu::Context) }),
                        Icon { name: "plus", size: 18 }
                    }
                    button {
                        class: "icon-btn",
                        title: t("Tools"),
                        onclick: move |_| menu.set(if menu() == Some(Menu::Tools) { None } else { Some(Menu::Tools) }),
                        Icon { name: "terminal", size: 18 }
                    }
                    span { class: "grow" }
                    button {
                        class: "send",
                        title: t("Send"),
                        disabled: !can_send,
                        onclick: move |_| {
                            if !has_model {
                                model.write().notify(NoticeKind::Info, tr("Connect an AI model first (Settings › AI Models).", &[]));
                                run(model, Command::OpenSettings(SettingsSection::AiModels));
                            } else {
                                send();
                            }
                        },
                        Icon { name: "send", size: 18 }
                    }
                }
            }
            div { class: "ai-foot", {t("AI Chat can make mistakes. Check important info.")} }
        }
    }
}
