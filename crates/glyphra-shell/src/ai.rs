//! Glyphra AI inside the model: the configuration (with its draft for Settings › AI), the keys
//! in the system keychain, the AI Chat conversation and the AI Tools run. No UI code.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use kd_ai::tasks::Task;
use kd_ai::{AiConfig, AiError, AiHub, Catalog, ChatState, Job, JobEvent, ModelRef, ProviderConfig, ProviderKind, Scope};
use kd_store::secrets::{MemorySecrets, SecretStore};

/// The keychain entry of a provider's key.
pub fn secret_name(provider_id: &str) -> String {
    format!("ai.{provider_id}")
}

/// The short name of a provider for model labels: its own name, else the company ("Anthropic").
fn short_name(p: &ProviderConfig) -> String {
    p.name.clone().filter(|n| !n.trim().is_empty()).unwrap_or_else(|| p.kind.vendor().to_string())
}

/// Which screen of Settings › AI providers is showing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AiScreen {
    #[default]
    List,
    /// Choose the kind of provider to add.
    AddKind,
    /// Fill in the provider chosen on [`AiScreen::AddKind`] (it is in [`AiState::form`]).
    AddForm,
    /// Edit the provider with this id.
    Edit(String),
}

/// The result of "Test connection".
/// What a connection test reports: the models, and the account note.
type TestAnswer = Result<(Vec<String>, Option<String>), String>;

pub enum TestState {
    Running(Receiver<TestAnswer>),
    /// The models the server offers, and its account note (credits, usage).
    Ok(Vec<String>, Option<String>),
    Err(String),
}

/// A provider's coloured square and initials (brand colours, so not theme variables).
pub fn kind_badge(kind: ProviderKind) -> (&'static str, &'static str) {
    use ProviderKind as K;
    match kind {
        K::AlephAlpha => ("#1F2937", "AA"),
        K::BlackForestLabs => ("#111111", "BF"),
        K::DeepL => ("#0F2B46", "DL"),
        K::Mistral => ("#FA520F", "M"),
        K::Alibaba => ("#FF6A00", "Qw"),
        K::DeepSeek => ("#4D6BFE", "DS"),
        K::Moonshot => ("#16162A", "K"),
        K::Sakana => ("#C81E3A", "S"),
        K::Zai => ("#2D3A8C", "Z"),
        K::Anthropic => ("#D97757", "A"),
        K::Cohere => ("#39594D", "Co"),
        K::Gemini => ("#4285F4", "G"),
        K::Meta => ("#0467DF", "M"),
        K::Microsoft => ("#0078D4", "MS"),
        K::OpenAi => ("#10A37F", "O"),
        K::SpaceXAi => ("#000000", "X"),
        K::LmStudio => ("#6B46C1", "LM"),
        K::Ollama => ("#4B5563", "Ol"),
        K::OpenAiCompatible => ("#64748B", "</>"),
        K::Custom => ("#64748B", "+"),
    }
}

/// The title on the Add screen: the company, or "Custom provider".
pub fn kind_title(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Custom => "Custom provider",
        ProviderKind::OpenAiCompatible => "OpenAI-compatible",
        k => k.vendor(),
    }
}

/// How a provider stands: (Fluent id, variable, tone) with tone 1 good, 2 bad, 0 muted.
pub fn status(p: &ProviderConfig) -> (&'static str, Option<String>, u8) {
    use kd_ai::KeySource;
    if !p.enabled {
        return ("ai-status-off", None, 0);
    }
    if p.base_url().is_empty() {
        return ("ai-status-endpoint", None, 2);
    }
    match p.key_source() {
        KeySource::None if p.kind.needs_key() => ("ai-status-key", None, 2),
        KeySource::Env(v) => ("ai-status-env", Some(v.to_string()), 1),
        KeySource::Stored => ("ai-status-ready", None, 1),
        KeySource::None if p.kind.is_local() => ("ai-status-local", None, 1),
        KeySource::None => ("ai-status-ready", None, 1),
    }
}

/// One model the user can pick: `value` is `provider:model`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelChoice {
    pub value: String,
    /// "Anthropic - Claude Sonnet 5.5"
    pub label: String,
}

/// An AI tool being used: the task, the text it produced and the request that makes it.
pub struct ToolRun {
    pub task: &'static Task,
    pub output: String,
    pub error: Option<String>,
    pub job: Option<Job>,
}

impl ToolRun {
    pub fn running(&self) -> bool {
        self.job.is_some()
    }
}

