//! The application model: which page is showing, the open tabs, the Templates filter, the
//! settings draft and the AI model profiles. Actions arrive as [`Command`]s.

use crate::i18n::{t, tr, tx};
use crate::providers::{self, Protocol, Region};
use gl_store::secrets::{self, Vault};
use gl_store::settings::DEFAULT_MCP_CONFIG;
use gl_store::{
    Appearance, CustomProvider, Library, ModelProfile, OpenIn, Settings, SpellingDetection, Store, ThemeFamily, atomic_write,
    mcp_server_names, now, unique_path, validate_mcp,
};
use gl_templates::{self as tpl, Filter, Kind, Template};
use std::path::{Path, PathBuf};

/// Templates shown per page on the Templates page.
/// Templates per page until the size of the window is known.
pub const TEMPLATES_PER_PAGE: usize = 10;

/// The grid of templates: the narrowest card, the space between cards and rows, the shape of a
/// thumbnail (width / height) and the height of a card below its thumbnail (gap + name).
const CARD_MIN: f32 = 150.0;
const CARD_GAP: f32 = 18.0;
const ROW_GAP: f32 = 22.0;
const THUMB_RATIO: f32 = 0.707;
const CARD_FOOT: f32 = 30.0;
/// The page bar below the grid, with its margin.
const PAGER: f32 = 66.0;

/// How many templates make complete rows that all show at once in a grid `width` wide, with
/// `height` left for the cards. At least one row, so a tiny window still shows something.
pub fn templates_that_fit(width: f32, height: f32) -> usize {
    let columns = (((width + CARD_GAP) / (CARD_MIN + CARD_GAP)).floor().max(1.0)) as usize;
    let card_w = (width - CARD_GAP * (columns as f32 - 1.0)) / columns as f32;
    let row = card_w / THUMB_RATIO + CARD_FOOT + ROW_GAP;
    let rows = (((height - PAGER + ROW_GAP) / row).floor().max(1.0)) as usize;
    columns * rows
}
/// Files this app opens in version 0.1.0 (the editor comes in 0.2.0).
pub const OPENABLE: [&str; 3] = ["md", "markdown", "txt"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Open,
    Templates,
    Collaboration,
    AiTools,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Region,
    AiModels,
    ModelAssignment,
    McpServers,
}

impl SettingsSection {
    pub const ALL: [SettingsSection; 5] = [
        SettingsSection::General,
        SettingsSection::Region,
        SettingsSection::AiModels,
        SettingsSection::ModelAssignment,
        SettingsSection::McpServers,
    ];
    pub fn label(self) -> &'static str {
        match self {
            SettingsSection::General => tx("General"),
            SettingsSection::Region => tx("Region"),
            SettingsSection::AiModels => tx("AI Models"),
            SettingsSection::ModelAssignment => tx("Model Assignment"),
            SettingsSection::McpServers => tx("MCP Servers"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    ShowPage(Page),
    OpenSettings(SettingsSection),
    ToggleAiPanel,
    NewDocument(Kind),
    NewFromTemplate(String),
    /// Show the file dialog, starting in this folder (or the documents folder).
    OpenDialog(Option<PathBuf>),
    OpenPath(PathBuf),
    /// Attach the file to AI Chat and show the AI panel (Home: "Open in Chat").
    OpenInChat(PathBuf),
    /// Take a file off the AI Chat's attachments.
    DetachFromChat(PathBuf),
    /// Home: show the file in the file manager.
    RevealPath(PathBuf),
    /// Home: pin or unpin a file in the list.
    TogglePin(PathBuf),
    /// Home: take a file off the list (the file itself stays on disk).
    ForgetRecent(PathBuf),
    /// Home: remove everything from the list whose file no longer exists.
    CleanupRecent,
    /// Home: remove everything from the list.
    ClearRecent,
    ActivateTab(usize),
    CloseTab(usize),
    CloseActiveTab,
    /// Use this model for AI Chat. The provider must be connected; its key is not asked again.
    UseModel {
        provider: String,
        model: String,
    },
    /// Use the next cheaper or more expensive model of the provider AI Chat is using now.
    StepModel(Step),
    /// Ask the provider (`Some(id)`) or every connected provider (`None`) which models it offers.
    /// Asks again only when the address or key changed, or after ten minutes.
    FetchModels(Option<String>),
    /// The "Load models" button: ask this provider now, whatever was asked before.
    ReloadModels(String),
    /// The answer to `Effect::FetchModels`: the text of the answer, or why it failed.
    ModelsFetched {
        provider: String,
        result: Result<String, String>,
    },
    Quit,
}

/// Which way to move in a provider's price list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Cheaper,
    Pricier,
}

/// A model in the lists of Settings and AI Chat.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    /// "$3 / $15" or a note; empty when the price is not known.
    pub price_short: String,
    /// "$3 in / $15 out per 1M tokens", or what is known; empty when unknown.
    pub price_label: String,
    pub chat: bool,
    pub cost: Option<f32>,
}

/// One entry of the model switcher: a model of a connected provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelChoice {
    pub provider: String,
    pub provider_name: String,
    pub model: String,
    /// The model's name; the price is shown in the list only, never once chosen.
    pub name: String,
    /// "$3 / $15" (per 1M tokens in / out), or empty.
    pub price: String,
    pub current: bool,
}

/// What only the platform layer can do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Send this request to a provider and answer with `Command::ModelsFetched`.
    FetchModels {
        provider: String,
        request: crate::models_api::Request,
    },
    OpenFileDialog {
        start: PathBuf,
    },
    /// Show this file (or folder) in the file manager.
    Reveal(PathBuf),
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub kind: NoticeKind,
    pub text: String,
}

/// An open document (read-only until the editor arrives).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
}

/// The fields of Settings › General that wait for Apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralDraft {
    pub user_name: String,
    pub language: String,
    pub appearance: Appearance,
    pub light_theme: ThemeFamily,
    pub dark_theme: ThemeFamily,
    pub open_file_in: OpenIn,
    pub spelling: SpellingDetection,
    pub gpu_acceleration: bool,
    pub ai_enabled: bool,
}

impl GeneralDraft {
    fn from(s: &Settings) -> Self {
        Self {
            user_name: s.user_name.clone(),
            language: s.language.clone(),
            appearance: s.appearance,
            light_theme: s.light_theme,
            dark_theme: s.dark_theme,
            open_file_in: s.open_file_in,
            spelling: s.spelling,
            gpu_acceleration: s.gpu_acceleration,
            ai_enabled: s.ai_enabled,
        }
    }
}

/// The "Connect new AI Model" form.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AddModelForm {
    /// Catalogue id, or `custom:<name>`.
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub profile_name: String,
    /// The provider is connected already: its saved key is used, so none is asked for.
    pub connected: bool,
    /// The id of the connected model being edited; `None` when a new one is added.
    pub editing: Option<String>,
}

/// Where a provider's API key is kept (one key for all of its models).
fn provider_key_name(provider: &str) -> String {
    format!("ai.provider.{provider}")
}

impl AddModelForm {
    fn needs_key(&self) -> bool {
        !self.connected && providers::get(&self.provider).is_none_or(|p| p.needs_key)
    }

    /// All required fields are filled in and the URL looks like one.
    pub fn valid(&self) -> bool {
        !self.provider.is_empty()
            && !self.model.trim().is_empty()
            && (self.base_url.starts_with("http://") || self.base_url.starts_with("https://"))
            && (!self.needs_key() || !self.api_key.trim().is_empty())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomProviderForm {
    pub name: String,
    pub base_url: String,
    /// A [`Region`] id; empty means local.
    pub region: String,
    /// A [`Protocol`] id; empty means OpenAI Chat Completions.
    pub protocol: String,
}

/// The MCP servers JSON editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpEditor {
    pub text: String,
    pub error: Option<String>,
}

/// The Templates page: tab, search, filters and page number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatesState {
    pub filter: Filter,
    pub page: usize,
    /// How many fit the window (see `templates_that_fit`).
    pub per_page: usize,
}

/// One row of the Home (recent files) or Open (recent folders) table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub stem: String,
    /// With its dot, e.g. ".md"; empty for folders.
    pub ext: String,
    pub location: String,
    pub when: String,
    pub path: PathBuf,
    /// The file is still on disk (rows of missing files are shown light).
    pub exists: bool,
    pub pinned: bool,
}

/// The templates to show and how many pages there are.
pub struct TemplateView {
    pub items: Vec<&'static Template>,
    pub page: usize,
    pub pages: usize,
    pub total: usize,
}

pub struct AppModel {
    store: Store,
    vault: Vault,
    /// Keys that could not be saved: kept until Glyphra closes.
    memory_secrets: std::collections::HashMap<String, String>,
    pub settings: Settings,
    pub library: Library,
    pub page: Page,
    pub tabs: Vec<Tab>,
    /// Files attached to the AI Chat (shown above the message box).
    pub chat_files: Vec<PathBuf>,
    /// `Some(i)`: tab `i` is in front instead of a page.
    pub active_tab: Option<usize>,
    pub section: SettingsSection,
    pub draft: GeneralDraft,
    pub templates: TemplatesState,
    pub add_model: Option<AddModelForm>,
    pub custom_form: Option<CustomProviderForm>,
    pub mcp: Option<McpEditor>,
    pub notice: Option<Notice>,
    /// What each provider last said it offers (kept in `models.json`), by provider id.
    received: std::collections::BTreeMap<String, crate::models_api::Received>,
    /// Providers being asked right now.
    models_busy: std::collections::HashSet<String>,
    /// Why the last question to a provider failed (cleared by the next good answer).
    models_failed: std::collections::HashMap<String, String>,
    /// When and with what address/key a provider was last asked.
    models_checked: std::collections::HashMap<String, (std::time::Instant, u64)>,
    /// Bumped whenever the model itself changes the text of a form field (a reset, a provider
    /// choice, a form opening). Text fields use it as their key, so they are rebuilt with the new
    /// text; while the user types they are left alone (a field the app re-fills on every
    /// keystroke would lose characters typed faster than the round trip).
    pub form_gen: u32,
}

