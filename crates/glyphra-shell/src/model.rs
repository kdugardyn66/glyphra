//! The state of the window and the one place commands run.
//!
//! Everything the user can do is a [`Command`]; [`AppModel::dispatch`] runs it, changes the
//! state and returns the [`Effect`]s only the platform can carry out (a file dialog, the
//! Trash, quitting). The model knows nothing about Dioxus, so all of this is tested without a
//! window.

use crate::ai::{AiState, ToolRun};
use crate::preview;
use glyphra_i18n::{Arg, Catalog, I18n};
use kd_commands::menu::{MenuView, WindowItem};
use kd_commands::{Command, Page, SettingsSection};
use kd_store::{
    DocGroup, DocKind, FileEntry, HomeState, InstalledApp, ModuleKind, OpenIn, Opener, Settings, Store, ThemePref, create_folder,
    create_new_document, detect_installed, duplicate_file, list_documents, list_subfolders, now, rename_file, rename_folder,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Something only the platform layer can do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Quit,
    /// Show the system Open dialog; the answer comes back as [`Command::OpenPath`].
    OpenFileDialog,
    /// Show the system folder dialog; the answer comes back through [`AppModel::folder_picked`].
    PickFolderDialog,
    Reveal(PathBuf),
    OpenExternal(PathBuf),
    OpenUrl(String),
    CopyText(String),
    /// Move to the Trash; report back with [`AppModel::trashed`].
    Trash(PathBuf),
    LaunchCraft {
        id: String,
        file: Option<PathBuf>,
    },
    FocusSearch,
    /// Bring the main window to the front.
    FocusMainWindow,
    /// Show the folder dialog for the documents folder; the answer goes to [`AppModel::draft_documents_dir`].
    PickDocumentsFolderDialog,
    /// Open a document in a window of its own (or bring that window forward).
    OpenInWindow(PathBuf),
    /// Show the system Open dialog for a file to add to the AI chat; the answer goes to [`AppModel::ai_attached`].
    AiAttachDialog,
}

/// What a tab shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabBody {
    Home,
    /// A read-only preview of a Markdown file.
    Markdown {
        html: String,
    },
    /// A module whose editor has not shipped yet.
    Soon(ModuleKind),
    /// The file could not be read.
    Unreadable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub path: Option<PathBuf>,
    pub title: String,
    pub body: TabBody,
}

impl Tab {
    fn home() -> Tab {
        Tab { path: None, title: String::new(), body: TabBody::Home }
    }
}

/// A question on top of the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    /// Ask for a text (rename, new folder).
    Prompt { kind: PromptKind, path: PathBuf, value: String, error: Option<String> },
    /// Ask to confirm something that cannot be undone from here.
    Confirm { kind: ConfirmKind, path: PathBuf },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    RenameFile,
    RenameFolder,
    NewFolder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmKind {
    DeleteFile,
    DeleteFolder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Kind,
    Modified,
    Opened,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub key: SortKey,
    pub ascending: bool,
}

/// A short message at the bottom of the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub id: u64,
    pub text: String,
    pub error: bool,
}

/// One row of the documents table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub entry: FileEntry,
    pub starred: bool,
    pub opened: Option<u64>,
}

pub struct AppModel {
    pub store: Store,
    pub state: HomeState,
    pub catalog: Catalog,
    pub i18n: I18n,
    pub system_language: String,
    pub tabs: Vec<Tab>,
    pub active: usize,
    closed: Vec<PathBuf>,
    pub page: Page,
    pub folder: PathBuf,
    pub filter: Option<DocGroup>,
    pub search: String,
    pub sort: Sort,
    pub installed: Vec<InstalledApp>,
    pub modal: Option<Modal>,
    pub settings: Option<SettingsSection>,
    /// The settings being edited in the dialog; only [`AppModel::apply_settings`] makes them real.
    pub draft: Settings,
    /// `data:` URIs of the icons of installed Crafting Apps, by app id.
    pub app_icons: HashMap<String, String>,
    /// This is a document window: documents open in its own tab, never in another window,
    /// and nothing is written to the settings or the library.
    pub detached: bool,
    /// Documents that have a window of their own right now (kept up to date by the window layer).
    pub open_windows: Vec<PathBuf>,
    /// Glyphra AI: providers, keys, the chat and the tools.
    pub ai: AiState,
    pub toast: Option<Toast>,
    toast_seq: u64,
    /// Bumped whenever files may have changed, so views re-read the disk.
    pub revision: u64,
}

impl AppModel {
    /// A cheap stand-in used while the real model loads: no disk access, so the splash can
    /// already show text in the system language.
    pub fn placeholder(catalog: Catalog, system_language: String) -> AppModel {
        let store = Store::platform();
        let state = HomeState::new(Store::default_documents_dir());
        let i18n = I18n::new(&catalog, "", &system_language);
        Self::assemble(store, state, catalog, i18n, system_language, Vec::new())
    }

    /// The real start-up: the config folder, the user's language packs, the library and the
    /// installed apps. Slow parts are here, so this runs off the UI thread behind the splash.
    pub fn boot(system_language: String) -> AppModel {
        let store = Store::platform();
        let (catalog, issues) = Catalog::load(Some(&store.locales_dir()));
        let mut m = Self::load(store, catalog, system_language);
        if let Some(first) = issues.first() {
            m.fail("error-language-pack", &format!("{}: {}", first.folder, first.problem));
        }
        m
    }

    fn assemble(
        store: Store,
        state: HomeState,
        catalog: Catalog,
        i18n: I18n,
        system_language: String,
        installed: Vec<InstalledApp>,
    ) -> AppModel {
        let folder = state.settings.documents_dir.clone();
        AppModel {
            installed,
            store,
            catalog,
            i18n,
            system_language,
            state,
            tabs: vec![Tab::home()],
            active: 0,
            closed: Vec::new(),
            page: Page::Home,
            folder,
            filter: None,
            search: String::new(),
            sort: Sort { key: SortKey::Modified, ascending: false },
            modal: None,
            settings: None,
            draft: Settings::with_documents_dir(PathBuf::new()),
            app_icons: HashMap::new(),
            detached: false,
            open_windows: Vec::new(),
            ai: AiState::empty(),
            toast: None,
            toast_seq: 0,
            revision: 0,
        }
    }

    /// Load settings and the library, the languages and the installed apps.
    pub fn load(store: Store, catalog: Catalog, system_language: String) -> AppModel {
        Self::load_with(store, catalog, system_language, Arc::new(kd_store::Keychain))
    }

    /// [`AppModel::load`] with the secrets kept somewhere else (tests keep them in memory).
    pub fn load_with(store: Store, catalog: Catalog, system_language: String, secrets: Arc<dyn kd_store::SecretStore>) -> AppModel {
        let (state, _first) = store.load_or_init();
        let i18n = I18n::new(&catalog, &state.settings.language, &system_language);
        let dir = store.dir().to_path_buf();
        let mut m = Self::assemble(store, state, catalog, i18n, system_language, detect_installed());
        m.load_app_icons();
        let (ai, warning) = AiState::load(dir, secrets);
        m.ai = ai;
        if let Some(w) = warning {
            m.fail("error-keychain", &w);
        }
        m
    }