pub struct AiState {
    /// What is in force (keys filled in from the keychain).
    pub hub: AiHub,
    /// What Settings › AI edits; [`AiState::save`] makes it real.
    pub draft: AiConfig,
    pub catalog: Catalog,
    pub chat: ChatState,
    pub tool: Option<ToolRun>,
    /// A file added to the next chat message: (name, text for the model).
    pub attachment: Option<(String, String)>,
    secrets: Arc<dyn SecretStore>,
    dir: Option<PathBuf>,
    /// Providers whose key could not be put in the keychain yet (tried again at the next Apply).
    pending: BTreeSet<String>,
    pub screen: AiScreen,
    /// The provider being added (see [`AiScreen::AddForm`]).
    pub form: Option<ProviderConfig>,
    pub key_visible: bool,
    /// "Test connection" results by provider id.
    pub tests: HashMap<String, TestState>,
}

impl AiState {
    /// No disk, no keychain: the stand-in used while the real model loads, and in document windows.
    pub fn empty() -> AiState {
        Self::build(AiConfig::default(), Arc::new(MemorySecrets::default()), None)
    }

    /// Read `ai.toml` from `dir` and the keys from `secrets`. A key still in the file (an older or
    /// hand-edited `ai.toml`) is moved into the keychain at once.
    pub fn load(dir: PathBuf, secrets: Arc<dyn SecretStore>) -> (AiState, Option<String>) {
        let mut config = AiConfig::load(&dir);
        let plain: Vec<String> =
            config.providers.iter().filter(|p| p.api_key.as_deref().is_some_and(|k| !k.trim().is_empty())).map(|p| p.id.clone()).collect();
        for p in &mut config.providers {
            if p.api_key.as_deref().is_none_or(|k| k.trim().is_empty())
                && let Ok(Some(k)) = secrets.get(&secret_name(&p.id))
            {
                p.api_key = Some(k);
            }
        }
        let mut state = Self::build(config, secrets, Some(dir));
        let warning = if plain.is_empty() { None } else { state.persist_keys(None).err() };
        if !plain.is_empty() {
            let _ = state.write_file();
        }
        (state, warning)
    }

    fn build(config: AiConfig, secrets: Arc<dyn SecretStore>, dir: Option<PathBuf>) -> AiState {
        let hub = AiHub::in_memory(config.clone());
        AiState {
            catalog: hub.catalog(),
            hub,
            draft: config,
            chat: ChatState::new(None, false),
            tool: None,
            attachment: None,
            secrets,
            dir,
            pending: BTreeSet::new(),
            screen: AiScreen::List,
            form: None,
            key_visible: false,
            tests: HashMap::new(),
        }
    }

    // ------------------------------------------------------------------ settings

    pub fn dirty(&self) -> bool {
        self.draft != self.hub.config()
    }

    pub fn reset_draft(&mut self) {
        self.screen = AiScreen::List;
        self.form = None;
        self.key_visible = false;
        self.tests.clear();
        self.draft = self.hub.config();
    }

    pub fn show(&mut self, screen: AiScreen) {
        self.key_visible = false;
        if screen != AiScreen::AddForm {
            self.form = None;
        }
        self.screen = screen;
    }

    /// A kind was chosen on the Add screen: start its form.
    pub fn choose_kind(&mut self, kind: ProviderKind) {
        let mut p = ProviderConfig::new(kind);
        p.id = self.draft.free_id(kind.default_id());
        self.form = Some(p);
        self.show(AiScreen::AddForm);
    }

    /// The provider on the form goes into the draft and is shown for editing.
    pub fn add_form_provider(&mut self) {
        if let Some(p) = self.form.take() {
            let id = p.id.clone();
            self.draft.providers.push(p);
            self.show(AiScreen::Edit(id));
        }
    }

    /// The provider the form on screen edits: the new one, or one in the draft.
    pub fn target(&mut self) -> Option<&mut ProviderConfig> {
        match &self.screen {
            AiScreen::AddForm => self.form.as_mut(),
            AiScreen::Edit(id) => {
                let id = id.clone();
                self.draft.provider_mut(&id)
            }
            _ => None,
        }
    }

    /// Read-only view of [`AiState::target`].
    pub fn target_ref(&self) -> Option<&ProviderConfig> {
        match &self.screen {
            AiScreen::AddForm => self.form.as_ref(),
            AiScreen::Edit(id) => self.draft.provider(id),
            _ => None,
        }
    }

