//! The start screen's sidebar: Home, New, Open, Recent, Starred, Folders and Settings.

use crate::app::run;
use crate::icons::{Icon, Logo};
use crate::model::AppModel;
use dioxus::prelude::*;
use kd_commands::{Command, Keymap, Os, Page, SettingsSection, display_sequence};
use kd_store::list_subfolders;
use std::path::PathBuf;

fn hint(cmd: &Command) -> String {
    let os = Os::current();
    Keymap::defaults(os).keys_for(cmd).map(|k| display_sequence(k, os)).unwrap_or_default()
}

#[component]
pub fn Sidebar() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let page = m.page;
    let roots = m.folder_roots();
    let folder = m.folder.clone();
    let (t_home, t_new, t_open, t_recent, t_starred, t_apps, t_chat, t_tools, t_folders, t_add, t_settings) = (
        m.t("nav-home"),
        m.t("nav-new"),
        m.t("nav-open"),
        m.t("nav-recent"),
        m.t("nav-starred"),
        m.t("nav-apps"),
        m.t("nav-ai-chat"),
        m.t("nav-ai-tools"),
        m.t("nav-folders"),
        m.t("nav-add-folder"),
        m.t("nav-settings"),
    );
    let h_new = hint(&Command::NewDocumentPicker);
    let h_open = hint(&Command::OpenDialog);
    let h_settings = hint(&Command::OpenSettings(SettingsSection::General));
    let on = |p: Page| if page == p { "nav active" } else { "nav" };
    rsx! {
        nav { class: "sidebar",
            div { class: "brand", Logo { class: "logo" } "Glyphra" }
            button { class: on(Page::Home), onclick: move |_| run(model, Command::ShowPage(Page::Home)),
                Icon { name: "home" }
                span { class: "t", "{t_home}" }
            }
            button { class: on(Page::New), onclick: move |_| run(model, Command::NewDocumentPicker),
                Icon { name: "plus" }
                span { class: "t", "{t_new}" }
                span { class: "hint", "{h_new}" }
            }
            button { class: "nav", onclick: move |_| run(model, Command::OpenDialog),
                Icon { name: "folder-open" }
                span { class: "t", "{t_open}" }
                span { class: "hint", "{h_open}" }
            }
            button { class: on(Page::Recent), onclick: move |_| run(model, Command::ShowPage(Page::Recent)),
                Icon { name: "clock" }
                span { class: "t", "{t_recent}" }
            }
            button { class: on(Page::Starred), onclick: move |_| run(model, Command::ShowPage(Page::Starred)),
                Icon { name: "star" }
                span { class: "t", "{t_starred}" }
            }
            button { class: on(Page::CraftApps), onclick: move |_| run(model, Command::ShowPage(Page::CraftApps)),
                Icon { name: "apps" }
                span { class: "t", "{t_apps}" }
            }
            div { class: "side-title",
                span { "Glyphra AI" }
            }
            button { class: on(Page::AiChat), onclick: move |_| run(model, Command::ShowPage(Page::AiChat)),
                Icon { name: "chat" }
                span { class: "t", "{t_chat}" }
            }
            button { class: on(Page::AiTools), onclick: move |_| run(model, Command::ShowPage(Page::AiTools)),
                Icon { name: "tools" }
                span { class: "t", "{t_tools}" }
            }
            div { class: "side-title",
                span { "{t_folders}" }
                button { title: "{t_add}", onclick: move |_| run(model, Command::PinFolder), Icon { name: "plus", size: 14 } }
            }
            for root in roots {
                TreeNode { key: "{root.display()}", dir: root, depth: 0usize, current: if page == Page::Folder { Some(folder.clone()) } else { None } }
            }
            div { class: "side-grow" }
            button { class: "nav", onclick: move |_| run(model, Command::OpenSettings(SettingsSection::General)),
                Icon { name: "settings" }
                span { class: "t", "{t_settings}" }
                span { class: "hint", "{h_settings}" }
            }
        }
    }
}

#[component]
fn TreeNode(dir: PathBuf, depth: usize, current: Option<PathBuf>) -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let open = m.is_expanded(&dir);
    let _ = m.revision; // re-read the disk after changes
    let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| dir.display().to_string());
    let children: Vec<PathBuf> = if open { list_subfolders(&dir) } else { Vec::new() };
    let active = current.as_deref() == Some(dir.as_path());
    let pad = depth * 14;
    let (d1, d2, d3) = (dir.clone(), dir.clone(), dir.clone());
    let cur = current.clone();
    let chev = if open { "chev-down" } else { "chev-right" };
    rsx! {
        div { class: if active { "tree-row active" } else { "tree-row" }, style: "padding-inline-start:{pad}px",
            button { class: "chev", onclick: move |_| run(model, Command::ToggleFolder(d1.clone())),
                Icon { name: chev, size: 14 }
            }
            button {
                class: "name",
                title: "{d3.display()}",
                onclick: move |_| run(model, Command::ShowFolder(d2.clone())),
                Icon { name: "folder", size: 16 }
                span { "{name}" }
            }
        }
        for child in children {
            TreeNode { key: "{child.display()}", dir: child, depth: depth + 1, current: cur.clone() }
        }
    }
}