    /// Read the icons of the installed Crafting Apps (see [`kd_store::appicon`]).
    fn load_app_icons(&mut self) {
        let cache = self.store.dir().join("icon-cache");
        self.app_icons = self
            .installed
            .iter()
            .filter_map(|i| kd_store::appicon::icon_data_uri(i, &cache).map(|uri| (i.app.id.to_string(), uri)))
            .collect();
    }

    /// A model for a document window: the settings and language of the main window, one
    /// document in a tab, no writes.
    pub fn for_document_window(path: &Path, system_language: String) -> AppModel {
        let store = Store::platform();
        let (catalog, _) = Catalog::load(Some(&store.locales_dir()));
        let (state, _) = store.load_or_init();
        let i18n = I18n::new(&catalog, &state.settings.language, &system_language);
        let mut m = Self::assemble(store, state, catalog, i18n, system_language, Vec::new());
        m.detached = true;
        let _ = m.open_path(path);
        m
    }

    // ------------------------------------------------------------------ small helpers

    pub fn t(&self, id: &str) -> String {
        self.i18n.t(id)
    }

    pub fn t_args(&self, id: &str, args: &[(&str, Arg)]) -> String {
        self.i18n.t_args(id, args)
    }

    fn say(&mut self, text: String, error: bool) {
        self.toast_seq += 1;
        self.toast = Some(Toast { id: self.toast_seq, text, error });
    }

    fn say_id(&mut self, id: &str) {
        self.notify(id);
    }

    fn fail(&mut self, id: &str, detail: &str) {
        self.notify_error(id, detail);
    }

    /// Show a message (a Fluent id) at the bottom of the window.
    pub fn notify(&mut self, id: &str) {
        let t = self.t(id);
        self.say(t, false);
    }

    /// Show an error (a Fluent id with a `$detail` variable).
    pub fn notify_error(&mut self, id: &str, detail: &str) {
        let t = self.t_args(id, &[("detail", detail.into())]);
        self.say(t, true);
    }

    /// Write `home.json` and the library; a failure is shown, never swallowed.
    pub fn persist(&mut self) {
        if let Err(e) = self.store.save(&self.state) {
            self.fail("error-save-settings", &e.to_string());
        }
    }

    fn set_language(&mut self, tag: &str) {
        self.state.settings.language = tag.to_string();
        self.i18n = I18n::new(&self.catalog, tag, &self.system_language);
    }

    /// What the native menu needs: whether a document is in front, and the Window menu's list
    /// (the home window, every open tab, every document window).
    pub fn menu_view(&self) -> MenuView {
        let mut windows = vec![WindowItem { title: self.t("app-name"), command: Command::ActivateTab(0), checked: self.active == 0 }];
        for (i, tab) in self.tabs.iter().enumerate().skip(1) {
            windows.push(WindowItem { title: tab.title.clone(), command: Command::ActivateTab(i), checked: self.active == i });
        }
        for path in &self.open_windows {
            let title = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            windows.push(WindowItem { title, command: Command::OpenPath(path.clone()), checked: false });
        }
        MenuView { document: self.active != 0, windows }
    }

    pub fn is_home(&self) -> bool {
        self.active == 0
    }

    /// Folders shown in the sidebar: the documents folder, then the pinned ones.
    pub fn folder_roots(&self) -> Vec<PathBuf> {
        let mut v = vec![self.state.settings.documents_dir.clone()];
        for f in &self.state.settings.folders {
            if !v.contains(f) {
                v.push(f.clone());
            }
        }
        v
    }

    pub fn is_expanded(&self, dir: &Path) -> bool {
        self.state.settings.expanded_folders.iter().any(|p| p == dir)
    }

    // ------------------------------------------------------------------ the documents table

    /// The rows of the current page, filtered, searched and sorted.
    pub fn rows(&self) -> Vec<Row> {
        let lib = &self.state.library;
        let mut entries: Vec<FileEntry> = match self.page {
            Page::Recent => lib.recent.iter().filter_map(|r| FileEntry::from_path(&r.path)).collect(),
            Page::Starred => lib.starred.iter().filter_map(|p| FileEntry::from_path(p)).collect(),
            Page::Folder => list_documents(&self.folder),
            _ => {
                let mut v = list_documents(&self.state.settings.documents_dir);
                for p in lib.recent_existing(50) {
                    if !v.iter().any(|e| e.path == p)
                        && let Some(e) = FileEntry::from_path(&p)
                    {
                        v.push(e);
                    }
                }
                v
            }
        };
        if let Some(g) = self.filter {
            entries.retain(|e| e.kind.group == g);
        }
        let q = self.search.trim().to_lowercase();
        if !q.is_empty() {
            entries.retain(|e| e.name.to_lowercase().contains(&q));
        }
        let mut rows: Vec<Row> = entries
            .into_iter()
            .map(|entry| Row { starred: lib.is_starred(&entry.path), opened: lib.opened_at(&entry.path), entry })
            .collect();
        let Sort { key, ascending } = self.sort;
        rows.sort_by(|a, b| {
            let o = match key {
                SortKey::Name => a.entry.name.to_lowercase().cmp(&b.entry.name.to_lowercase()),
                SortKey::Kind => {
                    a.entry.kind.app_name().cmp(b.entry.kind.app_name()).then(a.entry.name.to_lowercase().cmp(&b.entry.name.to_lowercase()))
                }
                SortKey::Modified => a.entry.modified.cmp(&b.entry.modified),
                SortKey::Opened => a.opened.unwrap_or(0).cmp(&b.opened.unwrap_or(0)),
                SortKey::Size => a.entry.size.cmp(&b.entry.size),
            };
            if ascending { o } else { o.reverse() }
        });
        rows
    }

    /// Sub-folders shown above the table on a folder page (matching the search).
    pub fn subfolders(&self) -> Vec<PathBuf> {
        if self.page != Page::Folder {
            return Vec::new();
        }
        let q = self.search.trim().to_lowercase();
        list_subfolders(&self.folder)
            .into_iter()
            .filter(|d| q.is_empty() || d.file_name().is_some_and(|n| n.to_string_lossy().to_lowercase().contains(&q)))
            .collect()
    }

    pub fn sort_by(&mut self, key: SortKey) {
        self.sort = if self.sort.key == key {
            Sort { key, ascending: !self.sort.ascending }
        } else {
            Sort { key, ascending: matches!(key, SortKey::Name | SortKey::Kind) }
        };
    }

    // ------------------------------------------------------------------ words for the UI

    /// "5 min ago", "yesterday" …, in the language in use.
    pub fn when(&self, then: u64) -> String {
        self.when_at(then, now())
    }

    pub fn when_at(&self, then: u64, now: u64) -> String {
        if then == 0 {
            return "—".into();
        }
        let d = now.saturating_sub(then);
        let n = |v: u64| [("n", Arg::from(v))];
        match d {
            0..=59 => self.t("time-just-now"),
            60..=3_599 => self.t_args("time-minutes-ago", &n(d / 60)),
            3_600..=86_399 => self.t_args("time-hours-ago", &n(d / 3_600)),
            86_400..=172_799 => self.t("time-yesterday"),
            _ if d < 30 * 86_400 => self.t_args("time-days-ago", &n(d / 86_400)),
            _ if d < 365 * 86_400 => self.t_args("time-months-ago", &n(d / (30 * 86_400))),
            _ => self.t_args("time-years-ago", &n(d / (365 * 86_400))),
        }
    }