    /// Take a provider out of the draft; the models that used it are forgotten.
    pub fn remove_provider(&mut self, id: &str) {
        self.draft.providers.retain(|p| p.id != id);
        self.draft.models.retain(|_, m| m.provider != id);
        if self.draft.default_model.as_ref().is_some_and(|m| m.provider == id) {
            self.draft.default_model = None;
        }
        self.tests.remove(id);
        self.show(AiScreen::List);
    }

    /// Ask the provider on the form for its models, on a worker thread (see [`AiState::pump`]).
    pub fn test_target(&mut self) {
        let Some(p) = self.target_ref().cloned() else { return };
        let (tx, rx) = channel();
        let probe = AiHub::in_memory(AiConfig { providers: vec![p.clone()], ..self.draft.clone() });
        let id = p.id.clone();
        std::thread::spawn(move || {
            let r = probe.fetch_models(&id).map(|list| (list, probe.account_note(&id))).map_err(|e| e.to_string());
            let _ = tx.send(r);
        });
        self.tests.insert(p.id, TestState::Running(rx));
    }

    /// The models to offer for `p`: its own, the catalog's and a test's result.
    pub fn offered(&self, p: &ProviderConfig) -> Vec<String> {
        let mut out = p.models.clone();
        let mut push = |m: &str| {
            if !out.iter().any(|x| x == m) {
                out.push(m.to_string());
            }
        };
        for m in self.catalog.for_kind(p.kind) {
            push(&m.id);
        }
        if let Some(TestState::Ok(list, _)) = self.tests.get(&p.id) {
            for m in list {
                push(m);
            }
        }
        out
    }

    /// Set the model of a scope (`home`, `markdown`, ...); `None` = the default model.
    pub fn set_scope_model(&mut self, scope: &str, value: &str) {
        match ModelRef::parse(value) {
            Some(m) => {
                self.draft.models.insert(scope.to_string(), m);
            }
            None => {
                self.draft.models.remove(scope);
            }
        }
    }

    pub fn set_default_model(&mut self, value: &str) {
        self.draft.default_model = ModelRef::parse(value);
    }

    /// Make the draft real: keys to the keychain, the rest to `ai.toml`, and the hub uses it from
    /// now on. On `Err` the settings are applied anyway; a key the keychain refused stays in memory
    /// until Glyphra closes (the message says why).
    pub fn save(&mut self) -> Result<(), String> {
        let old = self.hub.config();
        let _ = self.hub.set_config(self.draft.clone());
        let mut problems = Vec::new();
        for gone in old.providers.iter().filter(|o| self.draft.provider(&o.id).is_none()) {
            if let Err(e) = self.secrets.delete(&secret_name(&gone.id)) {
                problems.push(e.to_string());
            }
        }
        if let Err(e) = self.persist_keys(Some(&old)) {
            problems.push(e);
        }
        if let Err(e) = self.write_file() {
            problems.push(e);
        }
        self.catalog = self.hub.catalog();
        problems.dedup();
        if problems.is_empty() { Ok(()) } else { Err(problems.join("; ")) }
    }

