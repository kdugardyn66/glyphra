//! The AI Chat page: a conversation with Glyphra AI, with suggestions, a model picker and
//! a file to talk about.

use crate::app::{run, run_effect};
use crate::icons::Icon;
use crate::model::AppModel;
use crate::preview;
use dioxus::prelude::*;
use kd_ai::Speaker;
use kd_commands::{Command, SettingsSection};

/// The suggestion chips: (Fluent id of the label, Fluent id of the prompt).
const CHIPS: [(&str, &str); 10] = [
    ("chip-summarize", "chip-summarize-prompt"),
    ("chip-improve", "chip-improve-prompt"),
    ("chip-email", "chip-email-prompt"),
    ("chip-extract", "chip-extract-prompt"),
    ("chip-explain", "chip-explain-prompt"),
    ("chip-outline", "chip-outline-prompt"),
    ("chip-proofread", "chip-proofread-prompt"),
    ("chip-brainstorm", "chip-brainstorm-prompt"),
    ("chip-tone", "chip-tone-prompt"),
    ("chip-paraphrase", "chip-paraphrase-prompt"),
];

struct Msg {
    user: bool,
    html: String,
    text: String,
    error: bool,
    footer: String,
    who: String,
    empty: bool,
    streaming: bool,
}

#[component]
pub fn AiChatPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let mut input = use_signal(String::new);
    let m = model.read();
    let t = |id: &str| m.t(id);
    let (title, caption, placeholder, disclaimer) =
        (t("ai-chat-title"), t("ai-chat-caption"), t("ai-chat-placeholder"), t("ai-chat-disclaimer"));
    let (l_new, l_settings, l_attach, l_starred, l_send, l_stop, l_default, l_copy) = (
        t("ai-new-chat"),
        t("ai-open-settings"),
        t("ai-attach"),
        t("ai-use-starred"),
        t("ai-send"),
        t("ai-stop"),
        t("ai-default-model"),
        t("ai-copy"),
    );
    let (l_setup, l_setup_btn, l_thinking) = (t("ai-setup-note"), t("ai-setup-button"), t("ai-thinking"));
    let ai = &m.ai;
    let running = ai.chat.running();
    let ready = ai.ready();
    let starred = ai.chat.use_starred;
    let attachment = ai.attachment.as_ref().map(|(n, _)| n.clone());
    let status = ai.chat.status.clone();
    let chosen = ai.chat.model.as_ref().map(|x| x.to_string()).unwrap_or_default();
    let choices = ai.choices(&ai.hub.config());
    let default_label = ai.hub.model_for(kd_ai::Scope::Home).map(|x| ai.label_of(&x)).unwrap_or_default();
    let msgs: Vec<Msg> = ai
        .chat
        .messages
        .iter()
        .enumerate()
        .map(|(i, c)| Msg {
            user: c.speaker == Speaker::User,
            html: if c.speaker == Speaker::Assistant && !c.error { preview::render(&c.text) } else { String::new() },
            text: c.text.clone(),
            error: c.error,
            footer: c.footer.clone().unwrap_or_default(),
            who: c.model.as_ref().map(|x| ai.label_of(x)).unwrap_or_default(),
            empty: c.text.is_empty(),
            streaming: running && i + 1 == ai.chat.messages.len(),
        })
        .collect();
    let chips: Vec<(String, String)> = CHIPS.iter().map(|(l, p)| (t(l), t(p))).collect();
    drop(m);
    let has_msgs = !msgs.is_empty();
    let can_send = !running && !input.read().trim().is_empty();
    let mut send = move || {
        let text = input.read().trim().to_string();
        if !text.is_empty() && !model.read().ai.chat.running() {
            run(model, Command::AiSend(text));
            input.set(String::new());
            document::eval("setTimeout(() => { const l = document.getElementById('chat-log'); if (l) l.scrollTop = l.scrollHeight }, 60)");
        }
    };
    let default_option = if default_label.is_empty() { l_default.clone() } else { default_label.clone() };
    rsx! {
        div { class: "ai-page",
            div { class: "ai-head",
                h1 { "{title}" }
                div { class: "ai-head-actions",
                    button { class: "icon-btn", title: "{l_new}", onclick: move |_| run(model, Command::AiNewChat), Icon { name: "plus" } }
                    button {
                        class: "icon-btn",
                        title: "{l_settings}",
                        onclick: move |_| run(model, Command::OpenSettings(SettingsSection::AiProviders)),
                        Icon { name: "settings" }
                    }
                }
            }
            if !ready {
                div { class: "ai-note",
                    span { "{l_setup}" }
                    button { class: "btn primary", onclick: move |_| run(model, Command::OpenSettings(SettingsSection::AiProviders)), "{l_setup_btn}" }
                }
            }
            div { class: "ai-body", id: "chat-log",
                if !has_msgs {
                    div { class: "ai-hero",
                        svg { view_box: "0 0 120 120", width: "112", height: "112",
                            rect { x: "26", y: "14", width: "62", height: "82", rx: "9", style: "fill:var(--surface);stroke:var(--border-strong);stroke-width:2.5" }
                            path { d: "M38 36h38M38 48h38M38 60h24", style: "stroke:var(--border-strong);stroke-width:3;stroke-linecap:round;fill:none" }
                            path {
                                d: "M86 52 C86 68 94 76 110 76 C94 76 86 84 86 100 C86 84 78 76 62 76 C78 76 86 68 86 52 Z",
                                style: "fill:var(--accent)"
                            }
                            path { d: "M96 30 C96 36 99 39 105 39 C99 39 96 42 96 48 C96 42 93 39 87 39 C93 39 96 36 96 30 Z", style: "fill:var(--logo-spark)" }
                        }
                        div { class: "caption", "{caption}" }
                        div { class: "chips",
                            for (label , prompt) in chips {
                                button { key: "{label}", class: "chip", onclick: move |_| input.set(if prompt.trim_end().ends_with(':') { format!("{}\n\n", prompt.trim_end()) } else { prompt.clone() }), "{label}" }
                            }
                        }
                    }
                } else {
                    div { class: "ai-log",
                        for (i , msg) in msgs.into_iter().enumerate() {
                            div { key: "{i}", class: if msg.user { "msg user" } else if msg.error { "msg bot error" } else { "msg bot" },
                                if !msg.user && !msg.who.is_empty() {
                                    div { class: "who", "{msg.who}" }
                                }
                                if msg.user || msg.error {
                                    div { class: "txt selectable", "{msg.text}" }
                                } else if msg.empty {
                                    div { class: "txt dim", "{status.clone().unwrap_or_else(|| l_thinking.clone())}" }
                                } else {
                                    div { class: "txt md selectable", dangerous_inner_html: "{msg.html}" }
                                }
                                if !msg.user && !msg.empty && !msg.streaming {
                                    div { class: "foot-line",
                                        span { "{msg.footer}" }
                                        button {
                                            class: "mini",
                                            title: "{l_copy}",
                                            onclick: {
                                                let text = msg.text.clone();
                                                move |_| run_effect(model, crate::model::Effect::CopyText(text.clone()))
                                            },
                                            Icon { name: "copy", size: 14 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "ai-composer",
                if let Some(name) = attachment {
                    div { class: "attached",
                        Icon { name: "paperclip", size: 14 }
                        span { "{name}" }
                        button { class: "mini", onclick: move |_| run(model, Command::AiDetach), Icon { name: "x", size: 12 } }
                    }
                }
                textarea {
                    id: "chat-input",
                    rows: "2",
                    placeholder: "{placeholder}",
                    value: "{input}",
                    oninput: move |e| input.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter && !e.modifiers().shift() {
                            e.prevent_default();
                            send();
                        }
                    },
                }
                div { class: "bar",
                    button { class: "icon-btn", title: "{l_attach}", onclick: move |_| run(model, Command::AiAttach), Icon { name: "paperclip" } }
                    button {
                        class: if starred { "icon-btn on" } else { "icon-btn" },
                        title: "{l_starred}",
                        onclick: move |_| run(model, Command::AiToggleStarred),
                        Icon { name: "terminal" }
                    }
                    div { class: "grow" }
                    select {
                        class: "model-pick",
                        title: "{l_default}",
                        value: "{chosen}",
                        onchange: move |e| {
                            let v = e.value();
                            run(model, Command::AiSelectModel(if v.is_empty() { None } else { Some(v) }));
                        },
                        option { value: "", selected: chosen.is_empty(), "{default_option}" }
                        for c in choices {
                            option { key: "{c.value}", value: "{c.value}", selected: chosen == c.value, "{c.label}" }
                        }
                    }
                    if running {
                        button { class: "send stop", title: "{l_stop}", onclick: move |_| run(model, Command::AiStop), Icon { name: "stop" } }
                    } else {
                        button { class: "send", title: "{l_send}", disabled: !can_send, onclick: move |_| send(), Icon { name: "send" } }
                    }
                }
            }
            div { class: "ai-disclaimer", "{disclaimer}" }
        }
    }
}
