//! The Markdown templates shipped with Glyphra: documents and presentations
//! (`<!-- slide -->` markers). Carried over from mdCraft.
//!
//! Templates may contain placeholders that [`instantiate`] fills in: `{{author}}`, `{{date}}`,
//! `{{date_long}}`, `{{month_year}}`, `{{year}}`, `{{next_year}}`, `{{quarter}}`.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// What a template makes. The order is the order of the tabs on the Templates page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Document,
    Presentation,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::Document, Kind::Presentation];

    /// Tab title on the Templates page.
    pub fn plural(self) -> &'static str {
        match self {
            Kind::Document => "Documents",
            Kind::Presentation => "Presentations",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Template {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    pub description: &'static str,
    /// The theme the template's front matter selects.
    pub theme: &'static str,
    pub keywords: &'static str,
    /// Filter on the Templates page: Business, Personal, Education, Technical.
    pub category: &'static str,
    /// Filter on the Templates page: Report, Letter, Finance …
    pub purpose: &'static str,
    /// Templates are English for now; the filter exists so other languages can be added.
    pub language: &'static str,
    #[serde(skip)]
    pub source: &'static str,
}

macro_rules! t {
    ($id:literal, $name:literal, $kind:ident, $theme:literal, $file:literal, $desc:literal, $kw:literal, $cat:literal, $purpose:literal) => {
        Template {
            id: $id,
            name: $name,
            kind: Kind::$kind,
            description: $desc,
            theme: $theme,
            keywords: $kw,
            category: $cat,
            purpose: $purpose,
            language: "en",
            source: include_str!(concat!("../../../templates/", $file)),
        }
    };
}

static TEMPLATES: &[Template] = &[
    t!(
        "blank",
        "Blank document",
        Document,
        "office",
        "documents/blank.md",
        "An empty page with a title, ready to write.",
        "empty new",
        "Personal",
        "Notes"
    ),
    t!(
        "welcome",
        "Welcome to Glyphra",
        Document,
        "office",
        "documents/welcome.md",
        "A tour of everything Glyphra renders: styles, tables, diagrams, math, charts.",
        "tour sample help guide",
        "Personal",
        "Guide"
    ),
    t!(
        "report",
        "Business report",
        Document,
        "report",
        "documents/report.md",
        "Cover page, contents, executive summary, findings with charts and tables.",
        "annual quarterly analysis",
        "Business",
        "Report"
    ),
    t!(
        "proposal",
        "Project proposal",
        Document,
        "office",
        "documents/proposal.md",
        "Problem, solution, scope, timeline (Gantt), budget and sign-off.",
        "plan bid quote",
        "Business",
        "Plan"
    ),
    t!(
        "letter",
        "Formal letter",
        Document,
        "letter",
        "documents/letter.md",
        "A business letter with letterhead, address block and signature.",
        "correspondence mail cover",
        "Business",
        "Letter"
    ),
    t!(
        "memo",
        "Memo",
        Document,
        "office",
        "documents/memo.md",
        "A one-page internal memo with To / From / Subject block.",
        "memorandum note internal",
        "Business",
        "Letter"
    ),
    t!(
        "minutes",
        "Meeting minutes",
        Document,
        "office",
        "documents/minutes.md",
        "Attendees, agenda, decisions and an action-item checklist.",
        "meeting notes agenda actions",
        "Business",
        "Meeting"
    ),
    t!(
        "resume",
        "Résumé",
        Document,
        "modern",
        "documents/resume.md",
        "A clean one-page CV: profile, experience, skills, education.",
        "cv curriculum vitae job",
        "Personal",
        "Résumé"
    ),
    t!(
        "spec",
        "Technical specification",
        Document,
        "modern",
        "documents/spec.md",
        "Design doc with architecture diagram, API table, sequence diagram, risks.",
        "design doc rfc architecture engineering",
        "Technical",
        "Specification"
    ),
    t!(
        "paper",
        "Research paper",
        Document,
        "classic",
        "documents/paper.md",
        "Academic paper with abstract, equations, figures, footnotes and references.",
        "academic thesis article science math",
        "Education",
        "Report"
    ),
    t!(
        "newsletter",
        "Newsletter",
        Document,
        "newsletter",
        "documents/newsletter.md",
        "A team newsletter with a masthead, highlights and two-column stories.",
        "bulletin update news",
        "Business",
        "Newsletter"
    ),
    t!(
        "blank-deck",
        "Blank presentation",
        Presentation,
        "paper",
        "slides/blank.md",
        "A title slide and a content slide.",
        "empty slides deck new",
        "Personal",
        "Presentation"
    ),
    t!(
        "pitch",
        "Pitch deck",
        Presentation,
        "aurora",
        "slides/pitch.md",
        "Problem, solution, market, traction, business model, team, the ask.",
        "startup investor fundraising",
        "Business",
        "Presentation"
    ),
    t!(
        "status",
        "Project status update",
        Presentation,
        "corporate",
        "slides/status.md",
        "RAG status, milestones, risks and next steps for a steering meeting.",
        "steering report progress",
        "Business",
        "Report"
    ),
    t!(
        "qbr",
        "Quarterly business review",
        Presentation,
        "paper",
        "slides/qbr.md",
        "KPIs, revenue chart, wins and misses, priorities for next quarter.",
        "kpi metrics review results",
        "Business",
        "Report"
    ),
    t!(
        "training",
        "Training workshop",
        Presentation,
        "sunrise",
        "slides/training.md",
        "Agenda, learning objectives, exercises, quiz and wrap-up.",
        "course lesson onboarding",
        "Education",
        "Presentation"
    ),
    t!(
        "tech-talk",
        "Tech talk",
        Presentation,
        "midnight",
        "slides/tech-talk.md",
        "A conference talk with code, architecture diagrams and benchmarks.",
        "conference engineering code",
        "Technical",
        "Presentation"
    ),
    t!(
        "lecture",
        "Lecture",
        Presentation,
        "chalk",
        "slides/lecture.md",
        "A university lecture with definitions, equations and worked examples.",
        "class teaching math science",
        "Education",
        "Presentation"
    ),
    t!(
        "launch",
        "Product launch",
        Presentation,
        "ocean",
        "slides/launch.md",
        "Vision, features, roadmap timeline, pricing and launch plan.",
        "release roadmap marketing",
        "Business",
        "Plan"
    ),
];

/// Every template: documents and presentations.
pub fn all() -> &'static [Template] {
    TEMPLATES
}

