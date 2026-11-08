//! Questions on top of the window (rename, new folder, delete) and the toast.

use crate::model::{AppModel, ConfirmKind, Modal, PromptKind};
use dioxus::prelude::*;
use std::time::Duration;

#[component]
pub fn Dialogs() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let Some(modal) = m.modal.clone() else { return rsx! {} };
    let (cancel, ok, delete) = (m.t("modal-cancel"), m.t("modal-ok"), m.t("modal-delete"));
    match modal {
        Modal::Prompt { kind, value, error, .. } => {
            let title = m.t(match kind {
                PromptKind::RenameFile => "modal-rename-file-title",
                PromptKind::RenameFolder => "modal-rename-folder-title",
                PromptKind::NewFolder => "modal-new-folder-title",
            });
            let label = m.t("modal-name-label");
            drop(m);
            rsx! {
                div { class: "scrim", onclick: move |_| model.write().cancel_modal(),
                    div {
                        class: "dialog",
                        onclick: move |e| e.stop_propagation(),
                        onkeydown: move |e| match e.key() {
                            Key::Escape => model.write().cancel_modal(),
                            Key::Enter => {
                                let effects = model.write().submit_modal();
                                debug_assert!(effects.is_empty());
                            }
                            _ => {}
                        },
                        h3 { "{title}" }
                        input {
                            r#type: "text",
                            "aria-label": "{label}",
                            value: "{value}",
                            onmounted: move |e| {
                                spawn(async move {
                                    let _ = e.set_focus(true).await;
                                });
                            },
                            oninput: move |e| model.write().edit_prompt(e.value()),
                        }
                        if let Some(err) = error {
                            div { class: "err", "{err}" }
                        }
                        div { class: "row",
                            button { class: "btn", onclick: move |_| model.write().cancel_modal(), "{cancel}" }
                            button {
                                class: "btn primary",
                                onclick: move |_| {
                                    model.write().submit_modal();
                                },
                                "{ok}"
                            }
                        }
                    }
                }
            }
        }
        Modal::Confirm { kind, path } => {
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let (title, text) = match kind {
                ConfirmKind::DeleteFile => ("modal-delete-file-title", "modal-delete-file-text"),
                ConfirmKind::DeleteFolder => ("modal-delete-folder-title", "modal-delete-folder-text"),
            };
            let (title, text) = (m.t(title), m.t_args(text, &[("name", name.into())]));
            drop(m);
            rsx! {
                div { class: "scrim", onclick: move |_| model.write().cancel_modal(),
                    div {
                        class: "dialog",
                        onclick: move |e| e.stop_propagation(),
                        onkeydown: move |e| {
                            if e.key() == Key::Escape {
                                model.write().cancel_modal();
                            }
                        },
                        h3 { "{title}" }
                        p { "{text}" }
                        div { class: "row",
                            button { class: "btn", onclick: move |_| model.write().cancel_modal(), "{cancel}" }
                            button {
                                class: "btn danger",
                                onclick: move |_| {
                                    let effects = model.write().submit_modal();
                                    for e in effects {
                                        if let crate::Effect::Trash(_) = e {
                                            crate::app::run_effect(model, e);
                                        }
                                    }
                                },
                                "{delete}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A message that disappears after a few seconds. The `id` changes with every message, so a
/// new message restarts the timer.
#[component]
pub fn ToastView(id: u64, text: String, error: bool) -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    use_future(move || async move {
        tokio::time::sleep(Duration::from_secs(if error { 7 } else { 3 })).await;
        model.write().dismiss_toast(id);
    });
    rsx! {
        div { class: if error { "toast error" } else { "toast" }, role: "status", "{text}" }
    }
}
