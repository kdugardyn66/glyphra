//! The interface languages. Text in the code is English and is its own key: `t("Save")` looks
//! "Save" up in the catalogue of the chosen language (`i18n/<tag>.json`, a JSON object of English
//! text → translation) and falls back to the English text itself, so a missing translation shows
//! English, never a blank. Placeholders are written `{name}` and filled by [`tr`].
//!
//! Regional variants only hold what differs: `en-GB` is English with British spelling, and
//! `es-ES`, `pt-PT` and `fr-CA` hold their differences from `es-419`, `pt-BR` and `fr-FR`.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// The interface languages: BCP 47 tag and the name in that language (as language pickers show it).
pub const LANGUAGES: [(&str, &str); 11] = [
    ("en-US", "English (United States)"),
    ("en-GB", "English (United Kingdom)"),
    ("es-419", "Español (Latinoamérica)"),
    ("es-ES", "Español (España)"),
    ("pt-BR", "Português (Brasil)"),
    ("pt-PT", "Português (Portugal)"),
    ("fr-CA", "Français (Canada)"),
    ("fr-FR", "Français (France)"),
    ("de-DE", "Deutsch"),
    ("it-IT", "Italiano"),
    ("nl-NL", "Nederlands"),
];

pub const DEFAULT_LANGUAGE: &str = "en-US";

const CATALOGS: [(&str, &str); 10] = [
    ("en-GB", include_str!("../i18n/en-GB.json")),
    ("es-419", include_str!("../i18n/es-419.json")),
    ("es-ES", include_str!("../i18n/es-ES.json")),
    ("pt-BR", include_str!("../i18n/pt-BR.json")),
    ("pt-PT", include_str!("../i18n/pt-PT.json")),
    ("fr-CA", include_str!("../i18n/fr-CA.json")),
    ("fr-FR", include_str!("../i18n/fr-FR.json")),
    ("de-DE", include_str!("../i18n/de-DE.json")),
    ("it-IT", include_str!("../i18n/it-IT.json")),
    ("nl-NL", include_str!("../i18n/nl-NL.json")),
];

type Catalog = HashMap<String, String>;

fn catalog(tag: &str) -> Option<&'static Catalog> {
    static ALL: OnceLock<HashMap<&'static str, Catalog>> = OnceLock::new();
    ALL.get_or_init(|| CATALOGS.iter().map(|(t, text)| (*t, serde_json::from_str(text).unwrap_or_default())).collect()).get(tag)
}

/// The language a regional variant falls back to before English.
fn parent(tag: &str) -> Option<&'static str> {
    match tag {
        "es-ES" => Some("es-419"),
        "pt-PT" => Some("pt-BR"),
        "fr-CA" => Some("fr-FR"),
        _ => None,
    }
}

/// `key` in the language `tag`, or `key` itself.
pub fn translate(tag: &str, key: &str) -> String {
    let mut tag = Some(tag);
    while let Some(t) = tag {
        if let Some(v) = catalog(t).and_then(|c| c.get(key)).filter(|v| !v.is_empty()) {
            return v.clone();
        }
        tag = parent(t);
    }
    key.to_string()
}

static CURRENT: RwLock<Option<&'static str>> = RwLock::new(None);

/// Use `tag` (an unknown tag means English (United States)).
pub fn set_language(tag: &str) {
    let tag = LANGUAGES.iter().map(|(t, _)| *t).find(|t| *t == tag).unwrap_or(DEFAULT_LANGUAGE);
    if let Ok(mut g) = CURRENT.write() {
        *g = Some(tag);
    }
}

pub fn language() -> &'static str {
    CURRENT.read().ok().and_then(|g| *g).unwrap_or(DEFAULT_LANGUAGE)
}

/// Marks an English text in a `const` or `static` for translation: it stays as it is there, and
/// is translated with `t` where it is shown. (The catalogue test finds these like `t("…")`.)
pub const fn tx(s: &'static str) -> &'static str {
    s
}

/// Translate an interface text into the language in use.
pub fn t(key: &str) -> String {
    translate(language(), key)
}

/// Translate and fill `{name}` placeholders.
pub fn tr(key: &str, args: &[(&str, &str)]) -> String {
    fill(translate(language(), key), args)
}

fn fill(mut s: String, args: &[(&str, &str)]) -> String {
    for (name, value) in args {
        s = s.replace(&format!("{{{name}}}"), value);
    }
    s
}

/// "3 hours ago", in the language in use.
pub fn time_ago(then: u64, now: u64) -> String {
    time_ago_in(language(), then, now)
}

pub fn time_ago_in(tag: &str, then: u64, now: u64) -> String {
    let secs = now.saturating_sub(then);
    let (n, one, many) = match secs {
        0..=59 => return translate(tag, "just now"),
        60..=3599 => (secs / 60, "{n} minute ago", "{n} minutes ago"),
        3600..=86_399 => (secs / 3600, "{n} hour ago", "{n} hours ago"),
        86_400..=2_591_999 => (secs / 86_400, "{n} day ago", "{n} days ago"),
        2_592_000..=31_535_999 => (secs / 2_592_000, "{n} month ago", "{n} months ago"),
        _ => (secs / 31_536_000, "{n} year ago", "{n} years ago"),
    };
    fill(translate(tag, if n == 1 { one } else { many }), &[("n", &n.to_string())])
}

/// The supported language closest to a system locale such as `es_MX`, `pt-BR` or `fr_BE.UTF-8`.
pub fn nearest_language(locale: &str) -> &'static str {
    let clean = locale.split(['.', '@']).next().unwrap_or("").replace('_', "-");
    let mut parts = clean.split('-');
    let lang = parts.next().unwrap_or("").to_lowercase();
    let region = parts.find(|p| p.len() == 2 || p.len() == 3).unwrap_or("").to_uppercase();
    let pick = |tag: &'static str| LANGUAGES.iter().map(|(t, _)| *t).find(|t| *t == tag);
    let exact = format!("{lang}-{region}");
    if let Some(t) = LANGUAGES.iter().map(|(t, _)| *t).find(|t| *t == exact) {
        return t;
    }
    match (lang.as_str(), region.as_str()) {
        ("en", "US" | "") => "en-US",
        ("en", _) => "en-GB",
        ("es", "ES") => "es-ES",
        ("es", _) => "es-419",
        ("pt", "PT" | "AO" | "MZ") => "pt-PT",
        ("pt", _) => "pt-BR",
        ("fr", "CA") => "fr-CA",
        ("fr", _) => "fr-FR",
        ("de", _) => pick("de-DE").unwrap_or(DEFAULT_LANGUAGE),
        ("it", _) => "it-IT",
        ("nl", _) => "nl-NL",
        _ => DEFAULT_LANGUAGE,
    }
}