    /// "Good morning" … for an hour of the day (0-23).
    pub fn greeting_part(&self, hour: u32) -> String {
        self.t(match hour {
            5..=11 => "greeting-morning",
            12..=17 => "greeting-afternoon",
            _ => "greeting-evening",
        })
    }

    // ------------------------------------------------------------------ opening things

    fn go_home(&mut self) {
        self.active = 0;
    }

    fn open_path(&mut self, path: &Path) -> Vec<Effect> {
        let Some(kind) = DocKind::from_path(path) else {
            return vec![Effect::OpenExternal(path.to_path_buf())];
        };
        if let Opener::Craft(app) = kind.opener {
            return vec![Effect::LaunchCraft { id: app.id.to_string(), file: Some(path.to_path_buf()) }];
        }
        if !path.is_file() {
            self.fail("error-file-missing", &path.display().to_string());
            self.state.library.remove_recent(path);
            self.persist();
            return Vec::new();
        }
        if !self.detached && self.state.settings.open_files_in == OpenIn::Window {
            self.state.library.touch(path, now());
            self.persist();
            return vec![Effect::OpenInWindow(path.to_path_buf())];
        }
        // already open: switch to it
        if let Some(i) = self.tabs.iter().position(|t| t.path.as_deref() == Some(path)) {
            self.active = i;
        } else {
            let tab = self.read_tab(path, kind);
            self.tabs.push(tab);
            self.active = self.tabs.len() - 1;
        }
        if !self.detached {
            self.state.library.touch(path, now());
            self.persist();
        }
        Vec::new()
    }

    fn read_tab(&self, path: &Path, kind: DocKind) -> Tab {
        let title = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let body = match kind.opener {
            Opener::Module(ModuleKind::Markdown) => match fs::metadata(path) {
                Ok(m) if m.len() > preview::MAX_PREVIEW_BYTES => TabBody::Unreadable(self.t("error-too-large")),
                Ok(_) => match fs::read_to_string(path) {
                    Ok(text) => TabBody::Markdown { html: preview::render(&text) },
                    Err(e) => TabBody::Unreadable(e.to_string()),
                },
                Err(e) => TabBody::Unreadable(e.to_string()),
            },
            Opener::Module(m) => TabBody::Soon(m),
            Opener::Craft(_) => TabBody::Unreadable(String::new()),
        };
        Tab { path: Some(path.to_path_buf()), title, body }
    }

    fn close_tab(&mut self, i: usize) {
        if i == 0 || i >= self.tabs.len() {
            return; // Home cannot be closed
        }
        let tab = self.tabs.remove(i);
        if let Some(p) = tab.path {
            self.closed.push(p);
        }
        if self.active == i {
            self.active = i.min(self.tabs.len() - 1);
        } else if self.active > i {
            self.active -= 1;
        }
    }

    fn new_document(&mut self, kind: ModuleKind) -> Vec<Effect> {
        let dir = if self.page == Page::Folder && self.is_home() { self.folder.clone() } else { self.state.settings.documents_dir.clone() };
        match create_new_document(kind, &dir) {
            Ok(path) => {
                // coming from the New page: go back to the list, which now shows the new file
                if self.page == Page::New {
                    self.page = Page::Home;
                }
                self.revision += 1;
                self.open_path(&path)
            }
            Err(e) => {
                self.fail("error-create", &e.to_string());
                Vec::new()
            }
        }
    }

    // ------------------------------------------------------------------ the dispatcher

    /// Run a command.
    pub fn dispatch(&mut self, cmd: Command) -> Vec<Effect> {
        // The settings dialog is modal: until it is closed, everything else is ignored.
        if self.settings.is_some() && !matches!(cmd, Command::OpenSettings(_) | Command::Quit) {
            return Vec::new();
        }
        match cmd {
            Command::NewDocumentPicker => {
                self.go_home();
                self.page = Page::New;
            }
            Command::NewDocument(kind) => return self.new_document(kind),
            Command::OpenDialog => return vec![Effect::OpenFileDialog],
            Command::OpenPath(p) => return self.open_path(&p),
            Command::Save | Command::SaveAs | Command::Print => self.say_id("toast-editor-later"),
            Command::DeleteFile(p) => self.modal = Some(Modal::Confirm { kind: ConfirmKind::DeleteFile, path: p }),
            Command::RenameFile(p) => {
                let value = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                self.modal = Some(Modal::Prompt { kind: PromptKind::RenameFile, path: p, value, error: None });
            }
            Command::DuplicateFile(p) => match duplicate_file(&p) {
                Ok(_) => self.revision += 1,
                Err(e) => self.fail("error-duplicate", &e.to_string()),
            },
            Command::PinFolder => return vec![Effect::PickFolderDialog],
            Command::UnpinFolder(p) => {
                self.state.settings.folders.retain(|f| *f != p);
                if self.page == Page::Folder && self.folder == p {
                    self.page = Page::Home;
                }
                self.persist();
            }
            Command::ShowFolder(p) => {
                self.go_home();
                self.page = Page::Folder;
                self.folder = p;
                self.search.clear();
                self.filter = None;
            }
            Command::ToggleFolder(p) => {
                let open = &mut self.state.settings.expanded_folders;
                if let Some(i) = open.iter().position(|f| *f == p) {
                    open.remove(i);
                } else {
                    open.push(p);
                }
                self.persist();
            }
            Command::NewFolder(p) => {
                self.modal = Some(Modal::Prompt { kind: PromptKind::NewFolder, path: p, value: String::new(), error: None })
            }
            Command::RenameFolder(p) => {
                let value = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                self.modal = Some(Modal::Prompt { kind: PromptKind::RenameFolder, path: p, value, error: None });
            }
            Command::DeleteFolder(p) => self.modal = Some(Modal::Confirm { kind: ConfirmKind::DeleteFolder, path: p }),
            Command::ShowPage(page) => {
                self.go_home();
                self.page = page;
                self.search.clear();
                self.filter = None;
                self.sort = match page {
                    Page::Recent => Sort { key: SortKey::Opened, ascending: false },
                    _ => Sort { key: SortKey::Modified, ascending: false },
                };
                self.revision += 1;
            }
            Command::OpenSettings(section) => self.open_settings(section),
            Command::LaunchCraft(id) => return vec![Effect::LaunchCraft { id, file: None }],
            Command::ActivateTab(i) => {
                if i < self.tabs.len() {
                    self.active = i;
                }
                return vec![Effect::FocusMainWindow];
            }
            Command::NextTab => self.active = (self.active + 1) % self.tabs.len(),
            Command::PreviousTab => self.active = (self.active + self.tabs.len() - 1) % self.tabs.len(),
            Command::CloseTab(i) => self.close_tab(i),
            Command::CloseActiveTab => self.close_tab(self.active),
            Command::ReopenClosedTab => {
                if let Some(p) = self.closed.pop() {
                    return self.open_path(&p);
                }
            }
            Command::ToggleStar(p) => {
                self.state.library.toggle_star(&p);
                self.persist();
            }
            Command::RemoveFromRecent(p) => {
                self.state.library.remove_recent(&p);
                self.persist();
            }
            Command::OpenExternally(p) => return vec![Effect::OpenExternal(p)],
            Command::RevealInFolder(p) => return vec![Effect::Reveal(p)],
            Command::CopyPath(p) => return vec![Effect::CopyText(p.display().to_string())],
            Command::SetTheme(t) => {
                self.state.settings.theme = t;
                self.persist();
            }
            Command::CycleTheme => {
                self.state.settings.theme = self.state.settings.theme.cycle();
                self.persist();
            }
            Command::OpenUrl(u) => return vec![Effect::OpenUrl(u)],
            Command::Refresh => {
                self.installed = detect_installed();
                self.load_app_icons();
                self.state.library.prune_starred();
                self.revision += 1;
            }
            Command::FocusSearch => {
                self.go_home();
                return vec![Effect::FocusSearch];
            }
            Command::AiSend(text) => {
                self.ai.chat.draft = text;
                let context = self.ai_context();
                self.ai.chat.send(&self.ai.hub, context);
                self.ai.attachment = None;
            }
            Command::AiStop => {
                self.ai.chat.stop();
                if let Some(run) = &self.ai.tool
                    && let Some(job) = &run.job
                {
                    job.cancel();
                }
            }
            Command::AiNewChat => {
                self.ai.chat.clear();
                self.ai.attachment = None;
            }
            Command::AiSelectModel(m) => self.ai.chat.model = m.as_deref().and_then(kd_ai::ModelRef::parse),
            Command::AiAttach => return vec![Effect::AiAttachDialog],
            Command::AiDetach => self.ai.attachment = None,
            Command::AiToggleStarred => self.ai.chat.use_starred = !self.ai.chat.use_starred,
            Command::AiOpenTool(id) => {
                if let Some(task) = kd_ai::tasks::task(&id) {
                    self.go_home();
                    self.page = Page::AiTools;
                    self.ai.tool = Some(ToolRun { task, output: String::new(), error: None, job: None });
                }
            }
            Command::AiCloseTool => {
                if let Some(run) = self.ai.tool.take()
                    && let Some(job) = run.job
                {
                    job.cancel();
                }
            }
            Command::AiRunTool { input, note } => {
                let hub = self.ai.hub.clone();
                if let Some(run) = self.ai.tool.as_mut()
                    && run.job.is_none()
                    && !input.trim().is_empty()
                {
                    run.output.clear();
                    run.error = None;
                    run.job = Some(hub.start(run.task.request_with(&note, &input), None));
                }
            }
            Command::AiSaveToolResult => return self.save_tool_result(),
            Command::Quit => return vec![Effect::Quit],
        }
        Vec::new()
    }

