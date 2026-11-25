//! The AI providers offered in Settings › AI Models, grouped by region, with their endpoints and
//! suggested models. The list is data, not code: `data/catalog.json` is built in. The models a
//! provider really offers are asked from the provider itself once you are connected (see
//! `models_api`); the built-in models are the suggestions shown until then, with their prices.
//! Providers with no list (your own endpoint) take a typed model name.

use std::sync::OnceLock;

use crate::models_api::ModelsApi;
use serde::Deserialize;

/// Where a provider is based, for grouping in Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Eu,
    Apac,
    Us,
    Local,
}

impl Region {
    pub const ALL: [Region; 4] = [Region::Eu, Region::Apac, Region::Us, Region::Local];

    pub fn label(self) -> &'static str {
        match self {
            Region::Eu => "Europe (EU)",
            Region::Apac => "Asia-Pacific (APAC)",
            Region::Us => "North America (US/CA)",
            Region::Local => "Local & custom",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Region::Eu => "eu",
            Region::Apac => "apac",
            Region::Us => "us",
            Region::Local => "local",
        }
    }

    pub fn from_id(id: &str) -> Option<Region> {
        Region::ALL.into_iter().find(|r| r.id() == id)
    }
}

/// The wire protocol a provider speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Protocol {
    #[serde(rename = "openai")]
    OpenAiChat,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "deepl")]
    DeepL,
    #[serde(rename = "bfl")]
    Bfl,
}

impl Protocol {
    /// The protocols a custom provider can speak.
    pub const CHAT: [Protocol; 3] = [Protocol::OpenAiChat, Protocol::Anthropic, Protocol::Gemini];

    pub fn label(self) -> &'static str {
        match self {
            Protocol::OpenAiChat => "OpenAI Chat Completions",
            Protocol::Anthropic => "Anthropic Messages",
            Protocol::Gemini => "Gemini generateContent",
            Protocol::DeepL => "DeepL",
            Protocol::Bfl => "FLUX (Black Forest Labs)",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Protocol::OpenAiChat => "openai",
            Protocol::Anthropic => "anthropic",
            Protocol::Gemini => "gemini",
            Protocol::DeepL => "deepl",
            Protocol::Bfl => "bfl",
        }
    }

    pub fn from_id(id: &str) -> Option<Protocol> {
        Protocol::CHAT.into_iter().find(|p| p.id() == id)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Provider {
    pub id: String,
    /// The name in the lists.
    pub name: String,
    pub region: Region,
    /// ISO 3166 country code of the company's home (empty for local servers).
    #[serde(default)]
    pub country: String,
    pub protocol: Protocol,
    /// Empty when you enter your own (Aleph Alpha, Microsoft).
    #[serde(default)]
    pub base_url: String,
    /// An example endpoint, shown when you have to enter your own.
    #[serde(default)]
    pub endpoint_hint: String,
    /// A key is required (local servers and your own endpoint may take one, but need none).
    pub needs_key: bool,
    /// One line on what the provider offers.
    pub blurb: String,
    /// Where to get a key, or how to start the server.
    pub help: String,
    /// Where the provider lists its models (none: nothing to ask).
    #[serde(default)]
    pub models_api: Option<ModelsApi>,
    #[serde(default)]
    pub models: Vec<Model>,
}

/// A model of the catalogue, with what it costs: US dollars per million tokens.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub input: Option<f32>,
    #[serde(default)]
    pub output: Option<f32>,
    /// Replaces the price when it is not per token (images, characters) or not published.
    #[serde(default)]
    pub note: String,
    /// Can hold a conversation (not only images, speech or translation).
    #[serde(default = "yes")]
    pub chat: bool,
    #[serde(default)]
    pub recommended: bool,
}

fn yes() -> bool {
    true
}

impl Model {
    /// What it costs to use, for sorting: the output price (what long answers cost).
    pub fn cost(&self) -> Option<f32> {
        self.output.or(self.input)
    }

    /// "$3 in / $15 out per 1M tokens", or the note.
    pub fn price_label(&self) -> String {
        match (self.input, self.output) {
            (Some(i), Some(o)) => format!("${} in / ${} out per 1M tokens", money(i), money(o)),
            _ if !self.note.is_empty() => self.note.clone(),
            _ => "price not published".to_string(),
        }
    }

    /// "$3 / $15", or the note: the price next to the name in a list.
    pub fn price_short(&self) -> String {
        match (self.input, self.output) {
            (Some(i), Some(o)) => format!("${} / ${}", money(i), money(o)),
            _ => self.note.clone(),
        }
    }