/// Choose the interface language at start: the saved one, or on the very first run the language
/// of the operating system (when supported), which is then saved.
pub fn init_language(store: &Store) {
    let first_run = !store.dir().join("settings.json").exists();
    let mut settings = store.load_settings();
    if first_run {
        settings.language = crate::i18n::system_language().to_string();
        let _ = store.save_settings(&settings);
    }
    crate::i18n::set_language(&settings.language);
    crate::region::set(&settings.region);
}

impl AppModel {
    pub fn new(store: Store) -> Self {
        let vault = Vault::at(store.dir().join(secrets::KEY_FILE));
        let settings = store.load_settings();
        let dir = store.dir().to_path_buf();
        crate::i18n::set_language(&settings.language);
        let mut library = store.load_library();
        library.prune();
        // The documents folder is where Home starts; make sure it exists and is listed.
        let _ = std::fs::create_dir_all(&settings.documents_dir);
        if library.folders.is_empty() && settings.documents_dir.is_dir() {
            library.touch_folder(&settings.documents_dir, now());
        }
        Self {
            draft: GeneralDraft::from(&settings),
            store,
            vault,
            memory_secrets: Default::default(),
            settings,
            library,
            page: Page::Home,
            tabs: Vec::new(),
            chat_files: Vec::new(),
            active_tab: None,
            section: SettingsSection::General,
            templates: TemplatesState {
                filter: Filter { kind: Some(Kind::Document), ..Default::default() },
                page: 0,
                per_page: TEMPLATES_PER_PAGE,
            },
            add_model: None,
            custom_form: None,
            mcp: None,
            notice: None,
            received: crate::models_api::load_received(&dir),
            models_busy: Default::default(),
            models_failed: Default::default(),
            models_checked: Default::default(),
            form_gen: 0,
        }
    }

    // ------------------------------------------------------------------ commands

    pub fn dispatch(&mut self, cmd: Command) -> Vec<Effect> {
        match cmd {
            Command::ShowPage(p) => self.show_page(p),
            Command::OpenSettings(s) => {
                self.show_page(Page::Settings);
                self.section = s;
            }
            Command::ToggleAiPanel => {
                self.settings.ai_panel_open = !self.settings.ai_panel_open;
                self.save_settings();
            }
            Command::NewDocument(kind) => self.new_document(kind),
            Command::NewFromTemplate(id) => match tpl::get(&id) {
                Some(t) => self.create_and_open(t.name, &tpl::instantiate(t, &self.vars())),
                None => self.error(tr("Unknown template “{id}”.", &[("id", &id)])),
            },
            Command::OpenDialog(start) => {
                let start = start.filter(|d| d.is_dir()).unwrap_or_else(|| self.settings.documents_dir.clone());
                return vec![Effect::OpenFileDialog { start }];
            }
            Command::OpenPath(p) => self.open_path(&p),
            Command::OpenInChat(p) => self.attach_to_chat(p),
            Command::DetachFromChat(p) => self.chat_files.retain(|f| *f != p),
            Command::RevealPath(p) => {
                if p.exists() {
                    return vec![Effect::Reveal(p)];
                }
                match p.parent().filter(|d| d.is_dir()) {
                    Some(d) => return vec![Effect::Reveal(d.to_path_buf())],
                    None => self.error(tr("“{path}” no longer exists.", &[("path", &p.display().to_string())])),
                }
            }
            Command::TogglePin(p) => {
                self.library.toggle_pin(&p);
                self.save_library();
            }
            Command::ForgetRecent(p) => {
                self.library.forget_file(&p);
                self.save_library();
            }
            Command::CleanupRecent => {
                let n = self.library.cleanup_files();
                self.save_library();
                self.notify(
                    NoticeKind::Info,
                    if n == 0 {
                        t("Every file in the list still exists.")
                    } else {
                        tr("Removed {n} files that no longer exist from the list.", &[("n", &n.to_string())])
                    },
                );
            }
            Command::ClearRecent => {
                self.library.clear_files();
                self.save_library();
            }
            Command::ActivateTab(i) => {
                if i < self.tabs.len() {
                    self.active_tab = Some(i);
                }
            }
            Command::CloseTab(i) => self.close_tab(i),
            Command::UseModel { provider, model } => self.use_model(&provider, &model),
            Command::StepModel(step) => self.step_model(step),
            Command::FetchModels(p) => return self.fetch_models(p, false),
            Command::ReloadModels(p) => return self.fetch_models(Some(p), true),
            Command::ModelsFetched { provider, result } => self.models_fetched(&provider, result),
            Command::CloseActiveTab => {
                if let Some(i) = self.active_tab {
                    self.close_tab(i);
                }
            }
            Command::Quit => return vec![Effect::Quit],
        }
        Vec::new()
    }

    fn show_page(&mut self, p: Page) {
        self.form_gen += 1;
        self.active_tab = None;
        if p == Page::Settings && self.page != Page::Settings {
            self.draft = GeneralDraft::from(&self.settings);
        }
        if self.page == Page::Settings && p != Page::Settings {
            self.add_model = None;
            self.custom_form = None;
            self.mcp = None;
        }
        self.page = p;
    }

    /// The AI panel on the right is showing.
    pub fn ai_panel_visible(&self) -> bool {
        self.settings.ai_enabled && self.settings.ai_panel_open && (self.active_tab.is_some() || self.page != Page::Settings)
    }

    // ------------------------------------------------------------------ messages

    // ------------------------------------------------------------------ the models of a provider

    /// How to ask `provider` for its models: the catalogue's endpoint, or for a provider you
    /// added yourself the one of the API it speaks.
    fn models_api_of(&self, provider: &str) -> Option<crate::models_api::ModelsApi> {
        if let Some(p) = providers::get(provider) {
            return p.models_api.clone();
        }
        let c = provider.strip_prefix("custom:").and_then(|n| self.settings.custom_providers.iter().find(|c| c.name == n))?;
        Some(crate::models_api::ModelsApi::for_protocol(Protocol::from_id(&c.protocol).unwrap_or(Protocol::OpenAiChat)))
    }

    /// The address and key to ask with: what is typed in the open form for this provider, else
    /// what was saved when it was connected. `None`: nothing to ask with yet.
    fn models_credentials(&self, provider: &str) -> Option<(String, String)> {
        let saved_key = self.secret(&provider_key_name(provider)).unwrap_or_default();
        let needs_key = providers::get(provider).is_some_and(|p| p.needs_key);
        let (base, key) = match self.add_model.as_ref().filter(|f| f.provider == provider) {
            Some(f) => (f.base_url.trim().to_string(), if f.api_key.trim().is_empty() { saved_key } else { f.api_key.trim().to_string() }),
            None => (self.settings.models.iter().find(|m| m.provider == provider)?.base_url.clone(), saved_key),
        };
        (!base.is_empty() && !(needs_key && key.is_empty())).then_some((base, key))
    }

    fn fetch_models(&mut self, only: Option<String>, force: bool) -> Vec<Effect> {
        const AGAIN: std::time::Duration = std::time::Duration::from_secs(10 * 60);
        let ids: Vec<String> = match only {
            Some(p) => vec![p],
            None => {
                let mut v: Vec<String> = self.settings.models.iter().map(|m| m.provider.clone()).collect();
                v.dedup();
                v.sort();
                v.dedup();
                v
            }
        };
        let mut effects = Vec::new();
        for provider in ids {
            let (Some(api), Some((base, key))) = (self.models_api_of(&provider), self.models_credentials(&provider)) else {
                continue;
            };
            let request = match api.request(&base, &key) {
                Ok(r) => r,
                Err(e) => {
                    self.models_failed.insert(provider, e);
                    continue;
                }
            };
            let fingerprint = {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                (&request.url, &request.headers).hash(&mut h);
                h.finish()
            };
            let recent = !force && self.models_checked.get(&provider).is_some_and(|(at, fp)| *fp == fingerprint && at.elapsed() < AGAIN);
            if recent || self.models_busy.contains(&provider) {
                continue;
            }
            self.models_busy.insert(provider.clone());
            self.models_failed.remove(&provider);
            self.models_checked.insert(provider.clone(), (std::time::Instant::now(), fingerprint));
            effects.push(Effect::FetchModels { provider, request });
        }
        effects
    }

    fn models_fetched(&mut self, provider: &str, result: Result<String, String>) {
        self.models_busy.remove(provider);
        let format = self.models_api_of(provider).map(|a| a.format);
        let parsed = match (result, format) {
            (Ok(body), Some(f)) => crate::models_api::parse(f, &body),
            (Err(e), _) => Err(e),
            (Ok(_), None) => return,
        };
        match parsed {
            Ok(models) => {
                self.models_failed.remove(provider);
                let fetched = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
                self.received.insert(provider.to_string(), crate::models_api::Received { fetched, models });
                if let Ok(text) = serde_json::to_vec_pretty(&self.received) {
                    // not being able to keep the list is no reason to bother you: it is asked again next time
                    let _ = atomic_write(&self.store.dir().join(crate::models_api::FILE), &text);
                }
            }
            Err(e) => {
                // keep asking next time (a wrong key may have been fixed since)
                self.models_checked.remove(provider);
                self.models_failed.insert(provider.to_string(), e);
            }
        }
    }

