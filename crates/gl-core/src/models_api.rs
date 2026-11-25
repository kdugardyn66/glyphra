//! Asking a provider which models it offers: every provider has a "list models" endpoint, and
//! the catalogue says where it is, how the key is sent and how the answer is laid out.
//!
//! Glyphra builds the request here ([`ModelsApi::request`]); the shell sends it and hands the
//! text of the answer back, which [`parse`] reads. The key goes only to the address of the
//! provider (or the endpoint you entered for it), and redirects are never followed.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// How the API key is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Auth {
    /// `Authorization: Bearer <key>`
    #[default]
    Bearer,
    /// `<auth_name>: <auth_prefix><key>`, e.g. `x-api-key: <key>` or `api-key: <key>`
    Header,
    /// `?<auth_name>=<key>`
    Query,
    /// No key is sent.
    None,
}

/// How the answer is laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    /// `{"data": [{"id": …}]}` (OpenAI and the many providers that copy it)
    #[default]
    Openai,
    /// `{"data": [{"id": …, "display_name": …}]}`
    Anthropic,
    /// `{"models": [{"name": "models/…", "displayName": …, "supportedGenerationMethods": […]}]}`
    Google,
    /// `{"models": [{"name": …, "endpoints": […]}]}`
    Cohere,
    /// `{"models": [{"name": …}]}`
    Ollama,
    /// A list, or an object with one list, of names or objects with an `id`, `name` or `model`.
    Generic,
    /// The endpoint lists something else (DeepL: languages): a good answer only proves the key.
    KeyCheck,
}

/// Where and how to ask one provider for its models.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModelsApi {
    /// `{base}` is the endpoint as entered; `{host}` is it without a trailing `/v1` or `/openai/v1`.
    pub url: String,
    #[serde(default)]
    pub auth: Auth,
    /// The header or query parameter that carries the key (for `header` and `query`).
    #[serde(default)]
    pub auth_name: String,
    /// Text before the key in the header (`DeepL-Auth-Key `).
    #[serde(default)]
    pub auth_prefix: String,
    /// Further headers, such as a version.
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub format: Format,
}

/// A request for the shell to send: GET `url` with `headers`, or POST them with `body` (JSON).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// A model a provider says it has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Found {
    pub id: String,
    pub name: String,
}

/// The most models taken from one answer.
const MAX_MODELS: usize = 1000;

/// The endpoint without a trailing slash or API version.
fn host(base: &str) -> String {
    let mut h = base.trim().trim_end_matches('/');
    for suffix in ["/openai/v1", "/v1beta", "/v1"] {
        if let Some(rest) = h.strip_suffix(suffix) {
            h = rest.trim_end_matches('/');
            break;
        }
    }
    h.to_string()
}

/// https, or http to this computer (local model servers).
fn address_ok(url: &str) -> bool {
    url.starts_with("https://")
        || ["http://localhost", "http://127.0.0.1", "http://[::1]"]
            .iter()
            .any(|p| url.strip_prefix(p).is_some_and(|r| r.is_empty() || r.starts_with([':', '/'])))
}

pub(crate) fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

impl ModelsApi {
    /// The endpoint of a provider you added yourself, from the API it speaks.
    pub fn for_protocol(protocol: crate::providers::Protocol) -> ModelsApi {
        use crate::providers::Protocol;
        let base = ModelsApi {
            url: "{host}/v1/models".into(),
            auth: Auth::Bearer,
            auth_name: String::new(),
            auth_prefix: String::new(),
            headers: Vec::new(),
            format: Format::Openai,
        };
        match protocol {
            Protocol::Anthropic => ModelsApi {
                url: "{host}/v1/models?limit=1000".into(),
                auth: Auth::Header,
                auth_name: "x-api-key".into(),
                headers: vec![("anthropic-version".into(), "2023-06-01".into())],
                format: Format::Anthropic,
                ..base
            },
            Protocol::Gemini => ModelsApi {
                url: "{host}/v1beta/models?pageSize=1000".into(),
                auth: Auth::Query,
                auth_name: "key".into(),
                format: Format::Google,
                ..base
            },
            _ => base,
        }
    }