/// The language of the operating system, as far as it is supported.
pub fn system_language() -> &'static str {
    sys_locale::get_locale().map(|l| nearest_language(&l)).unwrap_or(DEFAULT_LANGUAGE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholders(s: &str) -> Vec<String> {
        let mut v: Vec<String> = s.split('{').skip(1).filter_map(|x| x.split('}').next()).map(str::to_string).collect();
        v.sort();
        v
    }

    #[test]
    fn full_catalogues_share_one_key_set_and_keep_placeholders() {
        let full = ["es-419", "pt-BR", "fr-FR", "de-DE", "it-IT", "nl-NL"];
        let reference = catalog("es-419").map(|c| c.keys().cloned().collect::<std::collections::BTreeSet<_>>()).unwrap_or_default();
        assert!(reference.len() > 200);
        for tag in full {
            let Some(c) = catalog(tag) else { panic!("{tag} missing") };
            let keys: std::collections::BTreeSet<_> = c.keys().cloned().collect();
            assert_eq!(keys, reference, "{tag} differs from es-419");
            for (k, v) in c {
                assert!(!v.trim().is_empty(), "{tag}: empty translation for {k}");
                assert_eq!(placeholders(k), placeholders(v), "{tag}: placeholders differ in {k}");
            }
        }
        for tag in ["es-ES", "pt-PT", "fr-CA", "en-GB"] {
            let Some(c) = catalog(tag) else { panic!("{tag} missing") };
            for (k, v) in c {
                assert!(reference.contains(k), "{tag}: override for unknown key {k}");
                assert_eq!(placeholders(k), placeholders(v), "{tag}: placeholders differ in {k}");
            }
        }
    }

    #[test]
    fn regional_overrides_win_over_the_parent() {
        assert_eq!(translate("pt-PT", "File"), "Ficheiro");
        assert_eq!(translate("pt-PT", "Cancel"), translate("pt-BR", "Cancel"));
        assert_eq!(translate("fr-CA", "Cancel"), translate("fr-FR", "Cancel"));
        assert_eq!(translate("de-DE", "Cancel"), "Abbrechen");
    }

    #[test]
    fn falls_back_through_the_parent_to_english() {
        // no catalogue content needed: unknown keys come back unchanged in every language
        for (tag, _) in LANGUAGES {
            assert_eq!(translate(tag, "A text no one translated"), "A text no one translated");
        }
        assert_eq!(parent("fr-CA"), Some("fr-FR"));
        assert_eq!(parent("de-DE"), None);
        assert_eq!(fill("{a} and {b}".into(), &[("a", "1"), ("b", "2")]), "1 and 2");
    }

    #[test]
    fn system_locales_map_to_the_nearest_supported_language() {
        for (input, want) in [
            ("en_US.UTF-8", "en-US"),
            ("en-AU", "en-GB"),
            ("en", "en-US"),
            ("es_MX", "es-419"),
            ("es-ES", "es-ES"),
            ("es", "es-419"),
            ("pt_BR.UTF-8", "pt-BR"),
            ("pt-PT", "pt-PT"),
            ("fr_CA", "fr-CA"),
            ("fr-BE", "fr-FR"),
            ("de_AT", "de-DE"),
            ("it", "it-IT"),
            ("nl-BE", "nl-NL"),
            ("ja-JP", "en-US"),
            ("", "en-US"),
        ] {
            assert_eq!(nearest_language(input), want, "{input}");
        }
    }

    #[test]
    fn every_language_has_a_unique_tag_and_a_catalogue() {
        let tags: std::collections::HashSet<_> = LANGUAGES.iter().map(|(t, _)| *t).collect();
        assert_eq!(tags.len(), 11);
        for (tag, _) in LANGUAGES {
            assert!(tag == "en-US" || catalog(tag).is_some(), "{tag}");
        }
    }
}
