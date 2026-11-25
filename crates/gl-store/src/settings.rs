//! User settings. `Settings` is what is saved; the Settings page edits a draft copy and saves
//! it with Apply.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Light, dark, or whatever the operating system is set to (macOS: System Settings › Appearance,
/// where Auto switches at sunset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    #[default]
    #[serde(alias = "auto")]
    System,
    Light,
    Dark,
}

impl Appearance {
    pub const ALL: [Appearance; 3] = [Appearance::System, Appearance::Light, Appearance::Dark];
    pub fn label(self) -> &'static str {
        match self {
            Appearance::System => "System",
            Appearance::Light => "Light",
            Appearance::Dark => "Dark",
        }
    }
}

/// A colour theme, in a light and a dark version (the palettes follow the VS Code / VSCodium
/// themes of the same names). You choose one for light and one for dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeFamily {
    #[default]
    Modern,
    Solarized,
    Plus,
}

impl ThemeFamily {
    pub const ALL: [ThemeFamily; 3] = [ThemeFamily::Modern, ThemeFamily::Solarized, ThemeFamily::Plus];
    /// The `data-theme` value of the light version (see glyphra.css).
    pub fn light_id(self) -> &'static str {
        match self {
            ThemeFamily::Modern => "light-modern",
            ThemeFamily::Solarized => "solarized-light",
            ThemeFamily::Plus => "light-plus",
        }
    }
    pub fn dark_id(self) -> &'static str {
        match self {
            ThemeFamily::Modern => "dark-modern",
            ThemeFamily::Solarized => "solarized-dark",
            ThemeFamily::Plus => "dark-plus",
        }
    }
    pub fn light_label(self) -> &'static str {
        match self {
            ThemeFamily::Modern => "Modern Light",
            ThemeFamily::Solarized => "Solarized Light",
            ThemeFamily::Plus => "Light+",
        }
    }
    pub fn dark_label(self) -> &'static str {
        match self {
            ThemeFamily::Modern => "Modern Dark",
            ThemeFamily::Solarized => "Solarized Dark",
            ThemeFamily::Plus => "Dark+",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenIn {
    #[default]
    OwnTab,
    NewWindow,
}

impl OpenIn {
    pub const ALL: [OpenIn; 2] = [OpenIn::OwnTab, OpenIn::NewWindow];
    pub fn label(self) -> &'static str {
        match self {
            OpenIn::OwnTab => "In its own tab",
            OpenIn::NewWindow => "In a new window",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellingDetection {
    #[default]
    Auto,
    Off,
}

impl SpellingDetection {
    pub const ALL: [SpellingDetection; 2] = [SpellingDetection::Auto, SpellingDetection::Off];
    pub fn label(self) -> &'static str {
        match self {
            SpellingDetection::Auto => "Auto",
            SpellingDetection::Off => "Off",
        }
    }
}

/// A connected AI model: provider, endpoint and model name. The API key is not here; it is in
/// the settings file, encrypted, under `ai.provider.<id>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelProfile {
    /// Stable id (`m1`, `m2` …).
    pub id: String,
    /// Provider id from the catalogue (`anthropic`) or the name of a custom provider.
    pub provider: String,
    pub base_url: String,
    pub model: String,
    /// What the list and the pickers show, e.g. "Anthropic - Claude Sonnet 5.5".
    pub name: String,
}

/// A provider the user added by hand (any OpenAI-compatible server).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomProvider {
    pub name: String,
    pub base_url: String,
    /// Where it is based: `eu`, `apac`, `us` or `local` (the default).
    #[serde(default = "default_region")]
    pub region: String,
    /// The API it speaks: `openai` (the default), `anthropic` or `gemini`.
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_region() -> String {
    "local".into()
}

fn default_protocol() -> String {
    "openai".into()
}

/// The tasks a model can be assigned to (Settings › Model Assignment), in display order.
pub const AI_TASKS: [(&str, &str); 8] = [
    ("chat", "AI Chat"),
    ("code", "Code"),
    ("summarization", "Summarization"),
    ("translation", "Translation"),
    ("analysis", "Text Analysis"),
    ("image", "Image generation"),
    ("ocr", "OCR"),
    ("vision", "Vision"),
];

pub const DEFAULT_MCP_CONFIG: &str = "{\n  \"mcpServers\": {}\n}";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub user_name: String,
    /// Interface language as a BCP 47 tag.
    pub language: String,
    pub appearance: Appearance,
    /// The theme used when the appearance is light.
    pub light_theme: ThemeFamily,
    /// The theme used when the appearance is dark.
    pub dark_theme: ThemeFamily,
    pub open_file_in: OpenIn,
    pub spelling: SpellingDetection,
    /// Units, date, time, number and currency formats.
    pub region: crate::region::RegionSettings,
    /// Applied after a restart.
    pub gpu_acceleration: bool,
    /// Applied after a restart.
    pub ai_enabled: bool,
    pub ai_panel_open: bool,
    /// Where new documents are saved.
    pub documents_dir: PathBuf,
    pub models: Vec<ModelProfile>,
    pub custom_providers: Vec<CustomProvider>,
    /// Profile id used when a task has no model of its own.
    pub default_model: Option<String>,
    /// Task id → profile id.
    pub assignments: std::collections::BTreeMap<String, String>,
    /// The MCP servers JSON, as typed.
    pub mcp_config: String,
    /// API keys, encrypted (`v1:…`), by name: `ai.provider.<id>`. See `secrets::Vault`.
    pub secrets: std::collections::BTreeMap<String, String>,
}