    /// The request for a provider at `base_url` (the entered endpoint) with `key` (empty: none).
    pub fn request(&self, base_url: &str, key: &str) -> Result<Request, String> {
        let mut url = self.url.replace("{base}", base_url.trim().trim_end_matches('/')).replace("{host}", &host(base_url));
        if url.contains(['{', '<', ' ']) || !address_ok(&url) {
            return Err("There is no valid address to ask for the models.".into());
        }
        if key.chars().any(char::is_control) {
            return Err("The API key contains characters that cannot be sent.".into());
        }
        let mut headers = Vec::new();
        if !key.is_empty() {
            match self.auth {
                Auth::Bearer => headers.push(("Authorization".to_string(), format!("Bearer {key}"))),
                Auth::Header => headers.push((self.auth_name.clone(), format!("{}{key}", self.auth_prefix))),
                Auth::Query => {
                    url.push(if url.contains('?') { '&' } else { '?' });
                    url.push_str(&format!("{}={}", self.auth_name, percent_encode(key)));
                }
                Auth::None => {}
            }
        }
        headers.extend(self.headers.iter().cloned());
        Ok(Request { url, headers, body: None })
    }
}

/// Models that are not for conversation or images (they would only clutter the list).
fn is_other_kind(id: &str) -> bool {
    let id = id.to_ascii_lowercase();
    ["embed", "moderation", "whisper", "tts", "transcribe", "rerank"].iter().any(|w| id.contains(w))
}

fn text<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
}

fn list<'a>(root: &'a Value, keys: &[&str]) -> Option<&'a Vec<Value>> {
    root.as_array().or_else(|| keys.iter().find_map(|k| root.get(*k).and_then(Value::as_array)))
}

fn has(v: &Value, key: &str, what: &str) -> Option<bool> {
    v.get(key).and_then(Value::as_array).map(|a| a.iter().any(|x| x.as_str() == Some(what)))
}

/// Reads the answer of a models endpoint. `KeyCheck` answers give no models.
pub fn parse(format: Format, body: &str) -> Result<Vec<Found>, String> {
    let root: Value = serde_json::from_str(body).map_err(|_| "The answer was not a model list.".to_string())?;
    let mut out: Vec<Found> = Vec::new();
    let mut add = |id: &str, name: Option<&str>| {
        let id = id.trim();
        if !id.is_empty() && id.len() <= 200 && !out.iter().any(|f| f.id == id) && out.len() < MAX_MODELS {
            out.push(Found { id: id.to_string(), name: name.unwrap_or(id).to_string() });
        }
    };
    match format {
        Format::KeyCheck => {}
        Format::Openai | Format::Generic | Format::Ollama => {
            let items = list(&root, &["data", "models", "items", "result"]).ok_or("The answer has no list of models.")?;
            for it in items {
                let id = it.as_str().or_else(|| text(it, "id")).or_else(|| text(it, "name")).or_else(|| text(it, "model"));
                if let Some(id) = id.filter(|id| !is_other_kind(id)) {
                    add(id, None);
                }
            }
        }
        Format::Anthropic => {
            let items = list(&root, &["data"]).ok_or("The answer has no list of models.")?;
            for it in items {
                if let Some(id) = text(it, "id") {
                    add(id, text(it, "display_name"));
                }
            }
        }
        Format::Google => {
            let items = list(&root, &["models"]).ok_or("The answer has no list of models.")?;
            for it in items {
                let Some(name) = text(it, "name") else { continue };
                if has(it, "supportedGenerationMethods", "generateContent") == Some(false) || is_other_kind(name) {
                    continue;
                }
                add(name.strip_prefix("models/").unwrap_or(name), text(it, "displayName"));
            }
        }
        Format::Cohere => {
            let items = list(&root, &["models"]).ok_or("The answer has no list of models.")?;
            for it in items {
                if has(it, "endpoints", "chat") == Some(false) {
                    continue;
                }
                if let Some(name) = text(it, "name").filter(|n| !is_other_kind(n)) {
                    add(name, None);
                }
            }
        }
    }
    Ok(out)
}

