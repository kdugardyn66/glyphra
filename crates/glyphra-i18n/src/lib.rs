//! Languages for Glyphra.
//!
//! Every visible string is a [Fluent](https://projectfluent.org) message id. A language is a
//! folder `locales/<tag>/` with `main.ftl` (the messages) and `locale.toml` (its name, text
//! direction and optional font stack). Four packs are built in (English, Dutch, French,
//! German); more are loaded at start-up from the user's `locales` folder, so a language such
//! as Japanese, Mandarin, Hebrew or Arabic is added by dropping in a folder: no code change.
//!
//! A message missing from a language falls back to English, one message at a time. A message
//! missing from English shows its id, which is how a typo is noticed.

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use include_dir::{Dir, include_dir};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use unic_langid::LanguageIdentifier;

static BUILT_IN: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../locales");

/// The language every other language falls back to.
pub const FALLBACK: &str = "en";

/// Left-to-right or right-to-left text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

impl Direction {
    /// Value of the HTML `dir` attribute.
    pub fn html(self) -> &'static str {
        match self {
            Direction::Ltr => "ltr",
            Direction::Rtl => "rtl",
        }
    }
}

/// What `locale.toml` says about a language.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LocaleInfo {
    /// BCP 47 tag: `nl`, `zh-Hans`, `ar`.
    pub tag: String,
    /// Name in English: "Dutch".
    pub name: String,
    /// Name in the language itself: "Nederlands".
    pub native_name: String,
    #[serde(default)]
    pub direction: Direction,
    /// CSS `font-family` list for scripts the default fonts do not cover (CJK, Arabic, Hebrew).
    #[serde(default)]
    pub font: Option<String>,
}

/// One loaded language.
struct Pack {
    info: LocaleInfo,
    bundle: FluentBundle<FluentResource>,
    /// Message ids this pack defines (for completeness reports).
    ids: Vec<String>,
    /// True when it came from the user's folder.
    user: bool,
}

/// Every language that could be loaded. Cheap to share between threads.
#[derive(Clone)]
pub struct Catalog {
    packs: Arc<Vec<Pack>>,
}

/// A problem found while loading a language pack; the pack is skipped, the app starts anyway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadIssue {
    pub folder: String,
    pub problem: String,
}

impl Catalog {
    /// The built-in languages plus any language packs in `user_dir` (`<config>/locales`).
    /// Problems with a user pack are returned, never fatal.
    pub fn load(user_dir: Option<&Path>) -> (Catalog, Vec<LoadIssue>) {
        let mut issues = Vec::new();
        let mut packs: Vec<Pack> = Vec::new();
        for dir in BUILT_IN.dirs() {
            let name = dir.path().file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let toml = dir.get_file(dir.path().join("locale.toml")).and_then(|f| f.contents_utf8());
            let ftl = dir.get_file(dir.path().join("main.ftl")).and_then(|f| f.contents_utf8());
            match (toml, ftl) {
                (Some(t), Some(f)) => match build(t, f, false) {
                    Ok(p) => packs.push(p),
                    Err(e) => issues.push(LoadIssue { folder: name, problem: e }),
                },
                _ => issues.push(LoadIssue { folder: name, problem: "needs main.ftl and locale.toml".into() }),
            }
        }
        if let Some(user) = user_dir
            && let Ok(rd) = fs::read_dir(user)
        {
            let mut dirs: Vec<_> = rd.flatten().filter(|e| e.path().is_dir()).collect();
            dirs.sort_by_key(|e| e.file_name());
            for entry in dirs {
                let name = entry.file_name().to_string_lossy().into_owned();
                let read = |f: &str| fs::read_to_string(entry.path().join(f));
                match (read("locale.toml"), read("main.ftl")) {
                    (Ok(t), Ok(f)) => match build(&t, &f, true) {
                        // a user pack with the tag of a built-in one replaces it
                        Ok(p) => {
                            packs.retain(|b| b.info.tag != p.info.tag);
                            packs.push(p);
                        }
                        Err(e) => issues.push(LoadIssue { folder: name, problem: e }),
                    },
                    _ => issues.push(LoadIssue { folder: name, problem: "needs main.ftl and locale.toml".into() }),
                }
            }
        }
        // English first, then alphabetical by native name: the order of the language list
        packs.sort_by(|a, b| {
            (a.info.tag != FALLBACK, a.info.native_name.to_lowercase()).cmp(&(b.info.tag != FALLBACK, b.info.native_name.to_lowercase()))
        });
        (Catalog { packs: Arc::new(packs) }, issues)
    }