    // ------------------------------------------------------------------ Glyphra AI

    /// The extra context of the next chat message: the attached file, and the starred files when asked.
    fn ai_context(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some((_, text)) = &self.ai.attachment {
            parts.push(text.clone());
        }
        if self.ai.chat.use_starred
            && let Some(t) = kd_ai::starred_context(&self.state.library.starred, kd_ai::CONTEXT_BUDGET)
        {
            parts.push(t);
        }
        (!parts.is_empty()).then(|| parts.join("\n\n"))
    }

    /// The Open dialog picked a file for the AI chat.
    pub fn ai_attached(&mut self, path: &Path) {
        match kd_ai::file_context(path, kd_ai::CONTEXT_BUDGET) {
            Some(text) => {
                let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.ai.attachment = Some((name, text));
            }
            None => self.notify("toast-ai-unreadable"),
        }
    }

    /// Write the open tool's result to a new Markdown document and open it.
    fn save_tool_result(&mut self) -> Vec<Effect> {
        let Some(text) = self.ai.tool.as_ref().map(|r| r.output.clone()).filter(|t| !t.trim().is_empty()) else { return Vec::new() };
        let dir = self.state.settings.documents_dir.clone();
        match create_new_document(ModuleKind::Markdown, &dir).and_then(|p| fs::write(&p, &text).map(|()| p)) {
            Ok(path) => {
                self.revision += 1;
                self.open_path(&path)
            }
            Err(e) => {
                self.fail("error-create", &e.to_string());
                Vec::new()
            }
        }
    }

    // ------------------------------------------------------------------ answers to questions

    /// The user confirmed or submitted the modal.
    pub fn submit_modal(&mut self) -> Vec<Effect> {
        let Some(modal) = self.modal.clone() else { return Vec::new() };
        match modal {
            Modal::Confirm { kind, path } => {
                self.modal = None;
                match kind {
                    ConfirmKind::DeleteFile | ConfirmKind::DeleteFolder => vec![Effect::Trash(path)],
                }
            }
            Modal::Prompt { kind, path, value, .. } => {
                let result = match kind {
                    PromptKind::RenameFile => rename_file(&path, &value),
                    PromptKind::RenameFolder => rename_folder(&path, &value),
                    PromptKind::NewFolder => create_folder(&path, &value),
                };
                match result {
                    Ok(new) => {
                        match kind {
                            PromptKind::RenameFile => {
                                self.state.library.rename(&path, &new);
                                for t in &mut self.tabs {
                                    if t.path.as_deref() == Some(path.as_path()) {
                                        t.title = new.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                        t.path = Some(new.clone());
                                    }
                                }
                            }
                            PromptKind::RenameFolder => {
                                self.state.library.rename_dir(&path, &new);
                                for f in self.state.settings.folders.iter_mut().chain(self.state.settings.expanded_folders.iter_mut()) {
                                    if *f == path {
                                        *f = new.clone();
                                    }
                                }
                                if self.folder == path {
                                    self.folder = new.clone();
                                }
                            }
                            PromptKind::NewFolder => {
                                if !self.is_expanded(&path) {
                                    self.state.settings.expanded_folders.push(path.clone());
                                }
                            }
                        }
                        self.modal = None;
                        self.revision += 1;
                        self.persist();
                    }
                    Err(e) => {
                        if let Some(Modal::Prompt { error, .. }) = &mut self.modal {
                            *error = Some(e.to_string());
                        }
                    }
                }
                Vec::new()
            }
        }
    }

    pub fn cancel_modal(&mut self) {
        self.modal = None;
    }

    pub fn edit_prompt(&mut self, text: String) {
        if let Some(Modal::Prompt { value, error, .. }) = &mut self.modal {
            *value = text;
            *error = None;
        }
    }