impl Settings {
    /// What the page needs: `data-theme`, `data-light` and `data-dark`. With Auto, the operating
    /// system decides between the last two (in CSS), live.
    pub fn theme_attrs(&self) -> (&'static str, &'static str, &'static str) {
        let (l, d) = (self.light_theme.light_id(), self.dark_theme.dark_id());
        match self.appearance {
            Appearance::Light => (l, l, d),
            Appearance::Dark => (d, l, d),
            Appearance::System => ("auto", l, d),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            user_name: String::new(),
            language: "en-US".into(),
            appearance: Appearance::System,
            light_theme: ThemeFamily::Modern,
            dark_theme: ThemeFamily::Modern,
            open_file_in: OpenIn::OwnTab,
            spelling: SpellingDetection::Auto,
            region: Default::default(),
            gpu_acceleration: true,
            ai_enabled: true,
            ai_panel_open: true,
            documents_dir: PathBuf::new(),
            models: Vec::new(),
            custom_providers: Vec::new(),
            default_model: None,
            assignments: Default::default(),
            mcp_config: DEFAULT_MCP_CONFIG.into(),
            secrets: Default::default(),
        }
    }
}

impl Settings {
    /// The profile with this id.
    pub fn model(&self, id: &str) -> Option<&ModelProfile> {
        self.models.iter().find(|m| m.id == id)
    }

    /// The model for a task: its own assignment, else the default, else the first model.
    pub fn model_for(&self, task: &str) -> Option<&ModelProfile> {
        self.assignments
            .get(task)
            .and_then(|id| self.model(id))
            .or_else(|| self.default_model.as_deref().and_then(|id| self.model(id)))
            .or_else(|| self.models.first())
    }

    /// Next free profile id.
    pub fn next_model_id(&self) -> String {
        (1u32..).map(|n| format!("m{n}")).find(|id| self.model(id).is_none()).unwrap_or_default()
    }

    /// Remove a profile and every assignment that pointed at it.
    pub fn remove_model(&mut self, id: &str) {
        self.models.retain(|m| m.id != id);
        self.assignments.retain(|_, v| v != id);
        if self.default_model.as_deref() == Some(id) {
            self.default_model = self.models.first().map(|m| m.id.clone());
        }
    }
}

/// Check the MCP configuration: JSON with an object `mcpServers`, each server an object.
pub fn validate_mcp(text: &str) -> Result<(), String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}"))?;
    let servers = v.get("mcpServers").ok_or("Missing \"mcpServers\"")?;
    let map = servers.as_object().ok_or("\"mcpServers\" must be an object")?;
    for (name, s) in map {
        if !s.is_object() {
            return Err(format!("Server \"{name}\" must be an object"));
        }
    }
    Ok(())
}

/// The server names in a valid MCP configuration, sorted by name; empty when it is not valid.
pub fn mcp_server_names(text: &str) -> Vec<String> {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(v) => v.get("mcpServers").and_then(|s| s.as_object()).map(|m| m.keys().cloned().collect()).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(id: &str) -> ModelProfile {
        ModelProfile { id: id.into(), provider: "anthropic".into(), base_url: String::new(), model: "x".into(), name: id.into() }
    }

    #[test]
    fn model_for_falls_back() {
        let mut s = Settings { models: vec![profile("m1"), profile("m2")], ..Default::default() };
        assert_eq!(s.model_for("code").map(|m| m.id.as_str()), Some("m1"));
        s.default_model = Some("m2".into());
        assert_eq!(s.model_for("code").map(|m| m.id.as_str()), Some("m2"));
        s.assignments.insert("code".into(), "m1".into());
        assert_eq!(s.model_for("code").map(|m| m.id.as_str()), Some("m1"));
    }

    #[test]
    fn removing_a_model_clears_assignments() {
        let mut s = Settings { models: vec![profile("m1"), profile("m2")], default_model: Some("m1".into()), ..Default::default() };
        s.assignments.insert("ocr".into(), "m1".into());
        s.remove_model("m1");
        assert!(s.assignments.is_empty());
        assert_eq!(s.default_model.as_deref(), Some("m2"));
        assert_eq!(s.next_model_id(), "m1");
    }

    #[test]
    fn mcp_validation() {
        assert!(validate_mcp(DEFAULT_MCP_CONFIG).is_ok());
        assert!(validate_mcp(r#"{"mcpServers":{"a":{"command":"x"}}}"#).is_ok());
        assert!(validate_mcp("{").unwrap_err().starts_with("Invalid JSON"));
        assert!(validate_mcp("{}").is_err());
        assert!(validate_mcp(r#"{"mcpServers":{"a":1}}"#).is_err());
    }

    #[test]
    fn old_files_without_new_fields_still_load() {
        let s: Settings = serde_json::from_str(r#"{"user_name":"Kris"}"#).unwrap();
        assert_eq!(s.user_name, "Kris");
        assert!(s.ai_enabled);
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn light_and_dark_themes_are_chosen_separately_and_auto_is_the_default() {
        let mut s = Settings::default();
        assert_eq!(s.appearance, Appearance::System);
        assert_eq!(s.theme_attrs(), ("auto", "light-modern", "dark-modern"));
        s.light_theme = ThemeFamily::Solarized;
        s.dark_theme = ThemeFamily::Plus;
        assert_eq!(s.theme_attrs(), ("auto", "solarized-light", "dark-plus"));
        s.appearance = Appearance::Dark;
        assert_eq!(s.theme_attrs().0, "dark-plus");
        s.appearance = Appearance::Light;
        assert_eq!(s.theme_attrs().0, "solarized-light");
        let ids: std::collections::HashSet<_> = ThemeFamily::ALL.iter().flat_map(|f| [f.light_id(), f.dark_id()]).collect();
        assert_eq!(ids.len(), 6);
        let back: Settings = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!((back.appearance, back.light_theme, back.dark_theme), (Appearance::Light, ThemeFamily::Solarized, ThemeFamily::Plus));
    }
}