/// A readable reason for a failed request.
pub fn explain(error: &str) -> String {
    if error.contains("status code 401") || error.contains("status code 403") {
        crate::i18n::t("The provider did not accept this API key.")
    } else if error.contains("status code 404") {
        crate::i18n::t("The provider has no model list at this address (404).")
    } else {
        crate::i18n::tr("Couldn't load the models from the provider: {error}", &[("error", error)])
    }
}

/// The file in the config folder where the lists of models last received are kept.
pub const FILE: &str = "models.json";

/// What was last received from each provider, to show when offline and at the next start.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Received {
    /// `YYYY-MM-DD`
    pub fetched: String,
    pub models: Vec<Found>,
}

/// Reads the kept lists (anything unreadable is dropped).
pub fn load_received(dir: &std::path::Path) -> std::collections::BTreeMap<String, Received> {
    std::fs::read_to_string(dir.join(FILE)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Protocol;

    fn api(url: &str, auth: Auth) -> ModelsApi {
        ModelsApi {
            url: url.into(),
            auth,
            auth_name: String::new(),
            auth_prefix: String::new(),
            headers: Vec::new(),
            format: Format::Openai,
        }
    }

    #[test]
    fn requests_follow_the_providers_documentation() {
        let r = api("https://api.openai.com/v1/models", Auth::Bearer).request("", "sk-1").unwrap();
        assert_eq!(r.url, "https://api.openai.com/v1/models");
        assert_eq!(r.headers, [("Authorization".to_string(), "Bearer sk-1".to_string())]);
        let a = ModelsApi {
            auth_name: "x-api-key".into(),
            headers: vec![("anthropic-version".into(), "2023-06-01".into())],
            ..api("https://api.anthropic.com/v1/models?limit=1000", Auth::Header)
        };
        let r = a.request("", "k").unwrap();
        assert_eq!(r.headers[0], ("x-api-key".to_string(), "k".to_string()));
        assert_eq!(r.headers[1].0, "anthropic-version");
        let d = ModelsApi {
            auth_name: "Authorization".into(),
            auth_prefix: "DeepL-Auth-Key ".into(),
            ..api("https://api.deepl.com/v2/languages?type=target", Auth::Header)
        };
        assert_eq!(d.request("", "abc:fx").unwrap().headers[0].1, "DeepL-Auth-Key abc:fx");
        let g = ModelsApi {
            auth_name: "key".into(),
            ..api("https://generativelanguage.googleapis.com/v1beta/models?pageSize=1000", Auth::Query)
        };
        assert_eq!(
            g.request("", "a b&c").unwrap().url,
            "https://generativelanguage.googleapis.com/v1beta/models?pageSize=1000&key=a%20b%26c"
        );
        let g = ModelsApi { auth_name: "key".into(), ..api("https://x.example/models", Auth::Query) };
        assert_eq!(g.request("", "k").unwrap().url, "https://x.example/models?key=k");
    }

    #[test]
    fn the_entered_endpoint_is_normalised_and_checked() {
        let az = ModelsApi { auth_name: "api-key".into(), ..api("{host}/openai/v1/models", Auth::Header) };
        for base in ["https://res.openai.azure.com", "https://res.openai.azure.com/", "https://res.openai.azure.com/openai/v1"] {
            assert_eq!(az.request(base, "k").unwrap().url, "https://res.openai.azure.com/openai/v1/models", "{base}");
        }
        let ollama = api("{host}/api/tags", Auth::None);
        assert_eq!(ollama.request("http://localhost:11434/v1", "").unwrap().url, "http://localhost:11434/api/tags");
        assert_eq!(
            api("{host}/v1/models", Auth::Bearer).request("http://localhost:8000/v1", "").unwrap().url,
            "http://localhost:8000/v1/models"
        );
        // no address yet, a placeholder, plain http to another machine, or a lookalike host
        assert!(az.request("", "k").is_err());
        assert!(az.request("https://<resource>.openai.azure.com/openai/v1", "k").is_err());
        assert!(api("{host}/v1/models", Auth::Bearer).request("http://example.com", "k").is_err());
        assert!(api("{host}/v1/models", Auth::Bearer).request("http://localhost.evil.example", "k").is_err());
        // no key: nothing is sent; a key with a line break is refused
        assert!(api("https://x.example/m", Auth::Bearer).request("", "").unwrap().headers.is_empty());
        assert!(api("https://x.example/m", Auth::Bearer).request("", "a\r\nX: y").is_err());
    }

    #[test]
    fn providers_you_add_use_the_api_they_speak() {
        let o = ModelsApi::for_protocol(Protocol::OpenAiChat).request("http://localhost:8000", "k").unwrap();
        assert_eq!(o.url, "http://localhost:8000/v1/models");
        let a = ModelsApi::for_protocol(Protocol::Anthropic).request("https://llm.corp.example/", "k").unwrap();
        assert_eq!((a.url.as_str(), a.headers[0].0.as_str()), ("https://llm.corp.example/v1/models?limit=1000", "x-api-key"));
        let g = ModelsApi::for_protocol(Protocol::Gemini).request("https://g.corp.example", "k").unwrap();
        assert!(g.url.ends_with("/v1beta/models?pageSize=1000&key=k"));
    }

    fn ids(v: Result<Vec<Found>, String>) -> Vec<String> {
        v.unwrap().into_iter().map(|f| f.id).collect()
    }

    #[test]
    fn answers_are_read_in_every_layout() {
        assert_eq!(
            ids(parse(
                Format::Openai,
                r#"{"object":"list","data":[{"id":"gpt-5"},{"id":"text-embedding-3-small"},{"id":"gpt-5"},{"id":"o4-mini"}]}"#
            )),
            ["gpt-5", "o4-mini"]
        );
        let a = parse(
            Format::Anthropic,
            r#"{"data":[{"id":"claude-sonnet-5-5","display_name":"Claude Sonnet 5.5","type":"model"}],"has_more":false}"#,
        )
        .unwrap();
        assert_eq!(a, [Found { id: "claude-sonnet-5-5".into(), name: "Claude Sonnet 5.5".into() }]);
        let g = parse(
            Format::Google,
            r#"{"models":[{"name":"models/gemini-3-pro","displayName":"Gemini 3 Pro","supportedGenerationMethods":["generateContent"]},{"name":"models/text-embedding-5","supportedGenerationMethods":["embedContent"]},{"name":"models/imagen-5","supportedGenerationMethods":["predict"]}]}"#,
        )
        .unwrap();
        assert_eq!(g, [Found { id: "gemini-3-pro".into(), name: "Gemini 3 Pro".into() }]);
        assert_eq!(
            ids(parse(
                Format::Cohere,
                r#"{"models":[{"name":"command-a","endpoints":["chat","generate"]},{"name":"embed-v4","endpoints":["embed"]}]}"#
            )),
            ["command-a"]
        );
        assert_eq!(
            ids(parse(Format::Ollama, r#"{"models":[{"name":"llama3.3:70b","size":1},{"name":"nomic-embed-text:latest"}]}"#)),
            ["llama3.3:70b"]
        );
        assert_eq!(
            ids(parse(Format::Generic, r#"[{"name":"luminous-1"},"pharia-1-7b",{"model":"x"}]"#)),
            ["luminous-1", "pharia-1-7b", "x"]
        );
        assert_eq!(ids(parse(Format::Generic, r#"{"models":[{"id":"flux-pro-2"}]}"#)), ["flux-pro-2"]);
        // DeepL lists languages: a good answer only proves the key
        assert!(parse(Format::KeyCheck, r#"[{"language":"NL","name":"Dutch"}]"#).unwrap().is_empty());
    }

    #[test]
    fn bad_answers_are_errors_not_empty_lists() {
        assert!(parse(Format::Openai, "<html>").is_err());
        assert!(parse(Format::Openai, r#"{"error":"nope"}"#).is_err());
        assert!(parse(Format::KeyCheck, "nope").is_err());
        let many = format!(r#"{{"data":[{}]}}"#, (0..1200).map(|i| format!(r#"{{"id":"m{i}"}}"#)).collect::<Vec<_>>().join(","));
        assert_eq!(parse(Format::Openai, &many).unwrap().len(), MAX_MODELS);
    }

    #[test]
    fn failures_are_explained() {
        assert!(explain("https://x: status code 401").contains("API key"));
        assert!(explain("status code 404").contains("404"));
        assert!(explain("connection refused").contains("connection refused"));
    }
}
