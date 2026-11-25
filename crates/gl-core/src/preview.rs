//! Markdown to HTML for thumbnails and the read-only document view.
//!
//! Raw HTML in a document is never passed to the web view: it is shown as text.

use pulldown_cmark::{CowStr, Event, Options, Parser, html};

/// The Markdown without its `---` front matter.
pub fn strip_front_matter(md: &str) -> &str {
    let Some(rest) = md.strip_prefix("---\n") else {
        return md;
    };
    match rest.find("\n---\n") {
        Some(i) => rest[i + 5..].trim_start_matches('\n'),
        None => md,
    }
}

/// Safe HTML for `md`; raw HTML in the document is shown as text.
pub fn to_html(md: &str) -> String {
    render(&slide_breaks(strip_front_matter(md)), false)
}

/// Safe HTML for a small thumbnail: like [`to_html`] but raw HTML (layout wrappers in the
/// templates) is left out instead of shown as text.
pub fn thumbnail_html(md: &str) -> String {
    render(&slide_breaks(strip_front_matter(md)), true)
}

/// The thumbnail of a template, with sample values in its placeholders.
pub fn template_html(t: &gl_templates::Template) -> String {
    use chrono::Datelike;
    let d = chrono::Local::now().date_naive();
    let vars = gl_templates::Vars { author: "Your name".into(), year: i64::from(d.year()), month: d.month(), day: d.day() };
    thumbnail_html(&gl_templates::instantiate(t, &vars))
}

/// `<!-- slide -->` markers become horizontal rules so decks read as separate slides.
fn slide_breaks(md: &str) -> String {
    md.lines()
        .map(|l| if l.trim_start().starts_with("<!-- slide") && l.trim_end().ends_with("-->") { "\n---\n" } else { l })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render(md: &str, drop_html: bool) -> String {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS | Options::ENABLE_FOOTNOTES;
    let events = Parser::new_ext(md, opts).filter_map(|e| match e {
        Event::Html(_) | Event::InlineHtml(_) if drop_html => None,
        Event::Html(s) | Event::InlineHtml(s) => Some(Event::Text(CowStr::from(s.into_string()))),
        other => Some(other),
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
    out
}

/// The title: the front matter's `title:` or the first `# ` heading.
pub fn title(md: &str) -> Option<String> {
    if let Some(rest) = md.strip_prefix("---\n") {
        for line in rest.lines().take_while(|l| *l != "---") {
            if let Some(t) = line.strip_prefix("title:") {
                return Some(t.trim().trim_matches('"').to_string()).filter(|t| !t.is_empty());
            }
        }
    }
    md.lines().find_map(|l| l.strip_prefix("# ").map(|t| t.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_is_removed() {
        assert_eq!(strip_front_matter("---\ntitle: A\n---\n\n# Hi\n"), "# Hi\n");
        assert_eq!(strip_front_matter("# Hi"), "# Hi");
    }

    #[test]
    fn raw_html_is_escaped() {
        let h = to_html("<script>alert(1)</script>\n\nhello <b>x</b>");
        assert!(!h.contains("<script>") && !h.contains("<b>"), "{h}");
        assert!(h.contains("&lt;script&gt;"));
    }

    #[test]
    fn thumbnails_drop_html_and_split_slides() {
        let h = thumbnail_html("<div class=\"x\">\n\n# Title\n\n</div>\n\n<!-- slide -->\n\n## Two");
        assert!(!h.contains("div") && h.contains("<h1>") && h.contains("<hr"), "{h}");
        for t in gl_templates::all() {
            assert!(template_html(t).len() > 20, "{}", t.id);
        }
    }

    #[test]
    fn tables_render_and_titles_are_found() {
        assert!(to_html("| a |\n|--|\n| 1 |").contains("<table>"));
        assert_eq!(title("---\ntitle: Memo\n---\n# X").as_deref(), Some("Memo"));
        assert_eq!(title("# Only heading").as_deref(), Some("Only heading"));
    }
}