    /// Put the keys that changed (or could not be stored last time) in the keychain. `old` is
    /// what was in force before; `None` stores every key.
    fn persist_keys(&mut self, old: Option<&AiConfig>) -> Result<(), String> {
        let config = self.hub.config();
        let trimmed = |p: &ProviderConfig| p.api_key.as_deref().map(str::trim).filter(|k| !k.is_empty()).map(str::to_string);
        let mut first_error = None;
        for p in &config.providers {
            let key = trimmed(p);
            let old_key = old.and_then(|o| o.provider(&p.id)).and_then(trimmed);
            let pending = self.pending.contains(&p.id);
            if old.is_some() && key == old_key && !pending {
                continue;
            }
            let name = secret_name(&p.id);
            let result = match &key {
                Some(k) => self.secrets.set(&name, k),
                None if old_key.is_some() || pending => self.secrets.delete(&name),
                None => Ok(()),
            };
            match result {
                Ok(()) => {
                    self.pending.remove(&p.id);
                }
                Err(e) => {
                    if key.is_some() {
                        self.pending.insert(p.id.clone());
                    }
                    first_error.get_or_insert(e.to_string());
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    fn write_file(&self) -> Result<(), String> {
        match &self.dir {
            Some(dir) => self.hub.config().save(dir).map_err(|e| e.to_string()),
            None => Ok(()),
        }
    }

    // ------------------------------------------------------------------ models

    /// Every model of the providers in `config` that can chat, for a picker.
    pub fn choices(&self, config: &AiConfig) -> Vec<ModelChoice> {
        let mut out = Vec::new();
        for p in config.providers.iter().filter(|p| p.usable_for_chat()) {
            let mut ids: Vec<String> = p.models.clone();
            if let Some(d) = &p.default_model
                && !ids.contains(d)
            {
                ids.push(d.clone());
            }
            for m in self.catalog.for_kind(p.kind) {
                if !ids.contains(&m.id) {
                    ids.push(m.id.clone());
                }
            }
            for id in ids {
                let info = self.catalog.info(p.kind, &id);
                let name = if info.name.is_empty() { id.clone() } else { info.name };
                out.push(ModelChoice { value: format!("{}:{}", p.id, id), label: format!("{} - {}", short_name(p), name) });
            }
        }
        out
    }

    /// The model the chat will use: the one picked in the composer, else Settings › AI's.
    pub fn chat_model(&self) -> Option<ModelRef> {
        self.chat.model.clone().or_else(|| self.hub.model_for(Scope::Home))
    }

    /// "Anthropic - Claude Sonnet 5.5" for a model of the applied configuration.
    pub fn label_of(&self, m: &ModelRef) -> String {
        let config = self.hub.config();
        match config.provider(&m.provider) {
            Some(p) => {
                let info = self.catalog.info(p.kind, &m.model);
                let name = if info.name.is_empty() { m.model.clone() } else { info.name };
                format!("{} - {}", short_name(p), name)
            }
            None => m.to_string(),
        }
    }

    /// Is there a provider that can answer at all?
    pub fn ready(&self) -> bool {
        self.hub.model_for(Scope::Home).is_some() || self.chat.model.is_some()
    }

    // ------------------------------------------------------------------ running

    /// Take in news from the running chat and tool; true when the screen has to be redrawn.
    pub fn pump(&mut self) -> bool {
        let mut changed = self.chat.pump();
        for t in self.tests.values_mut() {
            if let TestState::Running(rx) = t
                && let Ok(r) = rx.try_recv()
            {
                changed = true;
                *t = match r {
                    Ok((list, note)) => TestState::Ok(list, note),
                    Err(e) => TestState::Err(e),
                };
            }
        }
        if let Some(run) = self.tool.as_mut()
            && let Some(job) = run.job.as_mut()
        {
            let mut done = false;
            for e in job.poll() {
                changed = true;
                match e {
                    JobEvent::Text(t) => run.output.push_str(&t),
                    JobEvent::Done(Ok(r)) => {
                        if run.output.is_empty() {
                            run.output = r.text;
                        }
                        done = true;
                    }
                    JobEvent::Done(Err(AiError::Cancelled)) => done = true,
                    JobEvent::Done(Err(e)) => {
                        run.error = Some(e.to_string());
                        done = true;
                    }
                    _ => {}
                }
            }
            if done {
                run.job = None;
            }
        }
        changed
    }

    pub fn busy(&self) -> bool {
        self.tests.values().any(|t| matches!(t, TestState::Running(_)))
            || self.chat.running()
            || self.tool.as_ref().is_some_and(ToolRun::running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(dir: &std::path::Path) -> (AiState, Arc<MemorySecrets>) {
        let secrets = Arc::new(MemorySecrets::default());
        let (s, w) = AiState::load(dir.to_path_buf(), secrets.clone());
        assert!(w.is_none());
        (s, secrets)
    }

    #[test]
    fn keys_go_to_the_keychain_and_never_to_the_file() {
        let d = tempfile::tempdir().unwrap();
        let (mut s, secrets) = state(d.path());
        assert!(!s.dirty());
        s.draft.provider_mut("anthropic").unwrap().api_key = Some("  sk-secret ".into());
        assert!(s.dirty());
        s.save().unwrap();
        assert!(!s.dirty());
        assert_eq!(secrets.get("ai.anthropic").unwrap().as_deref(), Some("sk-secret"));
        let text = std::fs::read_to_string(d.path().join("ai.toml")).unwrap();
        assert!(!text.contains("sk-secret"), "{text}");
        // a restart finds the key again
        let secrets2 = secrets.clone();
        let (again, _) = AiState::load(d.path().to_path_buf(), secrets2);
        assert_eq!(again.hub.config().provider("anthropic").unwrap().api_key().as_deref(), Some("sk-secret"));
        // clearing the key removes the entry
        let (mut s, secrets) = (again, secrets);
        s.draft.provider_mut("anthropic").unwrap().api_key = None;
        s.save().unwrap();
        assert_eq!(secrets.get("ai.anthropic").unwrap(), None);
    }

    #[test]
    fn a_key_found_in_the_file_is_moved() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("ai.toml"), "[[providers]]\nid = \"openai\"\nkind = \"openai\"\napi_key = \"sk-old\"\n").unwrap();
        let secrets = Arc::new(MemorySecrets::default());
        let (s, w) = AiState::load(d.path().to_path_buf(), secrets.clone());
        assert!(w.is_none());
        assert_eq!(secrets.get("ai.openai").unwrap().as_deref(), Some("sk-old"));
        assert!(!std::fs::read_to_string(d.path().join("ai.toml")).unwrap().contains("sk-old"));
        assert_eq!(s.hub.config().provider("openai").unwrap().api_key().as_deref(), Some("sk-old"));
    }

    #[test]
    fn providers_can_be_added_and_removed_and_models_follow() {
        let d = tempfile::tempdir().unwrap();
        let (mut s, secrets) = state(d.path());
        s.choose_kind(ProviderKind::Ollama);
        assert_eq!(s.form.as_ref().unwrap().id, "ollama-2");
        s.add_form_provider();
        assert_eq!(s.screen, AiScreen::Edit(String::from("ollama-2")));
        s.set_scope_model("markdown", "ollama-2:llama3.2");
        s.set_default_model("ollama-2:llama3.2");
        s.draft.provider_mut("anthropic").unwrap().api_key = Some("k".into());
        s.save().unwrap();
        s.remove_provider("ollama-2");
        s.remove_provider("anthropic");
        assert!(s.draft.models.is_empty() && s.draft.default_model.is_none());
        s.save().unwrap();
        assert_eq!(secrets.get("ai.anthropic").unwrap(), None, "a removed provider's key is deleted");
        assert!(s.hub.config().provider("ollama-2").is_none());
    }

    #[test]
    fn choices_list_usable_providers_only() {
        let d = tempfile::tempdir().unwrap();
        let (mut s, _) = state(d.path());
        let none = s.choices(&s.draft);
        assert!(none.iter().all(|c| !c.value.starts_with("anthropic:")), "no key, no models");
        s.draft.provider_mut("anthropic").unwrap().api_key = Some("k".into());
        let some = s.choices(&s.draft);
        assert!(some.iter().any(|c| c.value.starts_with("anthropic:") && c.label.starts_with("Anthropic - ")));
    }

    /// A keychain that refuses everything and counts the attempts.
    #[derive(Default)]
    struct Locked(std::sync::atomic::AtomicUsize);

    impl SecretStore for Locked {
        fn get(&self, _: &str) -> Result<Option<String>, kd_store::SecretError> {
            Err(kd_store::SecretError::Unavailable("locked".into()))
        }
        fn set(&self, _: &str, _: &str) -> Result<(), kd_store::SecretError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(kd_store::SecretError::Unavailable("locked".into()))
        }
        fn delete(&self, _: &str) -> Result<(), kd_store::SecretError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(kd_store::SecretError::Unavailable("locked".into()))
        }
    }

    #[test]
    fn a_locked_keychain_keeps_the_key_in_memory_and_is_only_asked_when_needed() {
        let d = tempfile::tempdir().unwrap();
        let locked = Arc::new(Locked::default());
        let (mut s, w) = AiState::load(d.path().to_path_buf(), locked.clone());
        assert!(w.is_none());
        // changing something that is not a key never touches the keychain
        s.set_default_model("ollama:llama3.2");
        s.save().unwrap();
        assert_eq!(locked.0.load(std::sync::atomic::Ordering::SeqCst), 0);
        // a key the keychain refuses still works in this session, but is not on disk
        s.draft.provider_mut("openai").unwrap().api_key = Some("sk-mem".into());
        let err = s.save().unwrap_err();
        assert!(err.contains("locked"));
        assert_eq!(s.hub.config().provider("openai").unwrap().api_key().as_deref(), Some("sk-mem"));
        assert!(!std::fs::read_to_string(d.path().join("ai.toml")).unwrap().contains("sk-mem"));
        // the next Apply tries again
        s.set_default_model("ollama:other");
        assert!(s.save().is_err());
        assert_eq!(locked.0.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn nothing_is_ready_without_a_provider() {
        let s = AiState::empty();
        // environment keys could make a provider usable on a developer machine
        if std::env::vars().any(|(k, _)| k.ends_with("_API_KEY")) {
            return;
        }
        assert!(!s.ready());
    }
}
