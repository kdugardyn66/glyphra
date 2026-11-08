//! Read-only Markdown preview for 0.1.0 (the editor and the full renderer arrive in 0.3.0).
//!
//! Raw HTML inside a Markdown file is never passed to the web view: it is shown as text.
//! The web view can call into Glyphra, so a document must not be able to run script.

use pulldown_cmark::{Event, Options, Parser, html};

/// Largest file shown (bigger files get a message instead of a slow preview).
pub const MAX_PREVIEW_BYTES: u64 = 5 * 1024 * 1024;

/// Markdown to a safe HTML fragment.
pub fn render(markdown: &str) -> String {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS | Options::ENABLE_FOOTNOTES;
    let events = Parser::new_ext(markdown, opts).map(|e| match e {
        Event::Html(h) | Event::InlineHtml(h) => Event::Text(h),
        other => other,
    });
    let mut out = String::with_capacity(markdown.len() * 3 / 2);
    html::push_html(&mut out, events);
    sanitize_urls(out)
}

/// Links and images with a `javascript:` or `data:` target are neutralised.
fn sanitize_urls(html: String) -> String {
    let mut out = html;
    for attr in ["href", "src"] {
        for bad in ["javascript:", "data:", "vbscript:"] {
            let needle = format!("{attr}=\"{bad}");
            while let Some(i) = out.to_ascii_lowercase().find(&needle) {
                out.replace_range(i..i + needle.len(), &format!("{attr}=\"#blocked:"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_headings_tables_and_task_lists() {
        let h = render("# Title\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n- [x] done\n");
        assert!(h.contains("<h1>Title</h1>"));
        assert!(h.contains("<table>") && h.contains("<td>1</td>"));
        assert!(h.contains("checkbox"));
    }

    #[test]
    fn raw_html_is_shown_as_text_not_run() {
        let h = render("<script>alert(1)</script>\n\nhello <img src=x onerror=alert(1)> there");
        assert!(!h.contains("<script"), "{h}");
        assert!(!h.contains("<img"), "{h}");
        assert!(h.contains("&lt;script&gt;"));
    }

    #[test]
    fn script_urls_are_blocked() {
        let h = render("[x](javascript:alert(1)) ![y](data:text/html;base64,AAAA) [ok](https://example.org)");
        assert!(!h.to_lowercase().contains("href=\"javascript:"), "{h}");
        assert!(!h.to_lowercase().contains("src=\"data:"), "{h}");
        assert!(h.contains("href=\"https://example.org\""));
    }
}
