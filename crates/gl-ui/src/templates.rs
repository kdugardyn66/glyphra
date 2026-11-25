//! Templates: search, filters, one tab per kind, a grid of live thumbnails and page numbers.

use crate::icons::Icon;
use crate::shell::run;
use dioxus::prelude::*;
use gl_core::{AppModel, Command, preview, t};
use gl_templates::{Kind, categories, languages, purposes};

/// The page buttons to show: numbers (0-based) and `None` for a gap ("…").
pub fn page_items(page: usize, pages: usize) -> Vec<Option<usize>> {
    if pages == 0 {
        return Vec::new();
    }
    let mut keep: Vec<usize> = vec![0, pages - 1, page.saturating_sub(1), page, (page + 1).min(pages - 1)];
    if page <= 1 {
        keep.extend(0..4.min(pages));
    }
    if page + 2 >= pages {
        keep.extend(pages.saturating_sub(4)..pages);
    }
    keep.sort_unstable();
    keep.dedup();
    let mut out = Vec::new();
    let mut last: Option<usize> = None;
    for p in keep {
        if let Some(l) = last
            && p > l + 1
        {
            out.push(None);
        }
        out.push(Some(p));
        last = Some(p);
    }
    out
}

/// Sends [width of the grid, height of the window below the tabs] now and on every resize.
const MEASURE: &str = r#"
const start = () => {
    const page = document.querySelector('.templates');
    const pane = document.querySelector('.content');
    if (!page || !pane) { requestAnimationFrame(start); return; }
    const send = () => {
        const tabs = page.querySelector('.tabs');
        if (!tabs) return;
        const css = getComputedStyle(page);
        const width = page.clientWidth - parseFloat(css.paddingLeft) - parseFloat(css.paddingRight);
        // the room from the grid's top edge (the tabs' bottom and their margin) to the window's bottom
        const top = tabs.getBoundingClientRect().bottom + 20 - pane.getBoundingClientRect().top + pane.scrollTop;
        dioxus.send([width, pane.clientHeight - top]);
    };
    new ResizeObserver(send).observe(pane);
    new ResizeObserver(send).observe(page);
    send();
};
start();
"#;

#[component]
pub fn TemplatesPage() -> Element {
    let mut model = use_context::<Signal<AppModel>>();
    let m = model.read();
    let view = m.template_view();
    let f = m.templates.filter.clone();
    let kind = f.kind.unwrap_or(Kind::Document);
    let items: Vec<(&'static str, &'static str, String)> = view.items.iter().map(|t| (t.id, t.name, preview::template_html(t))).collect();
    let (page, pages, total) = (view.page, view.pages, view.total);
    drop(m);
    let buttons = page_items(page, pages);
    // Tell the model how big the grid is, now and whenever the window or the AI panel resizes.
    use_future(move || async move {
        let mut eval = document::eval(MEASURE);
        while let Ok(size) = eval.recv::<Vec<f64>>().await {
            if let [width, height] = size[..] {
                model.write().fit_templates(width as f32, height as f32);
            }
        }
    });
    let (cats, purps, langs) = (categories(), purposes(), languages());
    rsx! {
        div { class: "page templates",
            h1 { {t("Templates")} }
            div { class: "filters",
                label { class: "search",
                    Icon { name: "search", size: 18 }
                    input {
                        r#type: "text",
                        placeholder: t("Search templates (invoice, report, budget...)"),
                        initial_value: "{f.query}",
                        oninput: move |e| model.write().set_templates_query(e.value()),
                    }
                }
                select { class: "dd", onchange: move |e| model.write().set_templates_category(e.value()),
                    option { value: "", selected: f.category.is_empty(), {t("Category")} }
                    for c in cats {
                        option { key: "{c}", value: "{c}", selected: f.category == c, "{c}" }
                    }
                }
                select { class: "dd", onchange: move |e| model.write().set_templates_purpose(e.value()),
                    option { value: "", selected: f.purpose.is_empty(), {t("Purpose")} }
                    for p in purps {
                        option { key: "{p}", value: "{p}", selected: f.purpose == p, "{p}" }
                    }
                }
                select { class: "dd", onchange: move |e| model.write().set_templates_language(e.value()),
                    option { value: "", selected: f.language.is_empty(), {t("Language")} }
                    for l in langs {
                        option { key: "{l}", value: "{l}", selected: f.language == l, "{language_name(l)}" }
                    }
                }
            }
            div { class: "tabs", role: "tablist",
                for k in Kind::ALL {
                    button {
                        key: "{k.plural()}",
                        role: "tab",
                        class: if k == kind { "tab-btn active" } else { "tab-btn" },
                        onclick: move |_| model.write().set_templates_kind(k),
                        {t(k.plural())}
                    }
                }
            }
            if total == 0 {
                div { class: "empty", {t("No templates match. Clear the search or a filter.")} }
            }
            div { class: "grid",
                for (id , name , html) in items {
                    button { key: "{id}", class: "tpl", title: "Create “{name}”", onclick: move |_| run(model, Command::NewFromTemplate(id.to_string())),
                        div { class: "thumb",
                            div { class: "paper md", dangerous_inner_html: "{html}" }
                        }
                        span { class: "tpl-name", "{name}" }
                    }
                }
            }
            if pages > 1 {
                div { class: "pager",
                    button { class: "pg", disabled: page == 0, title: t("Previous page"), onclick: move |_| model.write().set_templates_page(page.saturating_sub(1)), Icon { name: "chev-left", size: 16 } }
                    for (i , b) in buttons.into_iter().enumerate() {
                        match b {
                            Some(p) => rsx! {
                                button { key: "{i}", class: if p == page { "pg active" } else { "pg" }, onclick: move |_| model.write().set_templates_page(p), "{p + 1}" }
                            },
                            None => rsx! {
                                span { key: "{i}", class: "gap", "…" }
                            },
                        }
                    }
                    button { class: "pg", disabled: page + 1 >= pages, title: t("Next page"), onclick: move |_| model.write().set_templates_page(page + 1), Icon { name: "chev-right", size: 16 } }
                }
            }
        }
    }
}

fn language_name(tag: &str) -> &'static str {
    match tag {
        "en" => "English",
        _ => "Other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shown(page: usize, pages: usize) -> String {
        page_items(page, pages)
            .into_iter()
            .map(|i| i.map(|p| (p + 1).to_string()).unwrap_or_else(|| "…".into()))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn page_buttons() {
        assert_eq!(shown(0, 1), "1");
        assert_eq!(shown(0, 3), "1 2 3");
        assert_eq!(shown(0, 9), "1 2 3 4 … 9");
        assert_eq!(shown(4, 9), "1 … 4 5 6 … 9");
        assert_eq!(shown(8, 9), "1 … 6 7 8 9");
        assert!(page_items(0, 0).is_empty());
    }
}