    /// The languages, in menu order.
    pub fn locales(&self) -> Vec<LocaleInfo> {
        self.packs.iter().map(|p| p.info.clone()).collect()
    }

    /// True for a pack that came from the user's folder.
    pub fn is_user_pack(&self, tag: &str) -> bool {
        self.packs.iter().any(|p| p.info.tag == tag && p.user)
    }

    /// Message ids English defines that `tag` does not (an unfinished translation).
    pub fn missing(&self, tag: &str) -> Vec<String> {
        let en = self.packs.iter().find(|p| p.info.tag == FALLBACK);
        let other = self.packs.iter().find(|p| p.info.tag == tag);
        match (en, other) {
            (Some(en), Some(o)) => en.ids.iter().filter(|id| !o.ids.contains(id)).cloned().collect(),
            _ => Vec::new(),
        }
    }

    /// Index of the pack that best fits `wanted` (`nl`, `nl-BE`, `zh-Hans-CN`, or empty).
    fn best(&self, wanted: &str) -> usize {
        let fallback = self.packs.iter().position(|p| p.info.tag == FALLBACK).unwrap_or(0);
        let w = wanted.trim().replace('_', "-");
        if w.is_empty() {
            return fallback;
        }
        let w = w.split('.').next().unwrap_or(&w).to_string(); // "nl_BE.UTF-8"
        if let Some(i) = self.packs.iter().position(|p| p.info.tag.eq_ignore_ascii_case(&w)) {
            return i;
        }
        // drop subtags from the right: zh-Hans-CN -> zh-Hans -> zh
        let mut parts: Vec<&str> = w.split('-').collect();
        while parts.len() > 1 {
            parts.pop();
            let t = parts.join("-");
            if let Some(i) = self.packs.iter().position(|p| p.info.tag.eq_ignore_ascii_case(&t)) {
                return i;
            }
        }
        // a pack for a region of the language (pt-BR when asked for pt)
        let lang = parts[0].to_ascii_lowercase();
        self.packs.iter().position(|p| p.info.tag.split('-').next().is_some_and(|l| l.eq_ignore_ascii_case(&lang))).unwrap_or(fallback)
    }
}

fn build(toml_text: &str, ftl: &str, user: bool) -> Result<Pack, String> {
    let info: LocaleInfo = toml::from_str(toml_text).map_err(|e| format!("locale.toml: {e}"))?;
    let lang: LanguageIdentifier = info.tag.parse().map_err(|_| format!("locale.toml: “{}” is not a language tag", info.tag))?;
    let resource = FluentResource::try_new(ftl.to_string()).map_err(|(partial, errors)| {
        // keep going with what parsed? No: a half-read pack hides mistakes. Report the first.
        let first = errors.first().map(|e| format!("{:?}", e.kind)).unwrap_or_default();
        format!("main.ftl: {} error(s), first: {first} ({} messages read)", errors.len(), partial.entries().count())
    })?;
    let ids = resource
        .entries()
        .filter_map(|e| match e {
            fluent_syntax::ast::Entry::Message(m) => Some(m.id.name.to_string()),
            _ => None,
        })
        .collect();
    let mut bundle = FluentBundle::new_concurrent(vec![lang]);
    // Isolation marks keep mixed-direction text in order, but only matter for right-to-left.
    bundle.set_use_isolating(info.direction == Direction::Rtl);
    bundle.add_resource(resource).map_err(|e| format!("main.ftl: {} duplicate message id(s)", e.len()))?;
    Ok(Pack { info, bundle, ids, user })
}