    /// The platform reports how moving a path to the Trash went.
    pub fn trashed(&mut self, path: &Path, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.state.library.forget(path);
                self.state.settings.folders.retain(|f| f != path);
                self.state.settings.expanded_folders.retain(|f| !f.starts_with(path));
                if let Some(i) = self.tabs.iter().position(|t| t.path.as_deref() == Some(path)) {
                    self.close_tab(i);
                }
                if self.page == Page::Folder && self.folder.starts_with(path) {
                    self.page = Page::Home;
                }
                self.revision += 1;
                self.persist();
            }
            Err(e) => self.fail("error-trash", &e),
        }
    }

    /// The user picked a folder to pin to the sidebar.
    pub fn folder_picked(&mut self, dir: PathBuf) {
        if !self.folder_roots().contains(&dir) {
            self.state.settings.folders.push(dir.clone());
            self.persist();
        }
        self.go_home();
        self.page = Page::Folder;
        self.folder = dir;
    }

    // ------------------------------------------------------------------ settings

    /// Show the dialog (or another section of it). Opening it starts a fresh draft.
    pub fn open_settings(&mut self, section: SettingsSection) {
        if self.settings.is_none() {
            self.draft = self.state.settings.clone();
            self.ai.reset_draft();
        }
        self.settings = Some(section);
    }

    /// Close the dialog and drop what was not applied.
    pub fn close_settings(&mut self) {
        self.settings = None;
        self.draft = self.state.settings.clone();
        self.ai.reset_draft();
    }

    /// The editable settings as one comparable value.
    fn editable(s: &Settings) -> (&str, ThemePref, &str, &Path, OpenIn) {
        (&s.display_name, s.theme, &s.language, &s.documents_dir, s.open_files_in)
    }

    /// Has anything been changed in the dialog (is Apply useful)?
    pub fn settings_dirty(&self) -> bool {
        Self::editable(&self.draft) != Self::editable(&self.state.settings) || self.ai.dirty()
    }

    /// Make the draft real: save it, switch language and theme, create the documents folder.
    /// Returns false (dialog stays open, an error is shown) when the folder cannot be made.
    pub fn apply_settings(&mut self) -> bool {
        if !self.settings_dirty() {
            return true;
        }
        if self.ai.dirty()
            && let Err(e) = self.ai.save()
        {
            self.fail("error-keychain", &e);
        }
        let d = self.draft.clone();
        if d.documents_dir.as_os_str().is_empty() {
            self.fail("error-docs-folder", "");
            return false;
        }
        if d.documents_dir != self.state.settings.documents_dir
            && let Err(e) = fs::create_dir_all(&d.documents_dir)
        {
            self.fail("error-docs-folder", &e.to_string());
            return false;
        }
        let old_docs = std::mem::replace(&mut self.state.settings.documents_dir, d.documents_dir.clone());
        if self.folder == old_docs {
            self.folder = d.documents_dir.clone();
        }
        let s = &mut self.state.settings;
        s.display_name = d.display_name.trim().to_string();
        s.theme = d.theme;
        s.open_files_in = d.open_files_in;
        s.open_modules_in = d.open_files_in;
        if s.language != d.language {
            self.set_language(&d.language);
        }
        self.draft = self.state.settings.clone();
        self.revision += 1;
        self.persist();
        true
    }

    pub fn draft_documents_dir(&mut self, dir: PathBuf) {
        self.draft.documents_dir = dir;
    }

    pub fn dismiss_toast(&mut self, id: u64) {
        if self.toast.as_ref().is_some_and(|t| t.id == id) {
            self.toast = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kd_commands::menu::menu_bar;

    /// A model with its own config and documents folder.
    fn model(tmp: &Path) -> AppModel {
        let store = Store::at(tmp.join("cfg"));
        let (catalog, issues) = Catalog::load(None);
        assert!(issues.is_empty());
        let mut m = AppModel::load_with(store, catalog, "en".into(), std::sync::Arc::new(kd_store::MemorySecrets::default()));
        m.state.settings.documents_dir = tmp.join("docs");
        m.folder = tmp.join("docs");
        fs::create_dir_all(&m.folder).unwrap();
        m.installed.clear();
        m
    }

    #[test]
    fn every_menu_item_runs_its_command_without_panicking() {
        let tmp = tempfile::tempdir().unwrap();
        for mac in [true, false] {
            let mut m = model(tmp.path());
            for menu in menu_bar(mac) {
                for c in menu.commands() {
                    let effects = m.dispatch(c.clone());
                    // a command either changes the state or asks the platform for something
                    let _ = effects;
                }
            }
            // closing the tabs the menu opened leaves a sane window
            assert!(m.active < m.tabs.len());
        }
    }

    #[test]
    fn new_documents_open_in_tabs_and_land_in_recent() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        m.dispatch(Command::NewDocument(ModuleKind::Markdown));
        assert_eq!(m.tabs.len(), 2);
        assert_eq!(m.active, 1);
        assert!(matches!(m.tabs[1].body, TabBody::Markdown { .. }));
        assert_eq!(m.state.library.recent.len(), 1);
        m.dispatch(Command::NewDocument(ModuleKind::Pdf));
        assert_eq!(m.tabs[2].body, TabBody::Soon(ModuleKind::Pdf));
        m.dispatch(Command::NewDocument(ModuleKind::Drawing));
        assert_eq!(m.tabs[3].body, TabBody::Soon(ModuleKind::Drawing));
        assert_eq!(m.rows().len(), 3);
    }

    #[test]
    fn creating_from_the_new_page_returns_to_the_list() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        m.dispatch(Command::NewDocumentPicker);
        assert_eq!(m.page, Page::New);
        m.dispatch(Command::NewDocument(ModuleKind::Markdown));
        assert_eq!(m.page, Page::Home);
    }

    #[test]
    fn opening_an_open_file_switches_to_its_tab() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let p = tmp.path().join("docs/a.md");
        fs::write(&p, "# A").unwrap();
        m.dispatch(Command::OpenPath(p.clone()));
        m.dispatch(Command::ActivateTab(0));
        m.dispatch(Command::OpenPath(p));
        assert_eq!(m.tabs.len(), 2);
        assert_eq!(m.active, 1);
    }

    #[test]
    fn tabs_close_and_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        for n in ["a", "b", "c"] {
            let p = tmp.path().join(format!("docs/{n}.md"));
            fs::write(&p, n).unwrap();
            m.dispatch(Command::OpenPath(p));
        }
        assert_eq!((m.tabs.len(), m.active), (4, 3));
        m.dispatch(Command::CloseActiveTab);
        assert_eq!((m.tabs.len(), m.active), (3, 2));
        m.dispatch(Command::CloseTab(0));
        assert_eq!(m.tabs.len(), 3, "Home cannot be closed");
        m.dispatch(Command::ReopenClosedTab);
        assert_eq!(m.tabs.len(), 4);
        assert_eq!(m.tabs[3].title, "c.md");
        m.dispatch(Command::NextTab);
        assert_eq!(m.active, 0);
        m.dispatch(Command::PreviousTab);
        assert_eq!(m.active, 3);
    }

    #[test]
    fn a_missing_file_is_reported_and_forgotten() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let p = tmp.path().join("docs/gone.md");
        m.state.library.touch(&p, 1);
        m.dispatch(Command::OpenPath(p.clone()));
        assert_eq!(m.tabs.len(), 1);
        assert!(m.toast.as_ref().is_some_and(|t| t.error));
        assert!(m.state.library.opened_at(&p).is_none());
    }

    #[test]
    fn recent_and_starred_survive_a_restart() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("docs/keep.md");
        {
            let mut m = model(tmp.path());
            fs::write(&p, "# keep").unwrap();
            m.dispatch(Command::OpenPath(p.clone()));
            m.dispatch(Command::ToggleStar(p.clone()));
        }
        // a second launch: same config folder, new model
        let store = Store::at(tmp.path().join("cfg"));
        let (catalog, _) = Catalog::load(None);
        let m = AppModel::load(store, catalog, "en".into());
        assert!(m.state.library.opened_at(&p).is_some());
        assert!(m.state.library.is_starred(&p));
    }

    #[test]
    fn filters_search_and_sort_the_table() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let d = tmp.path().join("docs");
        fs::write(d.join("beta.md"), "bb").unwrap();
        fs::write(d.join("alpha.md"), "a").unwrap();
        fs::write(d.join("zeta.pdf"), "pdf").unwrap();
        assert_eq!(m.rows().len(), 3);
        m.filter = Some(DocGroup::Markdown);
        assert_eq!(m.rows().len(), 2);
        m.filter = None;
        m.search = "ZET".into();
        assert_eq!(m.rows()[0].entry.name, "zeta.pdf");
        m.search.clear();
        m.sort = Sort { key: SortKey::Name, ascending: true };
        assert_eq!(m.rows().iter().map(|r| r.entry.name.as_str()).collect::<Vec<_>>(), ["alpha.md", "beta.md", "zeta.pdf"]);
        m.sort_by(SortKey::Name);
        assert_eq!(m.rows()[0].entry.name, "zeta.pdf");
        m.sort_by(SortKey::Size);
        assert_eq!(m.rows()[0].entry.name, "zeta.pdf", "a new column starts with the biggest");
        m.sort_by(SortKey::Size);
        assert_eq!(m.rows()[0].entry.name, "alpha.md");
    }

    #[test]
    fn starred_and_recent_pages_list_their_files() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let a = tmp.path().join("docs/a.md");
        let b = tmp.path().join("docs/b.md");
        fs::write(&a, "a").unwrap();
        fs::write(&b, "b").unwrap();
        m.dispatch(Command::ToggleStar(a.clone()));
        m.dispatch(Command::OpenPath(b.clone()));
        m.dispatch(Command::ShowPage(Page::Starred));
        assert_eq!(m.rows().iter().map(|r| r.entry.path.clone()).collect::<Vec<_>>(), vec![a.clone()]);
        m.dispatch(Command::ShowPage(Page::Recent));
        assert_eq!(m.rows().iter().map(|r| r.entry.path.clone()).collect::<Vec<_>>(), vec![b]);
        assert!(m.is_home(), "pages are shown on the Home tab");
    }

    #[test]
    fn rename_and_new_folder_go_through_the_prompt() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let a = tmp.path().join("docs/a.md");
        fs::write(&a, "a").unwrap();
        m.dispatch(Command::OpenPath(a.clone()));
        m.dispatch(Command::RenameFile(a.clone()));
        m.edit_prompt("b/c".into());
        m.submit_modal();
        assert!(matches!(&m.modal, Some(Modal::Prompt { error: Some(_), .. })), "bad names stay in the prompt");
        m.edit_prompt("minutes".into());
        m.submit_modal();
        assert!(m.modal.is_none());
        let b = tmp.path().join("docs/minutes.md");
        assert!(b.is_file() && !a.exists());
        assert_eq!(m.tabs[1].path.as_deref(), Some(b.as_path()));
        assert!(m.state.library.opened_at(&b).is_some());

        let docs = tmp.path().join("docs");
        m.dispatch(Command::NewFolder(docs.clone()));
        m.edit_prompt("Clients".into());
        m.submit_modal();
        assert!(docs.join("Clients").is_dir());
        assert!(m.is_expanded(&docs));
    }

    #[test]
    fn deleting_asks_first_then_the_platform_trashes() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let a = tmp.path().join("docs/a.md");
        fs::write(&a, "a").unwrap();
        m.dispatch(Command::OpenPath(a.clone()));
        m.dispatch(Command::ToggleStar(a.clone()));
        assert!(m.dispatch(Command::DeleteFile(a.clone())).is_empty());
        assert!(matches!(m.modal, Some(Modal::Confirm { kind: ConfirmKind::DeleteFile, .. })));
        assert_eq!(m.submit_modal(), vec![Effect::Trash(a.clone())]);
        m.trashed(&a, Ok(()));
        assert_eq!(m.tabs.len(), 1);
        assert!(m.state.library.opened_at(&a).is_none() && !m.state.library.is_starred(&a));
        m.trashed(&a, Err("no trash".into()));
        assert!(m.toast.as_ref().is_some_and(|t| t.error));
    }

    #[test]
    fn pinned_folders_and_the_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let other = tmp.path().join("other");
        fs::create_dir_all(&other).unwrap();
        m.folder_picked(other.clone());
        assert_eq!(m.page, Page::Folder);
        assert_eq!(m.folder_roots(), vec![tmp.path().join("docs"), other.clone()]);
        m.folder_picked(other.clone());
        assert_eq!(m.state.settings.folders.len(), 1, "no duplicates");
        m.dispatch(Command::ToggleFolder(other.clone()));
        assert!(m.is_expanded(&other));
        m.dispatch(Command::ToggleFolder(other.clone()));
        assert!(!m.is_expanded(&other));
        m.dispatch(Command::UnpinFolder(other));
        assert_eq!(m.page, Page::Home);
        assert_eq!(m.folder_roots().len(), 1);
    }

    fn choose_language(m: &mut AppModel, tag: &str) {
        m.open_settings(SettingsSection::General);
        m.draft.language = tag.to_string();
        assert!(m.apply_settings());
        m.close_settings();
    }

    #[test]
    fn the_window_menu_lists_home_tabs_and_windows() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let doc = tmp.path().join("a.md");
        fs::write(&doc, "# a").unwrap();
        let v = m.menu_view();
        assert!(!v.document, "home: Save, Save As and Print are off");
        assert_eq!(v.windows.len(), 1);
        assert_eq!(v.windows[0].title, "Glyphra");
        assert!(v.windows[0].checked);
        m.dispatch(Command::OpenPath(doc));
        m.open_windows.push(tmp.path().join("w.md"));
        let v = m.menu_view();
        assert!(v.document);
        let titles: Vec<_> = v.windows.iter().map(|w| (w.title.as_str(), w.checked)).collect();
        assert_eq!(titles, [("Glyphra", false), ("a.md", true), ("w.md", false)]);
        assert_eq!(m.dispatch(Command::ActivateTab(0)), vec![Effect::FocusMainWindow]);
        assert!(!m.menu_view().document);
    }

    #[test]
    fn settings_apply_only_when_changed_and_close_discards() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        m.open_settings(SettingsSection::General);
        assert!(!m.settings_dirty(), "nothing changed: Apply stays grey");
        m.draft.theme = ThemePref::Dark;
        m.draft.open_files_in = OpenIn::Window;
        assert!(m.settings_dirty());
        assert_eq!(m.state.settings.theme, ThemePref::System, "not real before Apply");
        m.close_settings();
        assert!(m.settings.is_none());
        assert_eq!(m.state.settings.theme, ThemePref::System, "Close throws the draft away");
        m.open_settings(SettingsSection::General);
        assert!(!m.settings_dirty(), "a new draft starts clean");
        m.draft.theme = ThemePref::Dark;
        m.draft.open_files_in = OpenIn::Window;
        assert!(m.apply_settings());
        assert!(!m.settings_dirty());
        assert_eq!(m.state.settings.theme, ThemePref::Dark);
        assert_eq!(m.state.settings.open_modules_in, OpenIn::Window);
        let (again, _) = Store::at(tmp.path().join("cfg")).load_or_init();
        assert_eq!(again.settings.open_files_in, OpenIn::Window);
    }

    #[test]
    fn the_documents_folder_can_be_changed() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let new = tmp.path().join("elsewhere/notes");
        m.open_settings(SettingsSection::General);
        m.draft_documents_dir(new.clone());
        assert!(m.settings_dirty());
        assert!(m.apply_settings());
        assert!(new.is_dir(), "the folder is created");
        assert_eq!(m.state.settings.documents_dir, new);
        // new documents land there
        m.close_settings();
        m.dispatch(Command::NewDocument(ModuleKind::Markdown));
        assert!(m.tabs.last().and_then(|t| t.path.as_deref()).is_some_and(|p| p.starts_with(&new)));
        // a folder that cannot be made keeps the dialog's draft and reports an error
        m.open_settings(SettingsSection::General);
        let file = tmp.path().join("a-file");
        fs::write(&file, "x").unwrap();
        m.draft_documents_dir(file.join("sub"));
        assert!(!m.apply_settings());
        assert!(m.toast.as_ref().is_some_and(|t| t.error));
        assert_eq!(m.state.settings.documents_dir, new);
    }

    #[test]
    fn the_settings_dialog_blocks_everything_else() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        m.dispatch(Command::OpenSettings(SettingsSection::General));
        assert!(m.dispatch(Command::OpenDialog).is_empty());
        assert!(m.dispatch(Command::NewDocument(ModuleKind::Markdown)).is_empty());
        m.dispatch(Command::NewDocumentPicker);
        assert_eq!(m.page, Page::Home);
        assert_eq!(m.tabs.len(), 1);
        assert_eq!(m.dispatch(Command::Quit), vec![Effect::Quit]);
        m.dispatch(Command::OpenSettings(SettingsSection::About));
        assert_eq!(m.settings, Some(SettingsSection::About));
        m.close_settings();
        assert_eq!(m.dispatch(Command::OpenDialog), vec![Effect::OpenFileDialog]);
    }

    #[test]
    fn documents_open_in_a_window_when_chosen() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let doc = tmp.path().join("a.md");
        fs::write(&doc, "# a").unwrap();
        assert!(m.dispatch(Command::OpenPath(doc.clone())).is_empty(), "tabs by default");
        assert_eq!(m.tabs.len(), 2);
        m.state.settings.open_files_in = OpenIn::Window;
        let b = tmp.path().join("b.md");
        fs::write(&b, "# b").unwrap();
        assert_eq!(m.dispatch(Command::OpenPath(b.clone())), vec![Effect::OpenInWindow(b.clone())]);
        assert_eq!(m.tabs.len(), 2, "no tab for it");
        // a document window itself always uses its own tab
        m.detached = true;
        assert!(m.dispatch(Command::OpenPath(b)).is_empty());
        assert_eq!(m.tabs.len(), 3);
    }

    #[test]
    fn theme_and_language_are_saved() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        m.dispatch(Command::CycleTheme);
        assert_eq!(m.state.settings.theme, ThemePref::Light);
        m.dispatch(Command::SetTheme(ThemePref::Dark));
        choose_language(&mut m, "nl");
        assert_eq!(m.t("menu-file"), "Bestand");
        let (again, _) = Store::at(tmp.path().join("cfg")).load_or_init();
        assert_eq!(again.settings.theme, ThemePref::Dark);
        assert_eq!(again.settings.language, "nl");
    }

    #[test]
    fn relative_times_are_translated() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        assert_eq!(m.when_at(1000, 1030), "just now");
        assert_eq!(m.when_at(1000, 1000 + 5 * 60), "5 min ago");
        assert_eq!(m.when_at(0, 5), "—");
        choose_language(&mut m, "fr");
        assert_eq!(m.when_at(1000, 1000 + 86_400), "hier");
        choose_language(&mut m, "de");
        assert_eq!(m.when_at(1000, 1000 + 3 * 86_400), "vor 3 Tagen");
    }

    #[test]
    fn effects_are_requested_not_performed() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        assert_eq!(m.dispatch(Command::Quit), vec![Effect::Quit]);
        assert_eq!(m.dispatch(Command::OpenDialog), vec![Effect::OpenFileDialog]);
        assert_eq!(m.dispatch(Command::CopyPath("/x/y.md".into())), vec![Effect::CopyText("/x/y.md".into())]);
        assert_eq!(m.dispatch(Command::FocusSearch), vec![Effect::FocusSearch]);
        // a Crafting App file is handed to the app, not previewed
        assert!(matches!(m.dispatch(Command::OpenPath("/x/part.step".into()))[..], [Effect::LaunchCraft { .. }]));
    }

    // ------------------------------------------------------------------ language coverage

    /// Every Fluent id the code asks for exists in every built-in language.
    #[test]
    fn every_id_in_the_code_exists_in_every_language() {
        let (catalog, _) = Catalog::load(None);
        let mut ids = std::collections::BTreeSet::new();
        // 1. ids written in the source as t("…"), t_args("…"), notify("…") and so on
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for entry in fs::read_dir(src).unwrap().flatten() {
            let text = fs::read_to_string(entry.path()).unwrap();
            for call in ["t(\"", "t_args(\"", "notify(\"", "notify_error(\"", "say_id(\"", "fail(\"", "title: \"", "sub: \""] {
                for part in text.split(call).skip(1) {
                    let id: String = part.chars().take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-').collect();
                    if id.contains('-') && part[id.len()..].starts_with('"') {
                        ids.insert(id);
                    }
                }
            }
        }
        // 2. ids built from data: modules, groups, sections, commands, menus
        for k in ModuleKind::ALL {
            ids.insert(format!("module-{}", k.id()));
            ids.insert(format!("module-{}-tagline", k.id()));
        }
        for g in DocGroup::ALL {
            ids.insert(format!("group-{}", g.id()));
        }
        for s in SettingsSection::ALL {
            ids.insert(format!("settings-{}", s.id()));
        }
        for (c, _) in kd_commands::bindable_commands() {
            ids.insert(c.label_key());
        }
        for mac in [true, false] {
            for menu in menu_bar(mac) {
                collect_menu_ids(&menu, &mut ids);
            }
        }
        assert!(ids.len() > 150, "found only {} ids", ids.len());
        for l in catalog.locales() {
            let i = I18n::new(&catalog, &l.tag, "");
            let missing: Vec<_> = ids.iter().filter(|id| !i.has(id)).collect();
            assert!(missing.is_empty(), "{} lacks {missing:?}", l.tag);
        }
    }

    fn collect_menu_ids(menu: &kd_commands::menu::Menu, ids: &mut std::collections::BTreeSet<String>) {
        use kd_commands::menu::Entry;
        ids.insert(menu.label_key.to_string());
        for e in &menu.entries {
            match e {
                Entry::Command(c) => {
                    ids.insert(c.label_key());
                }
                Entry::Labelled(k, _) => {
                    ids.insert(k.to_string());
                }
                Entry::Native(n) => {
                    ids.insert(n.label_key().to_string());
                }
                Entry::Submenu(m) => collect_menu_ids(m, ids),
                Entry::Separator => {}
            }
        }
    }

    // ------------------------------------------------------------------ Glyphra AI

    /// A one-request OpenAI-style server that streams "Hello there" and then stops.
    fn mock_ai_server() -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut s) = stream else { break };
                std::thread::spawn(move || {
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 4096];
                    // read the headers and the body
                    loop {
                        let n = s.read(&mut chunk).unwrap_or(0);
                        if n == 0 {
                            return;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                        let text = String::from_utf8_lossy(&buf).to_string();
                        if let Some(h) = text.find("\r\n\r\n") {
                            let len = text[..h]
                                .lines()
                                .find_map(|l| {
                                    l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap_or(0))
                                })
                                .unwrap_or(0);
                            if buf.len() >= h + 4 + len {
                                break;
                            }
                        }
                    }
                    let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\" there\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
                    let _ = write!(
                        s,
                        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    );
                });
            }
        });
        url
    }

    /// Set up a local provider through the settings dialog, as the user does.
    fn with_local_ai(m: &mut AppModel, url: &str) {
        m.open_settings(SettingsSection::AiProviders);
        m.ai.draft.provider_mut("lmstudio").unwrap().base_url = Some(url.into());
        m.ai.draft.provider_mut("lmstudio").unwrap().default_model = Some("local-model".into());
        m.ai.set_default_model("lmstudio:local-model");
        m.ai.draft.max_retries = 0; // a failing server fails at once
        assert!(m.settings_dirty(), "a change in Settings › AI makes Apply useful");
        assert!(m.apply_settings());
        m.close_settings();
        assert!(!m.settings_dirty());
    }

    fn wait_for_ai(m: &mut AppModel) {
        for _ in 0..200 {
            m.ai.pump();
            if !m.ai.busy() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!("the AI request did not finish");
    }

    #[test]
    fn the_ai_chat_streams_an_answer() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let url = mock_ai_server();
        assert!(!m.ai.ready() || std::env::vars().any(|(k, _)| k.ends_with("_API_KEY")));
        with_local_ai(&mut m, &url);
        assert!(m.ai.ready());
        m.dispatch(Command::ShowPage(Page::AiChat));
        assert_eq!(m.page, Page::AiChat);
        m.dispatch(Command::AiSend("Say hello".into()));
        assert!(m.ai.chat.running());
        wait_for_ai(&mut m);
        let msgs = &m.ai.chat.messages;
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].text, "Say hello");
        assert_eq!(msgs[1].text, "Hello there");
        assert!(!msgs[1].error);
        m.dispatch(Command::AiNewChat);
        assert!(m.ai.chat.messages.is_empty());
    }

    #[test]
    fn a_failing_provider_shows_an_error_in_the_chat() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        // nothing listens on this port
        with_local_ai(&mut m, "http://127.0.0.1:9/v1");
        m.dispatch(Command::AiSend("hi".into()));
        wait_for_ai(&mut m);
        let last = m.ai.chat.messages.last().unwrap();
        assert!(last.error && !last.text.is_empty(), "{last:?}");
    }

    #[test]
    fn an_ai_tool_runs_and_its_result_becomes_a_document() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        let url = mock_ai_server();
        with_local_ai(&mut m, &url);
        m.dispatch(Command::AiOpenTool("md.improve".into()));
        assert_eq!(m.page, Page::AiTools);
        assert!(m.ai.tool.is_some());
        m.dispatch(Command::AiRunTool { input: "  ".into(), note: String::new() });
        assert!(!m.ai.tool.as_ref().unwrap().running(), "no text, no request");
        m.dispatch(Command::AiRunTool { input: "teh text".into(), note: "shorter".into() });
        assert!(m.ai.tool.as_ref().unwrap().running());
        wait_for_ai(&mut m);
        assert_eq!(m.ai.tool.as_ref().unwrap().output, "Hello there");
        let before = m.tabs.len();
        m.dispatch(Command::AiSaveToolResult);
        assert_eq!(m.tabs.len(), before + 1, "the result opens as a document");
        let saved: Vec<_> = fs::read_dir(tmp.path().join("docs")).unwrap().flatten().collect();
        assert_eq!(saved.len(), 1);
        assert_eq!(fs::read_to_string(saved[0].path()).unwrap(), "Hello there");
        m.dispatch(Command::AiCloseTool);
        assert!(m.ai.tool.is_none());
    }

    #[test]
    fn the_ai_settings_wait_for_apply_and_close_drops_them() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        m.open_settings(SettingsSection::AiProviders);
        m.ai.draft.provider_mut("anthropic").unwrap().api_key = Some("sk-ant-x".into());
        assert!(m.settings_dirty());
        assert!(m.ai.hub.config().provider("anthropic").unwrap().api_key.is_none(), "nothing is in force before Apply");
        m.close_settings();
        assert!(!m.settings_dirty());
        assert!(m.ai.draft.provider("anthropic").unwrap().api_key.is_none(), "Close drops the draft");
        let ai_toml = tmp.path().join("cfg").join("ai.toml");
        m.open_settings(SettingsSection::AiProviders);
        m.ai.draft.provider_mut("anthropic").unwrap().api_key = Some("sk-ant-x".into());
        assert!(m.apply_settings());
        assert!(m.ai.ready());
        let text = fs::read_to_string(ai_toml).unwrap();
        assert!(!text.contains("sk-ant-x"), "the key is not in the file");
    }

    #[test]
    fn the_settings_dialog_blocks_the_ai_commands_too() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = model(tmp.path());
        m.open_settings(SettingsSection::General);
        m.dispatch(Command::AiToggleStarred);
        assert!(!m.ai.chat.use_starred);
        m.close_settings();
        m.dispatch(Command::AiToggleStarred);
        assert!(m.ai.chat.use_starred);
    }

    #[test]
    fn every_string_the_ai_screens_use_exists_in_every_language() {
        use std::collections::BTreeSet;
        let mut ids: BTreeSet<String> = BTreeSet::new();
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for f in ["ai_chat.rs", "ai_tools.rs", "settings.rs", "settings_ai.rs", "sidebar.rs"] {
            let text = fs::read_to_string(dir.join(f)).unwrap();
            for part in text.split("t(\"").skip(1) {
                if let Some(id) = part.split('"').next()
                    && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                    && !id.is_empty()
                {
                    ids.insert(id.to_string());
                }
            }
            for part in text.split("(\"").skip(1) {
                if let Some(id) = part.split('"').next()
                    && (id.starts_with("chip-") || id.starts_with("ai-"))
                    && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                {
                    ids.insert(id.to_string());
                }
            }
        }
        for t in kd_ai::tasks::TASKS {
            let k = format!("tool-{}", t.id.replace('.', "-"));
            ids.insert(format!("{k}-desc"));
            ids.insert(k);
        }
        for s in kd_ai::Scope::ALL {
            ids.insert(format!("ai-scope-{}", s.id()));
        }
        for r in ["eu", "apac", "us", "local"] {
            ids.insert(format!("ai-region-{r}"));
        }
        for (id, _, _) in kd_ai::ProviderKind::ALL.iter().map(|k| crate::ai::status(&kd_ai::ProviderConfig::new(*k))) {
            ids.insert(id.to_string());
        }
        for g in ["markdown", "pdf", "drawing"] {
            ids.insert(format!("ai-group-{g}"));
        }
        assert!(ids.len() > 80, "found only {} ids", ids.len());
        let (catalog, _) = Catalog::load(None);
        for l in catalog.locales() {
            let i = I18n::new(&catalog, &l.tag, "");
            let missing: Vec<_> = ids.iter().filter(|id| !i.has(id)).collect();
            assert!(missing.is_empty(), "{} lacks {missing:?}", l.tag);
        }
    }
}