    /// One line under the model list of a provider: that the models are being loaded, how many
    /// the provider offers, or why the built-in suggestions are shown ("" when nothing was asked).
    pub fn models_note(&self, provider: &str) -> String {
        if self.models_busy.contains(provider) {
            return t("Loading the models from the provider…");
        }
        if let Some(e) = self.models_failed.get(provider) {
            return format!("{} {}", crate::models_api::explain(e), t("Showing the models that came with Glyphra instead."));
        }
        let Some(r) = self.received.get(provider) else {
            return String::new();
        };
        let when = chrono::NaiveDate::parse_from_str(&r.fetched, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|d| crate::region::pattern(&crate::region::current().short_date, d))
            .unwrap_or_else(|| r.fetched.clone());
        if r.models.is_empty() {
            tr("The provider accepted the API key (checked {date}).", &[("date", &when)])
        } else {
            tr("{n} models from the provider (checked {date}).", &[("n", &r.models.len().to_string()), ("date", &when)])
        }
    }

    /// The models of a provider as shown in the lists: what the provider itself offers (once
    /// asked), with the names and prices of the catalogue; otherwise the catalogue's suggestions.
    /// Chat models come first, cheapest first, the ones without a price after them.
    pub fn models_of(&self, provider: &str) -> Vec<ModelEntry> {
        let known = providers::get(provider).map(|p| p.models.as_slice()).unwrap_or_default();
        let entry = |m: &providers::Model| ModelEntry {
            id: m.id.clone(),
            name: m.name.clone(),
            price_short: m.price_short(),
            price_label: m.price_label(),
            chat: m.chat,
            cost: m.cost(),
        };
        let mut out: Vec<ModelEntry> = match self.received.get(provider).filter(|r| !r.models.is_empty()) {
            Some(r) => r
                .models
                .iter()
                .map(|f| match known.iter().find(|m| m.id == f.id) {
                    Some(m) => entry(m),
                    None => ModelEntry {
                        id: f.id.clone(),
                        name: f.name.clone(),
                        price_short: String::new(),
                        price_label: String::new(),
                        chat: true,
                        cost: None,
                    },
                })
                .collect(),
            None => known.iter().map(entry).collect(),
        };
        out.sort_by(|a, b| {
            let rank = |m: &ModelEntry| (!m.chat, m.cost.unwrap_or(f32::MAX));
            let (x, y) = (rank(a), rank(b));
            x.0.cmp(&y.0).then(x.1.total_cmp(&y.1)).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        out
    }

    // ------------------------------------------------------------------ region

    /// Picks a region (`system` or a country code); its formats replace any manual ones.
    pub fn set_region(&mut self, region: &str) {
        self.settings.region.region = region.to_string();
        self.settings.region.custom = crate::region::effective(&gl_store::RegionSettings { manual: false, ..self.settings.region.clone() });
        self.region_changed();
    }

    /// "Change settings manually": starts from the region's formats, off goes back to them.
    pub fn set_region_manual(&mut self, manual: bool) {
        if manual == self.settings.region.manual {
            return;
        }
        let from = gl_store::RegionSettings { manual: false, ..self.settings.region.clone() };
        self.settings.region.custom = crate::region::effective(&from);
        self.settings.region.manual = manual;
        self.region_changed();
    }

    /// Changes one of the manual formats.
    pub fn edit_region(&mut self, edit: impl FnOnce(&mut gl_store::RegionFormat)) {
        if self.settings.region.manual {
            edit(&mut self.settings.region.custom);
            self.region_changed();
        }
    }

    pub fn reset_region(&mut self) {
        self.settings.region = gl_store::RegionSettings::default();
        self.region_changed();
    }

    fn region_changed(&mut self) {
        crate::region::set(&self.settings.region);
        self.save_settings();
    }

    pub fn notify(&mut self, kind: NoticeKind, text: impl Into<String>) {
        self.notice = Some(Notice { kind, text: text.into() });
    }

    fn error(&mut self, text: String) {
        self.notify(NoticeKind::Error, text);
    }

    pub fn dismiss_notice(&mut self) {
        self.notice = None;
    }

    // ------------------------------------------------------------------ files

    fn vars(&self) -> tpl::Vars {
        use chrono::Datelike;
        let d = chrono::Local::now().date_naive();
        let author =
            if self.settings.user_name.trim().is_empty() { "Author".to_string() } else { self.settings.user_name.trim().to_string() };
        tpl::Vars { author, year: i64::from(d.year()), month: d.month(), day: d.day() }
    }

    fn new_document(&mut self, kind: Kind) {
        let vars = self.vars();
        let (base, text) = match kind {
            Kind::Document => ("Untitled document", blank_text(Kind::Document, &vars)),
            Kind::Presentation => ("Untitled presentation", blank_text(Kind::Presentation, &vars)),
        };
        self.create_and_open(base, &text);
    }

    fn create_and_open(&mut self, base: &str, text: &str) {
        let dir = self.settings.documents_dir.clone();
        let path = unique_path(&dir, base, "md");
        match atomic_write(&path, text.as_bytes()) {
            Ok(()) => self.open_path(&path),
            Err(e) => {
                self.error(tr("Could not create “{path}”: {error}", &[("path", &path.display().to_string()), ("error", &e.to_string())]))
            }
        }
    }

    /// Attach a file to AI Chat and make sure the panel is showing.
    fn attach_to_chat(&mut self, path: PathBuf) {
        const MAX_FILES: usize = 10;
        if !self.settings.ai_enabled {
            self.notify(NoticeKind::Info, t("AI is switched off in Settings."));
            return;
        }
        if !path.is_file() {
            self.error(tr("“{path}” no longer exists.", &[("path", &path.display().to_string())]));
            return;
        }
        if !self.chat_files.contains(&path) {
            if self.chat_files.len() >= MAX_FILES {
                self.error(tr("At most {n} files can be attached to a chat.", &[("n", &MAX_FILES.to_string())]));
                return;
            }
            self.chat_files.push(path);
        }
        if !self.settings.ai_panel_open {
            self.settings.ai_panel_open = true;
            self.save_settings();
        }
    }

    fn open_path(&mut self, path: &Path) {
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        if !OPENABLE.contains(&ext.as_str()) {
            self.error(tr("Glyphra {version} opens Markdown files (.md, .markdown, .txt).", &[("version", gl_store::VERSION)]));
            return;
        }
        if let Some(i) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab = Some(i);
            self.library.touch_file(path, now());
            self.save_library();
            return;
        }
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let title = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.tabs.push(Tab { path: path.to_path_buf(), title, content });
                self.active_tab = Some(self.tabs.len() - 1);
                self.library.touch_file(path, now());
                self.save_library();
            }
            Err(e) => {
                self.error(tr("Could not open “{path}”: {error}", &[("path", &path.display().to_string()), ("error", &e.to_string())]));
            }
        }
    }

    fn close_tab(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        self.tabs.remove(i);
        self.active_tab = match self.active_tab {
            _ if self.tabs.is_empty() => None,
            Some(a) if a == i => Some(i.saturating_sub(1).min(self.tabs.len() - 1)),
            Some(a) if a > i => Some(a - 1),
            other => other,
        };
    }

    /// Recent files, pinned ones first, then newest first (Home). Files that are gone are
    /// included, with `exists` false.
    pub fn recent_file_rows(&self, now: u64, max: usize) -> Vec<Row> {
        let mut entries: Vec<&gl_store::RecentEntry> = self.library.files.iter().collect();
        entries.sort_by_key(|e| !e.pinned); // stable: the order of use is kept inside each group
        entries
            .into_iter()
            .take(max)
            .map(|e| {
                let stem = e.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                let ext = e.path.extension().map(|s| format!(".{}", s.to_string_lossy())).unwrap_or_default();
                Row {
                    stem,
                    ext,
                    location: parent_display(&e.path),
                    when: crate::i18n::time_ago(e.opened, now),
                    path: e.path.clone(),
                    exists: e.path.is_file(),
                    pinned: e.pinned,
                }
            })
            .collect()
    }

    /// Recent folders that still exist, newest first (Open).
    pub fn recent_folder_rows(&self, now: u64) -> Vec<Row> {
        self.library
            .folders
            .iter()
            .filter(|e| e.path.is_dir())
            .map(|e| Row {
                stem: e.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| e.path.display().to_string()),
                ext: String::new(),
                location: parent_display(&e.path),
                when: crate::i18n::time_ago(e.opened, now),
                path: e.path.clone(),
                exists: true,
                pinned: false,
            })
            .collect()
    }

    fn save_settings(&mut self) {
        if let Err(e) = self.store.save_settings(&self.settings) {
            self.error(tr("Could not save settings: {error}", &[("error", &e.to_string())]));
        }
    }

    fn save_library(&mut self) {
        if let Err(e) = self.store.save_library(&self.library) {
            self.error(tr("Could not save the recent list: {error}", &[("error", &e.to_string())]));
        }
    }

    // ------------------------------------------------------------------ templates

    pub fn set_templates_kind(&mut self, kind: Kind) {
        self.templates.filter.kind = Some(kind);
        self.templates.page = 0;
    }

    pub fn set_templates_query(&mut self, q: String) {
        self.templates.filter.query = q;
        self.templates.page = 0;
    }

    pub fn set_templates_category(&mut self, v: String) {
        self.templates.filter.category = v;
        self.templates.page = 0;
    }

    pub fn set_templates_purpose(&mut self, v: String) {
        self.templates.filter.purpose = v;
        self.templates.page = 0;
    }

    pub fn set_templates_language(&mut self, v: String) {
        self.templates.filter.language = v;
        self.templates.page = 0;
    }

    pub fn set_templates_page(&mut self, p: usize) {
        self.templates.page = p;
    }

    /// The window changed: the grid is `width` wide and `height` of it is on screen. The first
    /// template that was showing stays on screen.
    pub fn fit_templates(&mut self, width: f32, height: f32) {
        let per = templates_that_fit(width, height);
        if per != self.templates.per_page {
            let first = self.templates.page * self.templates.per_page;
            self.templates.per_page = per;
            self.templates.page = first / per;
        }
    }

    pub fn template_view(&self) -> TemplateView {
        let all = tpl::filter(&self.templates.filter);
        let total = all.len();
        let per = self.templates.per_page.max(1);
        let pages = total.div_ceil(per).max(1);
        let page = self.templates.page.min(pages - 1);
        let items = all.into_iter().skip(page * per).take(per).collect();
        TemplateView { items, page, pages, total }
    }

    // ------------------------------------------------------------------ settings › General

    pub fn general_dirty(&self) -> bool {
        self.draft != GeneralDraft::from(&self.settings)
    }

    /// Put the General page back to its defaults (still needs Apply).
    pub fn reset_general(&mut self) {
        self.draft = GeneralDraft::from(&Settings::default());
        self.form_gen += 1;
    }

    pub fn apply_general(&mut self) {
        let d = self.draft.clone();
        let restart = d.language != self.settings.language
            || d.gpu_acceleration != self.settings.gpu_acceleration
            || d.ai_enabled != self.settings.ai_enabled;
        self.settings.user_name = d.user_name.trim().to_string();
        self.settings.language = d.language;
        crate::i18n::set_language(&self.settings.language);
        self.settings.appearance = d.appearance;
        self.settings.light_theme = d.light_theme;
        self.settings.dark_theme = d.dark_theme;
        self.settings.open_file_in = d.open_file_in;
        self.settings.spelling = d.spelling;
        self.settings.gpu_acceleration = d.gpu_acceleration;
        self.settings.ai_enabled = d.ai_enabled;
        self.draft = GeneralDraft::from(&self.settings);
        self.save_settings();
        if restart {
            self.notify(NoticeKind::Info, t("Saved. Some options apply after you restart Glyphra."));
        }
    }

    // ------------------------------------------------------------------ settings › AI Models

    /// Catalogue providers followed by the user's own: (id, name).
    pub fn provider_options(&self) -> Vec<(String, String)> {
        providers::all()
            .iter()
            .map(|p| (p.id.to_string(), p.name.to_string()))
            .chain(self.settings.custom_providers.iter().map(|c| (format!("custom:{}", c.name), c.name.clone())))
            .collect()
    }

    /// The providers by region, in the order Settings shows them: (region, [(id, name)]). Custom
    /// providers are listed in the region they were given.
    pub fn provider_groups(&self) -> Vec<(&'static str, Vec<(String, String)>)> {
        Region::ALL
            .into_iter()
            .map(|r| {
                let list = providers::in_region(r)
                    .map(|p| (p.id.to_string(), p.name.to_string()))
                    .chain(
                        self.settings
                            .custom_providers
                            .iter()
                            .filter(|c| Region::from_id(&c.region).unwrap_or(Region::Local) == r)
                            .map(|c| (format!("custom:{}", c.name), c.name.clone())),
                    )
                    .collect::<Vec<_>>();
                (r.label(), list)
            })
            .filter(|(_, l)| !l.is_empty())
            .collect()
    }

    /// A line under the provider choice: what it offers and where to get a key.
    pub fn provider_help(&self, id: &str) -> String {
        if let Some(p) = providers::get(id) {
            return format!("{}. {}", p.blurb, p.help);
        }
        id.strip_prefix("custom:")
            .and_then(|n| self.settings.custom_providers.iter().find(|c| c.name == n))
            .map(|c| {
                let proto = Protocol::from_id(&c.protocol).unwrap_or(Protocol::OpenAiChat);
                format!("Your own provider, speaking {}.", proto.label())
            })
            .unwrap_or_default()
    }

    /// The example shown in the Base URL field of the provider's form.
    pub fn endpoint_hint(&self, id: &str) -> String {
        match providers::get(id) {
            Some(p) if p.needs_endpoint() => p.endpoint_hint.to_string(),
            _ => "Enter URL".to_string(),
        }
    }

    /// The model names to choose from for a provider; empty means "type one".
    pub fn provider_models(&self, provider: &str) -> Vec<(String, String)> {
        self.models_of(provider).into_iter().map(|m| (m.id, m.name)).collect()
    }

    /// The same list with prices for the pickers.
    pub fn provider_model_labels(&self, provider: &str) -> Vec<(String, String, String)> {
        self.models_of(provider).into_iter().map(|m| (m.id, m.name, m.price_short)).collect()
    }

    /// What a connected model costs, for the lists ("" when it is not known).
    pub fn model_price(&self, profile: &ModelProfile) -> String {
        self.models_of(&profile.provider).into_iter().find(|m| m.id == profile.model).map(|m| m.price_label).unwrap_or_default()
    }

    fn provider_name(&self, id: &str) -> String {
        self.provider_options().into_iter().find(|(i, _)| i == id).map(|(_, n)| n).unwrap_or_else(|| id.to_string())
    }

    pub fn begin_add_model(&mut self) {
        self.custom_form = None;
        self.form_gen += 1;
        self.add_model = Some(AddModelForm::default());
    }

    pub fn cancel_add_model(&mut self) {
        self.add_model = None;
    }

    /// Choosing a provider fills in its address and clears the model.
    pub fn select_provider(&mut self, id: &str) {
        let url = match providers::get(id) {
            Some(p) => p.base_url.to_string(),
            None => id
                .strip_prefix("custom:")
                .and_then(|n| self.settings.custom_providers.iter().find(|c| c.name == n))
                .map(|c| c.base_url.clone())
                .unwrap_or_default(),
        };
        // a provider that is connected keeps its address and key
        let known = self.settings.models.iter().find(|m| m.provider == id).map(|m| m.base_url.clone());
        self.form_gen += 1;
        if let Some(f) = &mut self.add_model {
            f.provider = id.to_string();
            f.connected = known.is_some();
            f.base_url = known.unwrap_or(url);
            f.model.clear();
            f.api_key.clear();
        }
    }

    /// Opens the form with a connected model: its provider, address, model and name can be changed.
    pub fn begin_edit_model(&mut self, id: &str) {
        let Some(p) = self.settings.model(id).cloned() else {
            return;
        };
        self.custom_form = None;
        self.form_gen += 1;
        self.add_model = Some(AddModelForm {
            connected: self.provider_connected(&p.provider),
            provider: p.provider,
            api_key: String::new(),
            base_url: p.base_url,
            model: p.model,
            profile_name: p.name,
            editing: Some(id.to_string()),
        });
    }

    /// Connect the model in the form. The API key is saved encrypted in the settings (see `secrets::Vault`), never as plain text.
    pub fn submit_add_model(&mut self) {
        let Some(form) = self.add_model.clone().filter(AddModelForm::valid) else {
            return;
        };
        let editing = form.editing.clone().filter(|e| self.settings.model(e).is_some());
        let id = editing.clone().unwrap_or_else(|| self.settings.next_model_id());
        let model_label = self
            .provider_models(&form.provider)
            .into_iter()
            .find(|(i, _)| *i == form.model)
            .map(|(_, n)| n)
            .unwrap_or_else(|| form.model.trim().to_string());
        let name = match form.profile_name.trim() {
            "" => {
                let full = self.provider_name(&form.provider);
                // "Anthropic (Claude)" -> "Anthropic": the profile name stays short
                format!("{} - {}", full.split(" (").next().unwrap_or(&full), model_label)
            }
            n => n.to_string(),
        };
        if !form.api_key.trim().is_empty() {
            self.store_secret(&provider_key_name(&form.provider), form.api_key.trim());
        }
        let profile = ModelProfile {
            id: id.clone(),
            provider: form.provider.clone(),
            base_url: form.base_url.trim().to_string(),
            model: form.model.trim().to_string(),
            name,
        };
        match self.settings.models.iter_mut().find(|m| editing.as_deref() == Some(m.id.as_str())) {
            Some(existing) => {
                let old_provider = std::mem::replace(existing, profile).provider;
                // the key belongs to the provider: it goes when no model uses the provider any more
                if old_provider != form.provider && !self.settings.models.iter().any(|m| m.provider == old_provider) {
                    let key = provider_key_name(&old_provider);
                    self.settings.secrets.remove(&key);
                    self.memory_secrets.remove(&key);
                }
            }
            None => self.settings.models.push(profile),
        }
        if self.settings.default_model.is_none() {
            self.settings.default_model = Some(id);
        }
        self.add_model = None;
        self.save_settings();
    }

    fn store_secret(&mut self, name: &str, value: &str) {
        match self.vault.seal(name, value) {
            Ok(sealed) => {
                self.settings.secrets.insert(name.to_string(), sealed);
                self.memory_secrets.remove(name);
            }
            Err(_) => {
                self.memory_secrets.insert(name.to_string(), value.to_string());
                self.notify(
                    NoticeKind::Info,
                    t("The API key could not be saved (the encryption key file is not writable), so it is kept only until Glyphra closes."),
                );
            }
        }
    }

    /// The saved API key of a model, decrypted from the settings (or from memory when it could not be saved).
    pub fn api_key(&self, model_id: &str) -> Option<String> {
        let profile = self.settings.model(model_id)?;
        self.secret(&provider_key_name(&profile.provider))
    }

    fn secret(&self, name: &str) -> Option<String> {
        self.memory_secrets.get(name).cloned().or_else(|| self.settings.secrets.get(name).and_then(|s| self.vault.open(name, s).ok()))
    }

    /// The provider has been set up: a model of it is connected (and its key saved, if it needs one).
    pub fn provider_connected(&self, provider: &str) -> bool {
        let needs_key = providers::get(provider).is_none_or(|p| p.needs_key);
        self.settings.models.iter().any(|m| m.provider == provider) && (!needs_key || self.secret(&provider_key_name(provider)).is_some())
    }

    /// Models of the connected providers, to switch between without touching the provider or
    /// its key: the catalogue's chat models cheapest first, then models you typed yourself.
    pub fn model_choices(&self) -> Vec<ModelChoice> {
        let current = self.settings.model_for("chat");
        let mut out = Vec::new();
        for (provider, provider_name) in self.provider_options() {
            if !self.provider_connected(&provider) {
                continue;
            }
            let is_current = |model: &str| current.is_some_and(|c| c.provider == provider && c.model == model);
            let known: Vec<_> = self.models_of(&provider).into_iter().filter(|m| m.chat).collect();
            for m in &known {
                out.push(ModelChoice {
                    provider: provider.clone(),
                    provider_name: provider_name.clone(),
                    model: m.id.clone(),
                    name: m.name.clone(),
                    price: m.price_short.clone(),
                    current: is_current(&m.id),
                });
            }
            for p in self.settings.models.iter().filter(|p| p.provider == provider && !known.iter().any(|m| m.id == p.model)) {
                out.push(ModelChoice {
                    provider: provider.clone(),
                    provider_name: provider_name.clone(),
                    model: p.model.clone(),
                    name: p.model.clone(),
                    price: String::new(),
                    current: is_current(&p.model),
                });
            }
        }
        out
    }

    /// Use this model for AI Chat. A model of a connected provider that has no profile yet gets
    /// one (same provider, address and key), so switching never asks for anything.
    pub fn use_model(&mut self, provider: &str, model: &str) {
        if !self.provider_connected(provider) {
            self.error(t("Connect this provider first (Settings › AI Models)."));
            return;
        }
        let existing = self.settings.models.iter().find(|m| m.provider == provider && m.model == model).map(|m| m.id.clone());
        let id = match existing {
            Some(id) => id,
            None => {
                let Some(base_url) = self.settings.models.iter().find(|m| m.provider == provider).map(|m| m.base_url.clone()) else {
                    return;
                };
                let id = self.settings.next_model_id();
                let full = self.provider_name(provider);
                let label = self
                    .provider_models(provider)
                    .into_iter()
                    .find(|(i, _)| i == model)
                    .map(|(_, n)| n)
                    .unwrap_or_else(|| model.to_string());
                let name = format!("{} - {}", full.split(" (").next().unwrap_or(&full), label);
                self.settings.models.push(ModelProfile {
                    id: id.clone(),
                    provider: provider.to_string(),
                    base_url,
                    model: model.to_string(),
                    name,
                });
                id
            }
        };
        self.settings.assignments.insert("chat".to_string(), id);
        self.save_settings();
    }

    /// The next cheaper or more expensive model of the provider AI Chat uses now.
    pub fn step_model(&mut self, step: Step) {
        let Some(current) = self.settings.model_for("chat").cloned() else {
            self.error(t("Connect an AI model first (Settings › AI Models)."));
            return;
        };
        let ladder: Vec<_> = self.models_of(&current.provider).into_iter().filter(|m| m.chat).collect();
        let Some(at) = ladder.iter().position(|m| m.id == current.model) else {
            self.notify(NoticeKind::Info, t("This model has no price list to step through. Pick another one from the list."));
            return;
        };
        let to = match step {
            Step::Cheaper => at.checked_sub(1),
            Step::Pricier => Some(at + 1).filter(|n| *n < ladder.len()),
        };
        match to.and_then(|n| ladder.get(n)) {
            Some(m) => self.use_model(&current.provider, &m.id),
            None => {
                let which = if step == Step::Cheaper { t("cheapest") } else { t("most expensive") };
                self.notify(
                    NoticeKind::Info,
                    tr(
                        "{model} is already the {which} model of {provider}.",
                        &[("model", &current.name), ("which", &which), ("provider", &self.provider_name(&current.provider))],
                    ),
                );
            }
        }
    }

    pub fn delete_model(&mut self, id: &str) {
        let provider = self.settings.model(id).map(|m| m.provider.clone());
        self.settings.remove_model(id);
        // the key belongs to the provider: it goes with its last model
        if let Some(p) = provider.filter(|p| !self.settings.models.iter().any(|m| &m.provider == p)) {
            let name = provider_key_name(&p);
            self.settings.secrets.remove(&name);
            self.memory_secrets.remove(&name);
        }
        self.save_settings();
    }

    pub fn make_default_model(&mut self, id: &str) {
        if self.settings.model(id).is_some() {
            self.settings.default_model = Some(id.to_string());
            self.save_settings();
        }
    }

    pub fn begin_custom_provider(&mut self) {
        self.add_model = None;
        self.form_gen += 1;
        self.custom_form = Some(CustomProviderForm::default());
    }

    pub fn cancel_custom_provider(&mut self) {
        self.custom_form = None;
    }

    /// Add a provider of the user's own.
    pub fn submit_custom_provider(&mut self) {
        let Some(f) = self.custom_form.clone() else {
            return;
        };
        let (name, url) = (f.name.trim().to_string(), f.base_url.trim().to_string());
        if name.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
            self.error(t("A custom provider needs a name and an address starting with http:// or https://."));
            return;
        }
        if self.provider_options().iter().any(|(_, n)| n.eq_ignore_ascii_case(&name)) {
            self.error(tr("There is already a provider called “{name}”.", &[("name", &name)]));
            return;
        }
        let region = Region::from_id(&f.region).unwrap_or(Region::Local).id().to_string();
        let protocol = Protocol::from_id(&f.protocol).unwrap_or(Protocol::OpenAiChat).id().to_string();
        self.settings.custom_providers.push(CustomProvider { name, base_url: url, region, protocol });
        self.custom_form = None;
        self.save_settings();
    }

    pub fn delete_custom_provider(&mut self, name: &str) {
        self.settings.custom_providers.retain(|c| c.name != name);
        self.save_settings();
    }

    // ------------------------------------------------------------------ settings › Model Assignment

    pub fn set_default_model(&mut self, id: Option<String>) {
        self.settings.default_model = id.filter(|i| self.settings.model(i).is_some());
        self.save_settings();
    }

    /// `None` = use the default model for this task.
    pub fn assign_model(&mut self, task: &str, id: Option<String>) {
        match id.filter(|i| self.settings.model(i).is_some()) {
            Some(i) => {
                self.settings.assignments.insert(task.to_string(), i);
            }
            None => {
                self.settings.assignments.remove(task);
            }
        }
        self.save_settings();
    }

    // ------------------------------------------------------------------ settings › MCP Servers

    pub fn open_mcp_editor(&mut self) {
        self.form_gen += 1;
        self.mcp = Some(McpEditor { text: self.settings.mcp_config.clone(), error: None });
    }

    pub fn set_mcp_text(&mut self, text: String) {
        if let Some(e) = &mut self.mcp {
            e.text = text;
            e.error = None;
        }
    }

    pub fn cancel_mcp_editor(&mut self) {
        self.mcp = None;
    }

    pub fn save_mcp(&mut self) {
        let Some(e) = &mut self.mcp else { return };
        match validate_mcp(&e.text) {
            Ok(()) => {
                self.settings.mcp_config = e.text.clone();
                self.mcp = None;
                self.save_settings();
            }
            Err(msg) => e.error = Some(msg),
        }
    }

    /// The names of the configured MCP servers (empty when the JSON is not valid).
    pub fn mcp_server_names(&self) -> Vec<String> {
        mcp_server_names(&self.settings.mcp_config)
    }
}

