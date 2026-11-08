//! The Home tab: greeting, Start something new, Apps and the documents table. Recent,
//! Starred, a folder, the New chooser and the full Apps list are other pages of the same tab.

use crate::app::run;
use crate::icons::Icon;
use crate::model::{AppModel, Row, SortKey};
use chrono::Timelike;
use dioxus::prelude::*;
use glyphra_i18n::Arg;
use kd_commands::{Command, Keymap, Os, Page};
use kd_store::{CRAFT_APPS, DocGroup, DocKind, ModuleKind, Opener, human_size};
use std::path::PathBuf;

/// The CSS variable name of a document group's colour.
fn colour(group: DocGroup) -> String {
    format!("var(--g-{})", group.id())
}

#[component]
pub fn HomeTab() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let page = model.read().page;
    rsx! {
        main { class: "main",
            div { class: "content",
                match page {
                    Page::New => rsx! { NewPage {} },
                    Page::CraftApps => rsx! { AppsPage {} },
                    Page::Recent => rsx! { ListPage { title: "page-recent-title", sub: "page-recent-sub" } },
                    Page::Starred => rsx! { ListPage { title: "page-starred-title", sub: "page-starred-sub" } },
                    Page::Folder => rsx! { FolderPage {} },
                    Page::Home => rsx! { StartPage {} },
                }
            }
        }
    }
}

// ------------------------------------------------------------------ start page

#[component]
fn StartPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let part = m.greeting_part(chrono::Local::now().hour());
    let name = m.state.settings.display_name.trim().to_string();
    let title = if name.is_empty() { part } else { m.t_args("home-greeting-named", &[("part", part.into()), ("name", name.into())]) };
    let sub = m.t("home-subtitle");
    drop(m);
    rsx! {
        h1 { class: "greet", "{title}" }
        div { class: "sub", "{sub}" }
        StartNew { into: None::<String> }
        AppsStrip {}
        Documents { heading: true }
    }
}

#[component]
fn StartNew(into: Option<String>) -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let title = m.t("home-start-new");
    let note = into.map(|f| m.t_args("home-start-new-into", &[("folder", f.into())]));
    let open_t = m.t("home-open-file");
    let open_tag = m.t("home-open-file-tagline");
    let tiles: Vec<(ModuleKind, String, String, String)> = ModuleKind::ALL
        .into_iter()
        .map(|k| (k, m.t(&format!("module-{}", k.id())), m.t(&format!("module-{}-tagline", k.id())), m.t("tile-new-document")))
        .collect();
    drop(m);
    rsx! {
        div { class: "section",
            div { class: "section-head",
                h2 { "{title}" }
                if let Some(n) = note {
                    span { class: "s", "{n}" }
                }
            }
            div { class: "cards",
                for (kind, name, tagline, foot) in tiles {
                    button { key: "{kind.id()}", class: "card", onclick: move |_| run(model, Command::NewDocument(kind)),
                        div { class: "lead", style: "background:{colour(kind.group())};color:#fff", "{kind.badge()}" }
                        div { class: "ttl", "{name}" }
                        div { class: "tag", "{tagline}" }
                        div { class: "foot", "{foot}" }
                    }
                }
                button { class: "card", onclick: move |_| run(model, Command::OpenDialog),
                    div { class: "lead", style: "background:var(--surface-hover);color:var(--text-muted)", Icon { name: "folder-open", size: 21 } }
                    div { class: "ttl", "{open_t}" }
                    div { class: "tag", "{open_tag}" }
                }
            }
        }
    }
}

/// One app tile on the start screen and the Apps page.
struct Tile {
    id: String,
    name: &'static str,
    purpose: &'static str,
    badge: &'static str,
    installed: bool,
    version: Option<String>,
    place: String,
    icon: Option<String>,
}