pub fn get(id: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.id.eq_ignore_ascii_case(id.trim()))
}

/// The distinct values of a filter, sorted: `categories()` and `purposes()`.
pub fn categories() -> Vec<&'static str> {
    distinct(|t| t.category)
}

pub fn purposes() -> Vec<&'static str> {
    distinct(|t| t.purpose)
}

pub fn languages() -> Vec<&'static str> {
    distinct(|t| t.language)
}

fn distinct(f: impl Fn(&'static Template) -> &'static str) -> Vec<&'static str> {
    let mut v: Vec<_> = TEMPLATES.iter().map(f).collect();
    v.sort();
    v.dedup();
    v
}

/// What the Templates page asks for. Empty strings mean "any".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    pub query: String,
    pub kind: Option<Kind>,
    pub category: String,
    pub purpose: String,
    pub language: String,
}

/// Templates matching the filter, in catalogue order.
pub fn filter(f: &Filter) -> Vec<&'static Template> {
    let words: Vec<String> = f.query.split_whitespace().map(fold).collect();
    TEMPLATES
        .iter()
        .filter(|t| f.kind.is_none_or(|k| k == t.kind))
        .filter(|t| f.category.is_empty() || t.category == f.category)
        .filter(|t| f.purpose.is_empty() || t.purpose == f.purpose)
        .filter(|t| f.language.is_empty() || t.language == f.language)
        .filter(|t| {
            let hay = fold(&format!("{} {} {} {}", t.id, t.name, t.description, t.keywords));
            words.iter().all(|w| hay.contains(w.as_str()))
        })
        .collect()
}

/// Lowercase without the accents people don't type (`résumé` → `resume`).
fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

/// Values for the placeholders.
#[derive(Clone, Debug)]
pub struct Vars {
    pub author: String,
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

impl Vars {
    fn month_name(&self) -> &'static str {
        MONTHS.get(self.month.saturating_sub(1) as usize).copied().unwrap_or("January")
    }
    pub fn pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("author", self.author.clone()),
            ("date", format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)),
            ("date_long", format!("{} {} {}", self.day, self.month_name(), self.year)),
            ("month_year", format!("{} {}", self.month_name(), self.year)),
            ("year", self.year.to_string()),
            ("next_year", (self.year + 1).to_string()),
            ("quarter", format!("Q{} {}", (self.month.clamp(1, 12) - 1) / 3 + 1, self.year)),
        ]
    }
}

/// The template's Markdown with its placeholders filled in.
pub fn instantiate(t: &Template, vars: &Vars) -> String {
    fill(t.source, vars)
}

pub fn fill(text: &str, vars: &Vars) -> String {
    let mut out = text.to_string();
    for (k, v) in vars.pairs() {
        out = out.replace(&format!("{{{{{k}}}}}"), &v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> Vars {
        Vars { author: "Ada Lovelace".into(), year: 2026, month: 10, day: 14 }
    }

    #[test]
    fn ids_are_unique_and_sources_present() {
        let mut ids: Vec<_> = all().iter().map(|t| t.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all().len());
        for t in all() {
            assert!(t.source.trim().len() > 20, "{} is empty", t.id);
            assert!(t.source.replace("\r\n", "\n").starts_with("---\n"), "{} has no front matter", t.id);
            assert!(t.source.contains(&format!("theme: {}", t.theme)), "{}: theme differs from the catalogue", t.id);
        }
    }

    #[test]
    fn presentations_have_slides_and_the_rest_do_not() {
        for t in all() {
            assert_eq!(t.source.contains("<!-- slide"), t.kind == Kind::Presentation, "{}", t.id);
        }
    }

    #[test]
    fn every_kind_has_templates() {
        for k in Kind::ALL {
            assert!(all().iter().any(|t| t.kind == k), "{k:?} has none");
        }
    }

    #[test]
    fn placeholders_are_filled() {
        for t in all() {
            let s = instantiate(t, &vars());
            assert!(!s.contains("{{"), "{} keeps a placeholder", t.id);
        }
        assert!(vars().pairs().contains(&("quarter", "Q4 2026".to_string())));
    }

    #[test]
    fn filters_combine() {
        assert_eq!(filter(&Filter { query: "cv".into(), ..Default::default() }).first().map(|t| t.id), Some("resume"));
        assert_eq!(filter(&Filter::default()).len(), all().len());
        let f = Filter { kind: Some(Kind::Presentation), category: "Business".into(), ..Default::default() };
        let hits = filter(&f);
        assert!(!hits.is_empty() && hits.iter().all(|t| t.kind == Kind::Presentation && t.category == "Business"));
        assert!(categories().contains(&"Education"));
    }
}