fn parent_display(path: &Path) -> String {
    path.parent().map(|p| p.display().to_string()).unwrap_or_default()
}

/// The text of a new, empty file of this kind.
pub fn blank_text(kind: Kind, vars: &tpl::Vars) -> String {
    let text = match kind {
        Kind::Document => tpl::get("blank").map(|t| t.source).unwrap_or("# Untitled\n"),
        Kind::Presentation => tpl::get("blank-deck").map(|t| t.source).unwrap_or("# Untitled\n"),
    };
    tpl::fill(text, vars)
}

/// Used by tests and the UI for the "no models" hint.
pub fn default_mcp() -> &'static str {
    DEFAULT_MCP_CONFIG
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        m: AppModel,
        _dir: tempfile::TempDir,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path().join("cfg"));
        let mut s = store.load_settings();
        s.documents_dir = dir.path().join("docs");
        store.save_settings(&s).unwrap();
        Fixture { m: AppModel::new(store), _dir: dir }
    }

    fn fill_form(m: &mut AppModel, provider: &str, model: &str) {
        m.begin_add_model();
        m.select_provider(provider);
        if let Some(f) = &mut m.add_model {
            f.api_key = "sk-test".into();
            f.model = model.into();
        }
    }

    #[test]
    fn starts_on_home_with_the_documents_folder_listed() {
        let f = fixture();
        assert_eq!(f.m.page, Page::Home);
        assert!(f.m.settings.documents_dir.is_dir());
        assert_eq!(f.m.recent_folder_rows(now()).len(), 1);
        assert!(f.m.ai_panel_visible());
    }

    #[test]
    fn navigation_and_the_ai_panel() {
        let mut f = fixture();
        f.m.dispatch(Command::ShowPage(Page::Templates));
        assert_eq!(f.m.page, Page::Templates);
        assert!(f.m.ai_panel_visible());
        f.m.dispatch(Command::OpenSettings(SettingsSection::McpServers));
        assert_eq!((f.m.page, f.m.section), (Page::Settings, SettingsSection::McpServers));
        assert!(!f.m.ai_panel_visible(), "Settings has no AI panel");
        f.m.dispatch(Command::ShowPage(Page::Home));
        f.m.dispatch(Command::ToggleAiPanel);
        assert!(!f.m.ai_panel_visible());
    }

    #[test]
    fn disabled_ai_hides_the_panel() {
        let mut f = fixture();
        f.m.draft.ai_enabled = false;
        f.m.apply_general();
        assert!(!f.m.ai_panel_visible());
        assert!(f.m.notice.is_some(), "restart note");
    }

    #[test]
    fn new_documents_are_created_listed_and_opened() {
        let mut f = fixture();
        for (kind, name) in [(Kind::Document, "Untitled document.md"), (Kind::Presentation, "Untitled presentation.md")] {
            f.m.dispatch(Command::NewDocument(kind));
            assert!(f.m.settings.documents_dir.join(name).is_file(), "{name}");
        }
        f.m.dispatch(Command::NewDocument(Kind::Document));
        assert!(f.m.settings.documents_dir.join("Untitled document 2.md").is_file());
        assert_eq!(f.m.tabs.len(), 3);
        assert_eq!(f.m.active_tab, Some(2));
        let rows = f.m.recent_file_rows(now(), 20);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].ext, ".md");
        assert!(f.m.notice.is_none());
    }

    #[test]
    fn templates_make_a_file_with_their_name() {
        let mut f = fixture();
        f.m.settings.user_name = "Ada".into();
        f.m.dispatch(Command::NewFromTemplate("memo".into()));
        let p = f.m.settings.documents_dir.join("Memo.md");
        let text = std::fs::read_to_string(p).unwrap();
        assert!(text.contains("Ada") && !text.contains("{{"));
        f.m.dispatch(Command::NewFromTemplate("nope".into()));
        assert_eq!(f.m.notice.as_ref().map(|n| n.kind), Some(NoticeKind::Error));
    }

    #[test]
    fn opening_the_same_file_twice_reuses_the_tab_and_closing_moves_focus() {
        let mut f = fixture();
        f.m.dispatch(Command::NewDocument(Kind::Document));
        f.m.dispatch(Command::NewDocument(Kind::Presentation));
        let first = f.m.tabs[0].path.clone();
        f.m.dispatch(Command::OpenPath(first));
        assert_eq!((f.m.tabs.len(), f.m.active_tab), (2, Some(0)));
        f.m.dispatch(Command::CloseTab(0));
        assert_eq!((f.m.tabs.len(), f.m.active_tab), (1, Some(0)));
        f.m.dispatch(Command::CloseActiveTab);
        assert_eq!(f.m.active_tab, None);
    }

    #[test]
    fn unsupported_and_missing_files_give_a_message() {
        let mut f = fixture();
        f.m.dispatch(Command::OpenPath(PathBuf::from("/nowhere/a.docx")));
        assert!(f.m.notice.as_ref().is_some_and(|n| n.text.contains("Markdown")));
        f.m.dismiss_notice();
        f.m.dispatch(Command::OpenPath(PathBuf::from("/nowhere/a.md")));
        assert_eq!(f.m.notice.as_ref().map(|n| n.kind), Some(NoticeKind::Error));
    }

    #[test]
    fn home_list_pins_shows_missing_files_and_cleans_up() {
        let mut f = fixture();
        f.m.dispatch(Command::NewDocument(Kind::Document));
        f.m.dispatch(Command::NewDocument(Kind::Presentation));
        let (a, b) = (f.m.tabs[0].path.clone(), f.m.tabs[1].path.clone());
        f.m.dispatch(Command::CloseTab(1));
        let gone = f.m.settings.documents_dir.join("gone.md");
        std::fs::write(&gone, "x").unwrap();
        f.m.dispatch(Command::OpenPath(gone.clone()));
        f.m.dispatch(Command::CloseActiveTab);
        std::fs::remove_file(&gone).unwrap();
        let rows = f.m.recent_file_rows(now(), 30);
        assert_eq!(rows.len(), 3, "the missing file stays, shown light");
        assert!(rows.iter().find(|r| r.path == gone).is_some_and(|r| !r.exists));
        // opening a missing file is an error that keeps it in the list
        f.m.dispatch(Command::OpenPath(gone.clone()));
        assert_eq!(f.m.notice.as_ref().map(|n| n.kind), Some(NoticeKind::Error));
        assert_eq!(f.m.recent_file_rows(now(), 30).len(), 3);
        // pinned first, and the pin shows
        f.m.dispatch(Command::TogglePin(a.clone()));
        let rows = f.m.recent_file_rows(now(), 30);
        assert_eq!((rows[0].path.clone(), rows[0].pinned), (a.clone(), true));
        // show in folder: the file, or its folder when it is gone, else a message
        assert_eq!(f.m.dispatch(Command::RevealPath(a.clone())), vec![Effect::Reveal(a.clone())]);
        let dir = f.m.settings.documents_dir.clone();
        assert_eq!(f.m.dispatch(Command::RevealPath(gone.clone())), vec![Effect::Reveal(dir)]);
        f.m.dismiss_notice();
        assert!(f.m.dispatch(Command::RevealPath(PathBuf::from("/nowhere/a/b.md"))).is_empty());
        assert!(f.m.notice.is_some());
        // open in chat: attaches the file (once) and shows the AI panel; a missing file is refused
        f.m.settings.ai_enabled = true;
        f.m.settings.ai_panel_open = false;
        f.m.dispatch(Command::OpenInChat(b.clone()));
        f.m.dispatch(Command::OpenInChat(b.clone()));
        assert!(f.m.settings.ai_panel_open);
        assert_eq!(f.m.chat_files, std::slice::from_ref(&b));
        f.m.dispatch(Command::OpenInChat(gone.clone()));
        assert_eq!(f.m.chat_files.len(), 1);
        f.m.dispatch(Command::DetachFromChat(b.clone()));
        assert!(f.m.chat_files.is_empty());
        f.m.dispatch(Command::OpenInChat(b.clone()));
        // clean up only drops the missing file; remove drops one; clear all drops the rest
        f.m.dispatch(Command::CleanupRecent);
        assert_eq!(f.m.recent_file_rows(now(), 30).len(), 2);
        f.m.dispatch(Command::ForgetRecent(b.clone()));
        assert_eq!(f.m.recent_file_rows(now(), 30).len(), 1);
        assert!(b.is_file(), "the file itself is untouched");
        f.m.dispatch(Command::ClearRecent);
        assert!(f.m.recent_file_rows(now(), 30).is_empty() && a.is_file());
        // the list survives a restart
        let again = AppModel::new(Store::at(f.m.store.dir().to_path_buf()));
        assert!(again.recent_file_rows(now(), 30).is_empty());
    }

    #[test]
    fn open_dialog_starts_in_the_given_or_documents_folder() {
        let mut f = fixture();
        let docs = f.m.settings.documents_dir.clone();
        assert_eq!(f.m.dispatch(Command::OpenDialog(None)), vec![Effect::OpenFileDialog { start: docs.clone() }]);
        assert_eq!(f.m.dispatch(Command::OpenDialog(Some(PathBuf::from("/nope")))), vec![Effect::OpenFileDialog { start: docs }]);
        assert_eq!(f.m.dispatch(Command::Quit), vec![Effect::Quit]);
    }

    #[test]
    fn the_templates_per_page_follow_the_window() {
        // 3 columns of 161 px, thumbnails 228 px high: 2 rows in 700 px, leaving room for the page bar
        assert_eq!(templates_that_fit(520.0, 700.0), 6);
        assert_eq!(templates_that_fit(520.0, 600.0), 3, "one row when two would push the page bar out");
        assert_eq!(templates_that_fit(1100.0, 700.0), 12);
        assert_eq!(templates_that_fit(1100.0, 1000.0), 18, "a taller window shows a third row");
        // never nothing, even in a tiny window
        assert_eq!(templates_that_fit(100.0, 50.0), 1);
        let mut f = fixture();
        f.m.set_templates_page(1);
        f.m.fit_templates(1100.0, 700.0);
        assert_eq!((f.m.templates.per_page, f.m.templates.page), (12, 0), "the first template shown stays in view");
        assert_eq!((f.m.template_view().items.len(), f.m.template_view().pages), (11, 1));
        f.m.fit_templates(520.0, 700.0);
        assert_eq!((f.m.template_view().items.len(), f.m.template_view().pages), (6, 2));
    }

    #[test]
    fn template_paging_and_filters() {
        let mut f = fixture();
        let v = f.m.template_view();
        assert_eq!(v.total, 11);
        assert_eq!((v.items.len(), v.pages), (10, 2));
        f.m.set_templates_page(1);
        assert_eq!(f.m.template_view().items.len(), 1);
        f.m.set_templates_kind(Kind::Presentation);
        assert_eq!(f.m.templates.page, 0);
        assert_eq!(f.m.template_view().total, 8);
        f.m.set_templates_query("zzz".into());
        let v = f.m.template_view();
        assert_eq!((v.total, v.pages, v.items.len()), (0, 1, 0));
        f.m.set_templates_page(9);
        assert_eq!(f.m.template_view().page, 0, "page is clamped");
    }

    #[test]
    fn one_key_per_provider_serves_all_its_models() {
        let mut f = fixture();
        fill_form(&mut f.m, "anthropic", "claude-sonnet-5-5");
        f.m.submit_add_model();
        // the second model of the same provider asks for no key and keeps the address
        f.m.begin_add_model();
        f.m.select_provider("anthropic");
        assert!(f.m.add_model.as_ref().is_some_and(|a| a.connected));
        f.m.add_model.as_mut().unwrap().model = "claude-haiku-5-5".into();
        assert!(f.m.add_model.as_ref().unwrap().valid());
        f.m.submit_add_model();
        assert_eq!(f.m.settings.models.len(), 2);
        assert_eq!(f.m.api_key("m1").as_deref(), Some("sk-test"));
        assert_eq!(f.m.api_key("m2").as_deref(), Some("sk-test"));
        // the key stays until the provider's last model is removed
        f.m.delete_model("m1");
        assert_eq!(f.m.api_key("m2").as_deref(), Some("sk-test"));
        f.m.delete_model("m2");
        assert!(!f.m.provider_connected("anthropic"));
        assert_eq!(f.m.secret("ai.provider.anthropic"), None);
        // a provider that is not connected still needs its key
        f.m.begin_add_model();
        f.m.select_provider("anthropic");
        f.m.add_model.as_mut().unwrap().model = "claude-haiku-5-5".into();
        assert!(!f.m.add_model.as_ref().unwrap().valid());
    }

    #[test]
    fn switching_models_stays_inside_the_provider_and_never_asks_for_a_key() {
        let mut f = fixture();
        assert!(f.m.model_choices().is_empty());
        fill_form(&mut f.m, "anthropic", "claude-sonnet-5-5");
        f.m.submit_add_model();
        let ladder: Vec<_> = providers::get("anthropic").unwrap().chat_models_by_cost().iter().map(|m| m.id.as_str()).collect();
        assert!(ladder.len() >= 3);
        let choices = f.m.model_choices();
        assert!(choices.iter().all(|c| c.provider == "anthropic"), "only connected providers are offered");
        assert_eq!(choices.iter().filter(|c| c.current).map(|c| c.model.as_str()).collect::<Vec<_>>(), ["claude-sonnet-5-5"]);
        assert!(choices.iter().all(|c| c.price.contains('$') && !c.name.contains('$')));
        // step down: a model with no profile yet gets one, on the same provider and key
        let at = ladder.iter().position(|m| *m == "claude-sonnet-5-5").unwrap();
        f.m.dispatch(Command::StepModel(Step::Pricier));
        let up = f.m.settings.model_for("chat").unwrap().clone();
        assert_eq!(up.provider, "anthropic");
        assert_eq!(Some(up.model.as_str()), ladder.get(at + 1).copied().or(Some("claude-sonnet-5-5")));
        f.m.dispatch(Command::UseModel { provider: "anthropic".into(), model: ladder[0].into() });
        assert_eq!(f.m.settings.model_for("chat").map(|m| m.model.as_str()), Some(ladder[0]));
        assert_eq!(f.m.api_key(&f.m.settings.model_for("chat").unwrap().id.clone()).as_deref(), Some("sk-test"));
        // already the cheapest: a message, no change
        f.m.dispatch(Command::StepModel(Step::Cheaper));
        assert_eq!(f.m.settings.model_for("chat").map(|m| m.model.as_str()), Some(ladder[0]));
        assert!(f.m.notice.as_ref().is_some_and(|n| n.text.contains("cheapest")));
        // the default model for other tasks is untouched
        assert_eq!(f.m.settings.default_model.as_deref(), Some("m1"));
        // a provider that is not connected cannot be switched to
        f.m.dispatch(Command::UseModel { provider: "openai".into(), model: "x".into() });
        assert!(f.m.notice.as_ref().is_some_and(|n| n.kind == NoticeKind::Error));
    }

    #[test]
    fn providers_are_grouped_by_region_and_custom_ones_join_their_region() {
        let mut f = fixture();
        let labels: Vec<_> = f.m.provider_groups().iter().map(|g| g.0).collect();
        assert_eq!(labels, ["Europe (EU)", "Asia-Pacific (APAC)", "North America (US/CA)", "Local & custom"]);
        f.m.begin_custom_provider();
        f.m.custom_form = Some(CustomProviderForm {
            name: "Corp".into(),
            base_url: "https://ai.corp/v1".into(),
            region: "eu".into(),
            protocol: "anthropic".into(),
        });
        f.m.submit_custom_provider();
        let groups = f.m.provider_groups();
        assert!(groups[0].1.iter().any(|(id, _)| id == "custom:Corp"));
        assert!(!groups[3].1.iter().any(|(id, _)| id == "custom:Corp"));
        assert!(f.m.provider_help("custom:Corp").contains("Anthropic"));
        assert!(f.m.provider_help("mistral").contains("console.mistral.ai"));
    }

    #[test]
    fn providers_with_their_own_endpoint_start_empty_with_a_hint() {
        let mut f = fixture();
        f.m.begin_add_model();
        f.m.select_provider("microsoft");
        assert_eq!(f.m.add_model.as_ref().map(|a| a.base_url.as_str()), Some(""));
        assert!(f.m.endpoint_hint("microsoft").contains("azure"));
        assert_eq!(f.m.endpoint_hint("openai"), "Enter URL");
        f.m.select_provider("deepseek");
        assert_eq!(f.m.add_model.as_ref().map(|a| a.base_url.as_str()), Some("https://api.deepseek.com"));
        assert!(!f.m.provider_models("deepseek").is_empty());
    }

    #[test]
    fn general_settings_wait_for_apply_and_persist() {
        let mut f = fixture();
        assert!(!f.m.general_dirty());
        f.m.draft.user_name = "  Kris ".into();
        f.m.draft.appearance = Appearance::Dark;
        f.m.draft.dark_theme = ThemeFamily::Solarized;
        assert!(f.m.general_dirty());
        assert_eq!(f.m.settings.appearance, Appearance::System);
        f.m.apply_general();
        assert_eq!((f.m.settings.user_name.as_str(), f.m.settings.appearance), ("Kris", Appearance::Dark));
        assert_eq!(f.m.settings.theme_attrs().0, "solarized-dark");
        assert!(!f.m.general_dirty());
        let reloaded = f.m.store.load_settings();
        assert_eq!((reloaded.appearance, reloaded.dark_theme), (Appearance::Dark, ThemeFamily::Solarized));
        let gen_before = f.m.form_gen;
        f.m.reset_general();
        assert!(f.m.general_dirty());
        assert!(f.m.form_gen > gen_before, "fields are rebuilt after a reset");
        // leaving and re-entering Settings discards an unapplied draft
        f.m.dispatch(Command::OpenSettings(SettingsSection::General));
        f.m.dispatch(Command::ShowPage(Page::Home));
        f.m.dispatch(Command::OpenSettings(SettingsSection::General));
        assert!(!f.m.general_dirty());
    }

    #[test]
    fn adding_a_model_validates_names_it_and_keeps_the_key_out_of_the_file() {
        let mut f = fixture();
        f.m.begin_add_model();
        assert!(!f.m.add_model.as_ref().unwrap().valid());
        f.m.select_provider("anthropic");
        assert_eq!(f.m.add_model.as_ref().unwrap().base_url, "https://api.anthropic.com");
        f.m.submit_add_model();
        assert!(f.m.settings.models.is_empty(), "invalid form does nothing");
        fill_form(&mut f.m, "anthropic", "claude-sonnet-5-5");
        f.m.submit_add_model();
        let m = &f.m.settings.models[0];
        assert_eq!(m.name, "Anthropic - Claude Sonnet 5.5");
        assert_eq!(f.m.settings.default_model.as_deref(), Some("m1"));
        assert_eq!(f.m.api_key("m1").as_deref(), Some("sk-test"));
        let saved = std::fs::read_to_string(f.m.store.dir().join("settings.json")).unwrap();
        assert!(!saved.contains("sk-test"));
        assert!(f.m.add_model.is_none());
    }

    #[test]
    fn local_providers_need_no_key_but_need_a_model() {
        let mut f = fixture();
        f.m.begin_add_model();
        f.m.select_provider("ollama");
        assert!(!f.m.add_model.as_ref().unwrap().valid());
        f.m.add_model.as_mut().unwrap().model = "llama3".into();
        f.m.add_model.as_mut().unwrap().profile_name = "My Llama".into();
        assert!(f.m.add_model.as_ref().unwrap().valid());
        f.m.submit_add_model();
        assert_eq!(f.m.settings.models[0].name, "My Llama");
        assert_eq!(f.m.api_key("m1"), None);
    }

    #[test]
    fn custom_providers_appear_in_the_provider_list() {
        let mut f = fixture();
        f.m.begin_custom_provider();
        f.m.custom_form = Some(CustomProviderForm { name: "Corp".into(), base_url: "ftp://x".into(), ..Default::default() });
        f.m.submit_custom_provider();
        assert!(f.m.settings.custom_providers.is_empty() && f.m.notice.is_some());
        f.m.custom_form = Some(CustomProviderForm { name: "Corp".into(), base_url: "https://ai.corp/v1".into(), ..Default::default() });
        f.m.submit_custom_provider();
        assert!(f.m.provider_options().iter().any(|(id, _)| id == "custom:Corp"));
        f.m.begin_add_model();
        f.m.select_provider("custom:Corp");
        assert_eq!(f.m.add_model.as_ref().unwrap().base_url, "https://ai.corp/v1");
        f.m.custom_form = Some(CustomProviderForm { name: "corp".into(), base_url: "https://x".into(), ..Default::default() });
        f.m.submit_custom_provider();
        assert_eq!(f.m.settings.custom_providers.len(), 1, "names are unique");
        f.m.delete_custom_provider("Corp");
        assert!(f.m.settings.custom_providers.is_empty());
    }

    #[test]
    fn assignments_default_and_delete() {
        let mut f = fixture();
        fill_form(&mut f.m, "anthropic", "claude-sonnet-5-5");
        f.m.submit_add_model();
        fill_form(&mut f.m, "openai", "gpt-4o");
        f.m.submit_add_model();
        assert_eq!(f.m.settings.model_for("code").unwrap().id, "m1");
        f.m.assign_model("code", Some("m2".into()));
        assert_eq!(f.m.settings.model_for("code").unwrap().id, "m2");
        f.m.assign_model("code", Some("zz".into()));
        assert_eq!(f.m.settings.model_for("code").unwrap().id, "m1", "unknown ids unassign");
        f.m.make_default_model("m2");
        assert_eq!(f.m.settings.model_for("ocr").unwrap().id, "m2");
        f.m.delete_model("m2");
        assert_eq!(f.m.settings.default_model.as_deref(), Some("m1"));
        assert_eq!(f.m.api_key("m2"), None);
    }

    #[test]
    fn mcp_editor_validates_before_saving() {
        let mut f = fixture();
        assert!(f.m.mcp_server_names().is_empty());
        f.m.open_mcp_editor();
        f.m.set_mcp_text("{ broken".into());
        f.m.save_mcp();
        assert!(f.m.mcp.as_ref().unwrap().error.is_some());
        assert_eq!(f.m.settings.mcp_config, default_mcp());
        f.m.set_mcp_text(r#"{"mcpServers":{"files":{"command":"x"},"web":{"url":"u"}}}"#.into());
        f.m.save_mcp();
        assert!(f.m.mcp.is_none());
        assert_eq!(f.m.mcp_server_names(), ["files", "web"]);
        assert!(f.m.store.load_settings().mcp_config.contains("files"));
    }

    #[test]
    fn api_keys_are_saved_encrypted_in_the_settings_and_survive_a_restart() {
        let mut f = fixture();
        fill_form(&mut f.m, "anthropic", "claude-sonnet-5-5");
        f.m.submit_add_model();
        let file = std::fs::read_to_string(f.m.store.dir().join("settings.json")).unwrap();
        assert!(file.contains("\"secrets\"") && file.contains("v1:"));
        assert!(!file.contains("sk-test"), "never as plain text");
        let id = f.m.settings.models[0].id.clone();
        assert_eq!(f.m.api_key(&id).as_deref(), Some("sk-test"));
        let again = AppModel::new(f.m.store.clone());
        assert_eq!(again.api_key(&id).as_deref(), Some("sk-test"));
        // settings copied to another computer: the key is simply not there, nothing breaks
        let other = tempfile::tempdir().unwrap();
        std::fs::copy(f.m.store.dir().join("settings.json"), {
            std::fs::create_dir_all(other.path().join("cfg")).unwrap();
            other.path().join("cfg").join("settings.json")
        })
        .unwrap();
        let moved = AppModel::new(Store::at(other.path().join("cfg")));
        assert_eq!(moved.api_key(&id), None);
        assert!(!moved.provider_connected("anthropic"));
    }

    #[test]
    fn a_connected_model_can_be_edited_and_moved_to_another_provider() {
        let mut f = fixture();
        fill_form(&mut f.m, "deepseek", "deepseek-chat");
        f.m.submit_add_model();
        let id = f.m.settings.models[0].id.clone();
        assert_eq!(f.m.settings.default_model.as_deref(), Some(id.as_str()));
        // same provider, another model: the key stays and nothing is asked for
        f.m.begin_edit_model(&id);
        let form = f.m.add_model.clone().unwrap();
        assert!(form.connected && form.editing.as_deref() == Some(id.as_str()) && form.api_key.is_empty());
        // to another provider, which needs its own key
        f.m.select_provider("mistral");
        assert!(!f.m.add_model.as_ref().unwrap().valid());
        f.m.add_model.as_mut().unwrap().api_key = "sk-mistral".into();
        f.m.add_model.as_mut().unwrap().model = "mistral-large-latest".into();
        f.m.add_model.as_mut().unwrap().profile_name.clear();
        f.m.submit_add_model();
        assert_eq!(f.m.settings.models.len(), 1, "the model is changed, not added");
        let m = &f.m.settings.models[0];
        assert_eq!((m.id.as_str(), m.provider.as_str()), (id.as_str(), "mistral"));
        assert!(m.name.starts_with("Mistral"), "{}", m.name);
        assert_eq!(f.m.settings.default_model.as_deref(), Some(id.as_str()));
        assert_eq!(f.m.api_key(&id).as_deref(), Some("sk-mistral"));
        assert!(!f.m.settings.secrets.contains_key("ai.provider.deepseek"), "the old provider's key goes with it");
        assert!(f.m.add_model.is_none());
    }

    #[test]
    fn editing_an_unknown_model_does_nothing() {
        let mut f = fixture();
        f.m.begin_edit_model("nope");
        assert!(f.m.add_model.is_none());
    }

    fn connect_anthropic(f: &mut Fixture) {
        fill_form(&mut f.m, "anthropic", "claude-sonnet-5-5");
        f.m.submit_add_model();
    }

    const ANTHROPIC_ANSWER: &str =
        r#"{"data":[{"id":"claude-sonnet-5-5","display_name":"Sonnet"},{"id":"claude-brand-new-9","display_name":"Claude Brand New 9"}]}"#;

    #[test]
    fn a_connected_provider_is_asked_for_its_models_and_the_answer_replaces_the_suggestions() {
        let mut f = fixture();
        // nothing connected, nothing to ask with
        assert!(f.m.dispatch(Command::FetchModels(None)).is_empty());
        connect_anthropic(&mut f);
        let effects = f.m.dispatch(Command::FetchModels(None));
        let [Effect::FetchModels { provider, request }] = effects.as_slice() else { panic!("{effects:?}") };
        assert_eq!(provider, "anthropic");
        assert_eq!(request.url, "https://api.anthropic.com/v1/models?limit=1000");
        assert!(request.headers.contains(&("x-api-key".to_string(), "sk-test".to_string())));
        assert!(f.m.models_note("anthropic").contains("Loading"));
        // not asked twice while the answer is on its way, nor again at once
        assert!(f.m.dispatch(Command::FetchModels(Some("anthropic".into()))).is_empty());
        f.m.dispatch(Command::ModelsFetched { provider: "anthropic".into(), result: Ok(ANTHROPIC_ANSWER.into()) });
        let list = f.m.provider_models("anthropic");
        assert_eq!(list.len(), 2, "only what the provider offers");
        assert!(list.contains(&("claude-brand-new-9".to_string(), "Claude Brand New 9".to_string())));
        assert!(f.m.models_note("anthropic").starts_with("2 models from the provider"));
        // known models keep their price, new ones have none and come last
        let labels = f.m.provider_model_labels("anthropic");
        assert!(labels[0].2.contains('$') && labels[1].2.is_empty(), "{labels:?}");
        // the chat switcher offers the same models
        assert!(f.m.model_choices().iter().any(|c| c.model == "claude-brand-new-9"));
        assert!(f.m.dispatch(Command::FetchModels(None)).is_empty(), "asked less than ten minutes ago");
        assert_eq!(f.m.dispatch(Command::ReloadModels("anthropic".into())).len(), 1, "the button always asks");
        // the list survives a restart
        let again = AppModel::new(Store::at(f.m.store.dir().to_path_buf()));
        assert!(again.provider_models("anthropic").iter().any(|(i, _)| i == "claude-brand-new-9"));
    }

    #[test]
    fn a_failed_question_keeps_the_suggestions_and_is_asked_again_next_time() {
        let mut f = fixture();
        connect_anthropic(&mut f);
        let before = f.m.provider_models("anthropic");
        f.m.dispatch(Command::FetchModels(None));
        f.m.dispatch(Command::ModelsFetched { provider: "anthropic".into(), result: Err("https://x: status code 401".into()) });
        assert_eq!(f.m.provider_models("anthropic"), before);
        let note = f.m.models_note("anthropic");
        assert!(note.contains("API key") && note.contains("came with Glyphra"), "{note}");
        assert!(f.m.notice.is_none(), "no pop-up for a background question");
        assert_eq!(f.m.dispatch(Command::FetchModels(None)).len(), 1);
        // an answer that is not a model list changes nothing either
        f.m.dispatch(Command::ModelsFetched { provider: "anthropic".into(), result: Ok("<html>".into()) });
        assert_eq!(f.m.provider_models("anthropic"), before);
        assert!(!f.m.models_note("anthropic").is_empty());
    }

    #[test]
    fn the_form_asks_with_the_key_and_address_typed_before_connecting() {
        let mut f = fixture();
        f.m.begin_add_model();
        f.m.select_provider("openai");
        assert!(f.m.dispatch(Command::FetchModels(Some("openai".into()))).is_empty(), "no key yet");
        f.m.add_model.as_mut().unwrap().api_key = "sk-typed".into();
        let effects = f.m.dispatch(Command::FetchModels(Some("openai".into())));
        let [Effect::FetchModels { request, .. }] = effects.as_slice() else { panic!("{effects:?}") };
        assert_eq!(request.url, "https://api.openai.com/v1/models");
        assert_eq!(request.headers[0].1, "Bearer sk-typed");
        // another key is another question
        f.m.dispatch(Command::ModelsFetched { provider: "openai".into(), result: Err("status code 401".into()) });
        f.m.add_model.as_mut().unwrap().api_key = "sk-other".into();
        assert_eq!(f.m.dispatch(Command::FetchModels(Some("openai".into()))).len(), 1);
        // a local server needs no key; your own provider speaks the API you chose
        f.m.select_provider("ollama");
        let effects = f.m.dispatch(Command::FetchModels(Some("ollama".into())));
        let [Effect::FetchModels { request, .. }] = effects.as_slice() else { panic!("{effects:?}") };
        assert_eq!(request.url, "http://localhost:11434/api/tags");
        assert!(request.headers.is_empty());
        // Microsoft has no address until you type yours
        f.m.select_provider("microsoft");
        f.m.add_model.as_mut().unwrap().api_key = "k".into();
        assert!(f.m.dispatch(Command::FetchModels(Some("microsoft".into()))).is_empty());
        f.m.add_model.as_mut().unwrap().base_url = "https://res.openai.azure.com/openai/v1".into();
        let effects = f.m.dispatch(Command::FetchModels(Some("microsoft".into())));
        let [Effect::FetchModels { request, .. }] = effects.as_slice() else { panic!("{effects:?}") };
        assert_eq!(request.url, "https://res.openai.azure.com/openai/v1/models");
        assert_eq!(request.headers[0], ("api-key".to_string(), "k".to_string()));
    }
}