fn tiles(m: &AppModel, installed_only: bool) -> Vec<Tile> {
    CRAFT_APPS
        .iter()
        .filter_map(|a| {
            let found = m.installed.iter().find(|i| i.app.id == a.id);
            if installed_only && found.is_none() {
                return None;
            }
            Some(Tile {
                id: a.id.to_string(),
                name: a.name,
                purpose: a.purpose,
                badge: a.badge,
                installed: found.is_some(),
                version: found.and_then(|i| i.version.clone()),
                place: found.map(|i| i.location()).unwrap_or_default(),
                icon: m.app_icons.get(a.id).cloned(),
            })
        })
        .collect()
}

#[component]
fn AppsStrip() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let n = m.installed.len();
    let title = m.t("apps-title");
    let see_all = m.t("apps-see-all");
    let sub =
        if n == 0 { m.t("apps-subtitle-none") } else { m.t_args("apps-subtitle", &[("n", n.into()), ("total", CRAFT_APPS.len().into())]) };
    let empty = m.t("apps-empty");
    let tiles = tiles(&m, true);
    let t_refresh = m.t("apps-refresh");
    drop(m);
    rsx! {
        div { class: "section",
            div { class: "section-head",
                div { style: "display:flex;align-items:baseline",
                    h2 { "{title}" }
                    span { class: "s", "{sub}" }
                }
                div { style: "display:flex;align-items:center;gap:6px",
                    button { class: "icon-btn", title: "{t_refresh}", onclick: move |_| run(model, Command::Refresh), Icon { name: "refresh", size: 16 } }
                    button { class: "link", onclick: move |_| run(model, Command::ShowPage(Page::CraftApps)), "{see_all}" }
                }
            }
            if tiles.is_empty() {
                div { class: "empty", "{empty}" }
            } else {
                div { class: "apps",
                    for t in tiles {
                        button {
                            key: "{t.id}",
                            class: "app-tile",
                            title: "{t.place}",
                            onclick: {
                                let id = t.id.clone();
                                move |_| run(model, Command::LaunchCraft(id.clone()))
                            },
                            AppIcon { icon: t.icon.clone(), badge: t.badge, name: t.name }
                            div { class: "tx",
                                div { class: "nm", "{t.name}" }
                                div { class: "pp",
                                    if let Some(v) = &t.version {
                                        span { class: "ver", "v{v}" }
                                    }
                                    "{t.purpose}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ other pages

#[component]
fn ListPage(title: &'static str, sub: &'static str) -> Element {
    let model = use_context::<Signal<AppModel>>();
    let (t, s) = {
        let m = model.read();
        (m.t(title), m.t(sub))
    };
    rsx! {
        h1 { class: "page-title", "{t}" }
        div { class: "sub", "{s}" }
        Documents { heading: false }
    }
}

#[component]
fn NewPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let (title, sub) = (m.t("page-new-title"), m.t("page-new-sub"));
    let keymap = Keymap::defaults(Os::current());
    let tiles: Vec<(ModuleKind, String, String, String)> = ModuleKind::ALL
        .into_iter()
        .map(|k| {
            let letter = keymap.keys_for(&Command::NewDocument(k)).and_then(|ks| ks.last()).map(|c| c.key.clone()).unwrap_or_default();
            (k, m.t(&format!("module-{}", k.id())), m.t(&format!("module-{}-tagline", k.id())), letter)
        })
        .collect();
    drop(m);
    let letters: Vec<(String, ModuleKind)> = tiles.iter().map(|(k, _, _, l)| (l.to_lowercase(), *k)).collect();
    rsx! {
        div {
            class: "new-grid",
            tabindex: "0",
            onmounted: move |e| {
                spawn(async move {
                    let _ = e.set_focus(true).await;
                });
            },
            onkeydown: move |e| {
                // Cmd+N, then a letter picks the type
                if let Key::Character(c) = e.key()
                    && let Some((_, kind)) = letters.iter().find(|(l, _)| *l == c.to_lowercase())
                {
                    run(model, Command::NewDocument(*kind));
                }
            },
            h1 { class: "page-title", "{title}" }
            div { class: "sub", "{sub}" }
            div { class: "cards",
                for (kind, name, tagline, letter) in tiles {
                    button { key: "{kind.id()}", class: "card", onclick: move |_| run(model, Command::NewDocument(kind)),
                        div { class: "lead", style: "background:{colour(kind.group())};color:#fff", "{kind.badge()}" }
                        div { class: "ttl", "{name}" }
                        div { class: "tag", "{tagline}" }
                        div { class: "key", "{letter}" }
                    }
                }
            }
        }
    }
}

/// An app's own icon when it could be read from the install, else its coloured badge.
#[component]
fn AppIcon(icon: Option<String>, badge: &'static str, name: &'static str) -> Element {
    match icon {
        Some(uri) => rsx! {
            div { class: "ai img",
                img { src: "{uri}", alt: "{name}" }
            }
        },
        None => rsx! {
            div { class: "ai", style: "background:var(--accent)", "{badge}" }
        },
    }
}

#[component]
fn AppsPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let (title, sub, missing, launcher) =
        (m.t("page-apps-title"), m.t("page-apps-sub"), m.t("apps-not-installed"), m.t("apps-get-launcher"));
    let rows = tiles(&m, false);
    let t_refresh = m.t("apps-refresh");
    drop(m);
    rsx! {
        div { style: "display:flex;align-items:center;justify-content:space-between;gap:12px",
            h1 { class: "page-title", "{title}" }
            button { class: "icon-btn", title: "{t_refresh}", onclick: move |_| run(model, Command::Refresh), Icon { name: "refresh", size: 18 } }
        }
        div { class: "sub", "{sub}" }
        div { class: "apps",
            for t in rows {
                button {
                    key: "{t.id}",
                    class: "app-tile",
                    style: if t.installed { "" } else { "opacity:.55" },
                    title: if t.installed { "{t.place}" } else { "{missing}" },
                    onclick: {
                        let (id, installed) = (t.id.clone(), t.installed);
                        move |_| {
                            if installed {
                                run(model, Command::LaunchCraft(id.clone()));
                            } else {
                                run(model, Command::OpenUrl(kd_store::LAUNCHER_REPO.to_string()));
                            }
                        }
                    },
                    AppIcon { icon: t.icon.clone(), badge: t.badge, name: t.name }
                    div { class: "tx",
                        div { class: "nm", "{t.name}" }
                        div { class: "pp",
                            if let Some(v) = &t.version {
                                span { class: "ver", "v{v}" }
                            }
                            if t.installed { "{t.purpose}" } else { "{launcher}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FolderPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let dir = m.folder.clone();
    let roots = m.folder_roots();
    let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| dir.display().to_string());
    let root = roots.iter().filter(|r| dir.starts_with(r)).max_by_key(|r| r.components().count()).cloned();
    let pinned_root = root.as_deref() == Some(dir.as_path()) && dir != m.state.settings.documents_dir;
    let (t_new, t_reveal, t_remove) = (m.t("folder-new"), m.t("folder-reveal"), m.t("folder-remove"));
    let can_make = dir.is_dir();
    // the path as clickable steps from its root
    let mut steps: Vec<(String, PathBuf)> = Vec::new();
    if let Some(root) = &root
        && let Ok(rest) = dir.strip_prefix(root)
    {
        let mut acc = root.clone();
        steps.push((root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| root.display().to_string()), acc.clone()));
        for part in rest.components() {
            acc = acc.join(part);
            steps.push((part.as_os_str().to_string_lossy().into_owned(), acc.clone()));
        }
    }
    let plain = dir.display().to_string();
    let into = Some(name.clone());
    drop(m);
    let (d1, d2, d3) = (dir.clone(), dir.clone(), dir.clone());
    let last = steps.len().saturating_sub(1);
    rsx! {
        div { class: "folder-head",
            h1 { class: "page-title", style: "margin:0", "{name}" }
            div { style: "display:flex;gap:8px",
                if can_make {
                    button { class: "btn", onclick: move |_| run(model, Command::NewFolder(d1.clone())), "{t_new}" }
                }
                button { class: "btn", onclick: move |_| run(model, Command::RevealInFolder(d2.clone())), "{t_reveal}" }
                if pinned_root {
                    button { class: "btn", onclick: move |_| run(model, Command::UnpinFolder(d3.clone())), "{t_remove}" }
                }
            }
        }
        div { class: "crumbs selectable",
            if steps.len() > 1 {
                for (i , (label , path)) in steps.into_iter().enumerate() {
                    if i == last {
                        span { "{label}" }
                    } else {
                        button { onclick: move |_| run(model, Command::ShowFolder(path.clone())), "{label}" }
                        span { "›" }
                    }
                }
            } else {
                span { "{plain}" }
            }
        }
        StartNew { into }
        Documents { heading: false }
    }
}

// ------------------------------------------------------------------ the documents table

#[component]
fn Documents(heading: bool) -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let menu_open: Signal<Option<MenuAnchor>> = use_signal(|| None);
    let m = model.read();
    let _ = m.revision;
    let rows = m.rows();
    let subfolders = m.subfolders();
    let page = m.page;
    let filter = m.filter;
    let search = m.search.clone();
    let sort = m.sort;
    let title = m.t("docs-title");
    let count = m.t_args("home-files-count", &[("n", Arg::from(rows.len()))]);
    let (t_all, t_ph, t_refresh) = (m.t("filter-all"), m.t("search-placeholder"), m.t("refresh"));
    let pills: Vec<(DocGroup, String)> = DocGroup::ALL.into_iter().map(|g| (g, m.t(&format!("group-{}", g.id())))).collect();
    let when_key = if page == Page::Recent { SortKey::Opened } else { SortKey::Modified };
    let cols = [
        (SortKey::Name, m.t("col-name")),
        (SortKey::Kind, m.t("col-kind")),
        (when_key, m.t(if page == Page::Recent { "col-opened" } else { "col-modified" })),
        (SortKey::Size, m.t("col-size")),
    ];
    let empty = m.t(if filter.is_some() || !search.is_empty() { "docs-empty-filter" } else { "docs-empty" });
    let menu_labels = MenuLabels {
        open: m.t("row-menu-open"),
        reveal: m.t("row-menu-reveal"),
        copy: m.t("row-menu-copy-path"),
        duplicate: m.t("row-menu-duplicate"),
        rename: m.t("row-menu-rename"),
        remove_recent: m.t("row-menu-remove-recent"),
        delete: m.t("row-menu-delete"),
        rename_folder: m.t("folder-rename"),
        delete_folder: m.t("folder-delete"),
    };
    let table_rows: Vec<TableRow> = rows
        .iter()
        .map(|r| {
            let when = if page == Page::Recent { r.opened.unwrap_or(r.entry.modified) } else { r.entry.modified };
            TableRow {
                row: r.clone(),
                kind_label: kind_label(&m, &r.entry.kind),
                when: m.when(when),
                size: if r.entry.exists { human_size(r.entry.size) } else { "—".into() },
                star_label: m.t(if r.starred { "row-unstar" } else { "row-star" }),
            }
        })
        .collect();
    let folder_rows: Vec<(PathBuf, String)> =
        subfolders.iter().map(|d| (d.clone(), d.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())).collect();
    let folder_kind = m.t("kind-folder");
    drop(m);

    let is_empty = table_rows.is_empty() && folder_rows.is_empty();
    let arrow: &'static str = if sort.ascending { "sort-up" } else { "sort-down" };
    rsx! {
        div { class: "section",
            if heading {
                div { class: "section-head",
                    div { style: "display:flex;align-items:baseline",
                        h2 { "{title}" }
                        span { class: "s", "{count}" }
                    }
                }
            }
            div { class: "toolbar",
                div { class: "pills",
                    button { class: if filter.is_none() { "pill on" } else { "pill" }, onclick: move |_| model.write().filter = None, "{t_all}" }
                    for (g , label) in pills {
                        button {
                            key: "{g.id()}",
                            class: if filter == Some(g) { "pill on" } else { "pill" },
                            onclick: move |_| {
                                let mut m = model.write();
                                m.filter = if m.filter == Some(g) { None } else { Some(g) };
                            },
                            "{label}"
                        }
                    }
                }
                div { class: "tools",
                    label { class: "search",
                        Icon { name: "search", size: 15 }
                        input {
                            id: "search",
                            r#type: "text",
                            placeholder: "{t_ph}",
                            value: "{search}",
                            oninput: move |e| model.write().search = e.value(),
                        }
                    }
                    button { class: "icon-btn", title: "{t_refresh}", onclick: move |_| run(model, Command::Refresh), Icon { name: "refresh", size: 16 } }
                }
            }
            if is_empty {
                div { class: "empty", style: "padding:24px 4px", "{empty}" }
            } else {
                div { class: "table",
                    div { class: "thead",
                        span {}
                        for (col , label) in cols {
                            button {
                                key: "{label}",
                                class: if sort.key == col { "on" } else { "" },
                                onclick: move |_| model.write().sort_by(col),
                                "{label}"
                                if sort.key == col {
                                    Icon { name: arrow, size: 12 }
                                }
                            }
                        }
                        span {}
                    }
                    for (dir , name) in folder_rows {
                        FolderRow { key: "{dir.display()}", dir, name, kind: folder_kind.clone(), menu_open }
                    }
                    for tr in table_rows {
                        FileRow { key: "{tr.row.entry.path.display()}", tr, menu_open }
                    }
                }
            }
            if let Some(anchor) = menu_open() {
                RowMenu { anchor: anchor.clone(), labels: menu_labels.clone(), in_recent: in_recent_of(&rows, &anchor.path), menu_open }
            }
        }
    }
}

fn in_recent_of(rows: &[Row], path: &std::path::Path) -> bool {
    rows.iter().any(|r| r.entry.path == path && r.opened.is_some())
}

fn kind_label(m: &AppModel, kind: &DocKind) -> String {
    match kind.opener {
        Opener::Module(_) => m.t(&format!("group-{}", kind.group.id())),
        Opener::Craft(app) => app.name.to_string(),
    }
}

/// Where a row menu was opened: the row's path, the click position and whether it is a folder.
#[derive(Clone, PartialEq)]
pub struct MenuAnchor {
    path: PathBuf,
    x: f64,
    y: f64,
    folder: bool,
}

#[derive(Clone, PartialEq)]
struct MenuLabels {
    open: String,
    reveal: String,
    copy: String,
    duplicate: String,
    rename: String,
    remove_recent: String,
    delete: String,
    rename_folder: String,
    delete_folder: String,
}

#[derive(Clone, PartialEq)]
struct TableRow {
    row: Row,
    kind_label: String,
    when: String,
    size: String,
    star_label: String,
}

#[component]
fn FileRow(tr: TableRow, menu_open: Signal<Option<MenuAnchor>>) -> Element {
    let model = use_context::<Signal<AppModel>>();
    let mut menu_open = menu_open;
    let path = tr.row.entry.path.clone();
    let exists = tr.row.entry.exists;
    let (p_open, p_star, p_menu) = (path.clone(), path.clone(), path.clone());
    let group = tr.row.entry.kind.group;
    rsx! {
        div {
            class: if exists { "trow" } else { "trow gone" },
            onclick: move |_| run(model, Command::OpenPath(p_open.clone())),
            button {
                class: if tr.row.starred { "star on" } else { "star" },
                title: "{tr.star_label}",
                onclick: move |ev| {
                    ev.stop_propagation();
                    run(model, Command::ToggleStar(p_star.clone()));
                },
                Icon { name: "star", size: 17 }
            }
            div { class: "fname",
                span { class: "b", style: "background:{colour(group)}", "{tr.row.entry.kind.badge()}" }
                span { class: "n", "{tr.row.entry.name}" }
            }
            span { class: "cell", "{tr.kind_label}" }
            span { class: "cell", "{tr.when}" }
            span { class: "cell", "{tr.size}" }
            button {
                class: "menu-btn",
                onclick: move |ev| {
                    ev.stop_propagation();
                    let at = ev.client_coordinates();
                    menu_open.set(Some(MenuAnchor { path: p_menu.clone(), x: at.x, y: at.y, folder: false }));
                },
                Icon { name: "more", size: 18 }
            }
        }
    }
}

#[component]
fn FolderRow(dir: PathBuf, name: String, kind: String, menu_open: Signal<Option<MenuAnchor>>) -> Element {
    let model = use_context::<Signal<AppModel>>();
    let mut menu_open = menu_open;
    let (p_open, p_menu) = (dir.clone(), dir.clone());
    rsx! {
        div { class: "trow folder-row", onclick: move |_| run(model, Command::ShowFolder(p_open.clone())),
            span { style: "display:grid;place-items:center;color:var(--text-faint)", Icon { name: "folder", size: 18 } }
            div { class: "fname", span { class: "n", "{name}" } }
            span { class: "cell", "{kind}" }
            span {}
            span {}
            button {
                class: "menu-btn",
                onclick: move |ev| {
                    ev.stop_propagation();
                    let at = ev.client_coordinates();
                    menu_open.set(Some(MenuAnchor { path: p_menu.clone(), x: at.x, y: at.y, folder: true }));
                },
                Icon { name: "more", size: 18 }
            }
        }
    }
}

/// The "…" menu of a row, drawn at the click position on top of everything.
#[component]
fn RowMenu(anchor: MenuAnchor, labels: MenuLabels, in_recent: bool, menu_open: Signal<Option<MenuAnchor>>) -> Element {
    let model = use_context::<Signal<AppModel>>();
    let mut menu_open = menu_open;
    let p = anchor.path.clone();
    // right edge of the menu at the button; never below the bottom of the window
    let left = (anchor.x - 200.0).max(8.0);
    let top = anchor.y + 14.0;
    let style = format!("position:fixed;left:{left}px;top:min({top}px, calc(100vh - 310px));inset-inline-end:auto;z-index:30");
    let mut item = move |cmd: Command| {
        menu_open.set(None);
        run(model, cmd);
    };
    rsx! {
        div { class: "scrim-clear", onclick: move |_| menu_open.set(None) }
        div { class: "popup", style: "{style}",
            if anchor.folder {
                button { onclick: { let p = p.clone(); move |_| item(Command::RevealInFolder(p.clone())) }, "{labels.reveal}" }
                button { onclick: { let p = p.clone(); move |_| item(Command::RenameFolder(p.clone())) }, "{labels.rename_folder}" }
                hr {}
                button { class: "danger", onclick: { let p = p.clone(); move |_| item(Command::DeleteFolder(p.clone())) }, "{labels.delete_folder}" }
            } else {
                button { onclick: { let p = p.clone(); move |_| item(Command::OpenPath(p.clone())) }, "{labels.open}" }
                button { onclick: { let p = p.clone(); move |_| item(Command::RevealInFolder(p.clone())) }, "{labels.reveal}" }
                button { onclick: { let p = p.clone(); move |_| item(Command::CopyPath(p.clone())) }, "{labels.copy}" }
                hr {}
                button { onclick: { let p = p.clone(); move |_| item(Command::DuplicateFile(p.clone())) }, "{labels.duplicate}" }
                button { onclick: { let p = p.clone(); move |_| item(Command::RenameFile(p.clone())) }, "{labels.rename}" }
                if in_recent {
                    button { onclick: { let p = p.clone(); move |_| item(Command::RemoveFromRecent(p.clone())) }, "{labels.remove_recent}" }
                }
                hr {}
                button { class: "danger", onclick: { let p = p.clone(); move |_| item(Command::DeleteFile(p.clone())) }, "{labels.delete}" }
            }
        }
    }
}