    /// "Claude Sonnet 5.5 · $3 / $15", for lists.
    pub fn label(&self) -> String {
        match (self.input, self.output) {
            (Some(i), Some(o)) => format!("{} · ${} / ${}", self.name, money(i), money(o)),
            _ if !self.note.is_empty() => format!("{} · {}", self.name, self.note),
            _ => self.name.clone(),
        }
    }
}

fn money(v: f32) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

impl Provider {
    /// You must enter the endpoint (it belongs to your account or install).
    pub fn needs_endpoint(&self) -> bool {
        self.base_url.is_empty()
    }

    /// The chat models, cheapest first (those without a price last).
    pub fn chat_models_by_cost(&self) -> Vec<&Model> {
        let mut v: Vec<_> = self.models.iter().filter(|m| m.chat).collect();
        v.sort_by(|a, b| a.cost().unwrap_or(f32::MAX).total_cmp(&b.cost().unwrap_or(f32::MAX)));
        v
    }
}

/// The providers and models, as of a date.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Catalog {
    /// `YYYY-MM-DD`, when the prices were checked.
    pub updated: String,
    pub providers: Vec<Provider>,
}

const BUILT_IN: &str = include_str!("../data/catalog.json");

static CURRENT: OnceLock<Catalog> = OnceLock::new();

/// The catalogue shipped with this version.
pub fn built_in() -> Catalog {
    // The built-in file is checked by a test, so this cannot fail in a released build.
    serde_json::from_str(BUILT_IN).unwrap_or(Catalog { updated: String::new(), providers: Vec::new() })
}

/// The catalogue in use.
pub fn current() -> &'static Catalog {
    CURRENT.get_or_init(built_in)
}

pub fn all() -> &'static [Provider] {
    &current().providers
}

pub fn get(id: &str) -> Option<&'static Provider> {
    all().iter().find(|p| p.id == id)
}

/// The catalogue providers in `region`, in the order Settings shows them.
pub fn in_region(region: Region) -> impl Iterator<Item = &'static Provider> {
    all().iter().filter(move |p| p.region == region)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_in_catalogue_is_valid_and_complete() {
        let c = built_in();
        assert_eq!(c.updated.len(), 10);
        assert_eq!(c.providers.len(), 19);
        let mut ids = std::collections::HashSet::new();
        for p in &c.providers {
            assert!(ids.insert(p.id.as_str()), "{} twice", p.id);
            assert!(!p.name.is_empty() && !p.blurb.is_empty() && !p.help.is_empty(), "{}", p.id);
            assert!(p.base_url.is_empty() == !p.endpoint_hint.is_empty(), "{}: endpoint and hint go together", p.id);
            assert!(p.base_url.is_empty() || p.base_url.starts_with("http"), "{}", p.id);
            assert_eq!(p.country.is_empty(), p.region == Region::Local, "{}", p.id);
            assert!(p.models_api.is_some(), "{} has no models endpoint", p.id);
        }
        assert!(get("anthropic").is_some_and(|p| p.models.iter().any(|m| m.id == "claude-fable-5-2")));
        assert!(get("microsoft").is_some_and(|p| p.needs_endpoint()));
        assert!(!get("ollama").is_some_and(|p| p.needs_key));
    }

    #[test]
    fn every_endpoint_of_the_catalogue_builds_a_request() {
        for p in all() {
            let api = p.models_api.as_ref().unwrap();
            let base = if p.base_url.is_empty() { "https://example.openai.azure.com/openai/v1" } else { &p.base_url };
            let r = api.request(base, "k").unwrap_or_else(|e| panic!("{}: {e}", p.id));
            assert!(r.url.starts_with("https://") || r.url.starts_with("http://localhost"), "{}: {}", p.id, r.url);
        }
    }

    #[test]
    fn every_region_has_providers_and_models_sort_by_cost() {
        for r in Region::ALL {
            assert!(in_region(r).next().is_some(), "{r:?}");
        }
        let order = get("anthropic").map(|p| p.chat_models_by_cost()).unwrap_or_default();
        assert!(order.len() >= 3);
        assert!(order.windows(2).all(|w| w[0].cost().unwrap_or(f32::MAX) <= w[1].cost().unwrap_or(f32::MAX)));
        assert_eq!(money(3.0), "3");
        assert_eq!(money(0.25), "0.25");
        assert_eq!(Region::from_id("apac"), Some(Region::Apac));
        assert_eq!(Protocol::from_id("gemini"), Some(Protocol::Gemini));
    }
}