/// The language in use: look messages up with [`I18n::t`].
#[derive(Clone)]
pub struct I18n {
    catalog: Catalog,
    current: usize,
    fallback: usize,
}

/// A value for a message variable.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Text(String),
    Number(f64),
}

impl From<&str> for Arg {
    fn from(v: &str) -> Self {
        Arg::Text(v.to_string())
    }
}
impl From<String> for Arg {
    fn from(v: String) -> Self {
        Arg::Text(v)
    }
}
impl From<usize> for Arg {
    fn from(v: usize) -> Self {
        Arg::Number(v as f64)
    }
}
impl From<u64> for Arg {
    fn from(v: u64) -> Self {
        Arg::Number(v as f64)
    }
}

impl I18n {
    /// `requested` is the tag saved in Settings (empty = follow the system); `system` is what
    /// the operating system reports.
    pub fn new(catalog: &Catalog, requested: &str, system: &str) -> I18n {
        let wanted = if requested.trim().is_empty() { system } else { requested };
        let fallback = catalog.packs.iter().position(|p| p.info.tag == FALLBACK).unwrap_or(0);
        I18n { catalog: catalog.clone(), current: catalog.best(wanted), fallback }
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// The language in use.
    pub fn info(&self) -> &LocaleInfo {
        &self.catalog.packs[self.current].info
    }

    /// The text for a message id.
    pub fn t(&self, id: &str) -> String {
        self.t_args(id, &[])
    }

    /// The text for a message id with variables: `t_args("files-count", &[("n", 3usize.into())])`.
    pub fn t_args(&self, id: &str, args: &[(&str, Arg)]) -> String {
        let mut fa = FluentArgs::new();
        for (k, v) in args {
            match v {
                Arg::Text(s) => fa.set(*k, FluentValue::from(s.clone())),
                Arg::Number(n) => fa.set(*k, FluentValue::from(*n)),
            }
        }
        for idx in [self.current, self.fallback] {
            let pack = &self.catalog.packs[idx];
            if let Some(msg) = pack.bundle.get_message(id)
                && let Some(pattern) = msg.value()
            {
                let mut errors = Vec::new();
                let out = pack.bundle.format_pattern(pattern, Some(&fa), &mut errors);
                if errors.is_empty() {
                    return out.into_owned();
                }
            }
        }
        id.to_string()
    }

    /// True when `id` exists in the language in use (not only in English).
    pub fn has(&self, id: &str) -> bool {
        self.catalog.packs[self.current].bundle.has_message(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin() -> Catalog {
        let (c, issues) = Catalog::load(None);
        assert!(issues.is_empty(), "{issues:?}");
        c
    }

    #[test]
    fn four_languages_are_built_in_english_first() {
        let tags: Vec<_> = builtin().locales().into_iter().map(|l| l.tag).collect();
        assert_eq!(tags[0], "en");
        for t in ["en", "nl", "fr", "de"] {
            assert!(tags.iter().any(|x| x == t), "{t} missing");
        }
    }

    #[test]
    fn every_built_in_language_is_complete() {
        let c = builtin();
        for l in c.locales() {
            assert_eq!(c.missing(&l.tag), Vec::<String>::new(), "{} is missing messages", l.tag);
        }
    }

    #[test]
    fn no_built_in_language_has_ids_english_lacks() {
        let c = builtin();
        let en = c.packs.iter().find(|p| p.info.tag == "en").unwrap();
        for p in c.packs.iter() {
            for id in &p.ids {
                assert!(en.ids.contains(id), "{} defines {id}, which English does not", p.info.tag);
            }
        }
    }

    #[test]
    fn picks_the_closest_language() {
        let c = builtin();
        let tag = |req: &str, sys: &str| I18n::new(&c, req, sys).info().tag.clone();
        assert_eq!(tag("nl", "en"), "nl");
        assert_eq!(tag("", "fr-BE"), "fr");
        assert_eq!(tag("", "de_DE.UTF-8"), "de");
        assert_eq!(tag("", "ja-JP"), "en");
        assert_eq!(tag("", ""), "en");
        assert_eq!(tag("NL-be", "en"), "nl");
    }

    #[test]
    fn a_missing_message_falls_back_to_english_then_to_its_id() {
        let tmp = std::env::temp_dir().join(format!("glyphra-i18n-{}-a", std::process::id()));
        let pack = tmp.join("it");
        fs::create_dir_all(&pack).unwrap();
        fs::write(pack.join("locale.toml"), "tag = \"it\"\nname = \"Italian\"\nnative_name = \"Italiano\"\n").unwrap();
        fs::write(pack.join("main.ftl"), "app-name = Glifra\n").unwrap();
        let (c, issues) = Catalog::load(Some(&tmp));
        assert!(issues.is_empty(), "{issues:?}");
        let i = I18n::new(&c, "it", "en");
        assert_eq!(i.t("app-name"), "Glifra");
        assert!(!c.missing("it").is_empty());
        let some_other = c.missing("it")[0].clone();
        assert_eq!(i.t(&some_other), I18n::new(&c, "en", "en").t(&some_other), "falls back to English");
        assert_eq!(i.t("no-such-message"), "no-such-message");
        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn a_right_to_left_pack_in_the_users_folder_works_without_code_changes() {
        let tmp = std::env::temp_dir().join(format!("glyphra-i18n-{}-b", std::process::id()));
        let pack = tmp.join("he");
        fs::create_dir_all(&pack).unwrap();
        fs::write(
            pack.join("locale.toml"),
            "tag = \"he\"\nname = \"Hebrew\"\nnative_name = \"עברית\"\ndirection = \"rtl\"\nfont = \"'Noto Sans Hebrew', sans-serif\"\n",
        )
        .unwrap();
        fs::write(pack.join("main.ftl"), "app-name = גליפרה\n").unwrap();
        let (c, issues) = Catalog::load(Some(&tmp));
        assert!(issues.is_empty());
        let i = I18n::new(&c, "", "he-IL");
        assert_eq!(i.info().direction, Direction::Rtl);
        assert_eq!(i.info().direction.html(), "rtl");
        assert!(i.info().font.as_deref().unwrap().contains("Hebrew"));
        assert!(c.is_user_pack("he") && !c.is_user_pack("en"));
        assert!(i.t("app-name").contains("גליפרה"));
        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn a_broken_pack_is_reported_and_skipped() {
        let tmp = std::env::temp_dir().join(format!("glyphra-i18n-{}-c", std::process::id()));
        fs::create_dir_all(tmp.join("xx")).unwrap();
        fs::write(tmp.join("xx/locale.toml"), "tag = \"xx\"\nname = \"X\"\nnative_name = \"X\"\n").unwrap();
        fs::write(tmp.join("xx/main.ftl"), "= nonsense {\n").unwrap();
        fs::create_dir_all(tmp.join("yy")).unwrap();
        let (c, issues) = Catalog::load(Some(&tmp));
        assert_eq!(issues.len(), 2, "{issues:?}");
        assert!(c.locales().iter().all(|l| l.tag != "xx"));
        fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn variables_and_plurals() {
        let c = builtin();
        let en = I18n::new(&c, "en", "");
        assert_eq!(en.t_args("home-files-count", &[("n", 1usize.into())]), "1 file");
        assert_eq!(en.t_args("home-files-count", &[("n", 3usize.into())]), "3 files");
        let nl = I18n::new(&c, "nl", "");
        assert_eq!(nl.t_args("home-files-count", &[("n", 3usize.into())]), "3 bestanden");
        assert_eq!(en.t_args("home-greeting-named", &[("part", "Good morning".into()), ("name", "Kris".into())]), "Good morning, Kris");
    }
}
