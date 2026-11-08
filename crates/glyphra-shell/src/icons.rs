//! Inline icons (24x24, stroke) and the Glyphra logo.

use dioxus::prelude::*;

fn paths(name: &str) -> &'static str {
    match name {
        "home" => r#"<path d="M3 11.5 12 4l9 7.5"/><path d="M5.5 10v10h4.5v-6h4v6h4.5V10"/>"#,
        "plus" => r#"<path d="M12 5v14M5 12h14"/>"#,
        "folder" => {
            r#"<path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5z"/>"#
        }
        "folder-open" => {
            r#"<path d="M3 18.5V7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h6.5A2.5 2.5 0 0 1 20 9.5V11"/><path d="M3.5 19 6 12h15l-2.6 6.4a1.5 1.5 0 0 1-1.4 1.1H5a1.5 1.5 0 0 1-1.5-.5z"/>"#
        }
        "clock" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>"#,
        "star" => r#"<path d="m12 3.5 2.6 5.5 6 .8-4.4 4.2 1.1 6-5.3-2.9-5.3 2.9 1.1-6L3.4 9.8l6-.8z"/>"#,
        "chev-right" => r#"<path d="m9.5 6 6 6-6 6"/>"#,
        "chev-down" => r#"<path d="m6 9.5 6 6 6-6"/>"#,
        "settings" => {
            r#"<path d="M4 7h9M17 7h3M4 12h3M11 12h9M4 17h11M19 17h1"/><circle cx="15" cy="7" r="2"/><circle cx="9" cy="12" r="2"/><circle cx="17" cy="17" r="2"/>"#
        }
        "search" => r#"<circle cx="11" cy="11" r="6.5"/><path d="m20 20-4.2-4.2"/>"#,
        "refresh" => r#"<path d="M19.5 12a7.5 7.5 0 1 1-2.3-5.4"/><path d="M19.5 4.5v4h-4"/>"#,
        "more" => r#"<circle cx="5.5" cy="12" r="1.3"/><circle cx="12" cy="12" r="1.3"/><circle cx="18.5" cy="12" r="1.3"/>"#,
        "x" => r#"<path d="m6.5 6.5 11 11M17.5 6.5l-11 11"/>"#,
        "sun" => {
            r#"<circle cx="12" cy="12" r="3.8"/><path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6 7 7M17 17l1.4 1.4M5.6 18.4 7 17M17 7l1.4-1.4"/>"#
        }
        "moon" => r#"<path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z"/>"#,
        "monitor" => r#"<rect x="3" y="4.5" width="18" height="12" rx="2"/><path d="M8.5 20h7M12 16.5V20"/>"#,
        "apps" => {
            r#"<rect x="4" y="4" width="6.5" height="6.5" rx="1.5"/><rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5"/><rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5"/><rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5"/>"#
        }
        "chat" => {
            r#"<path d="M5 5.5h14A1.5 1.5 0 0 1 20.5 7v8.5A1.5 1.5 0 0 1 19 17h-7l-4.5 3.5V17H5A1.5 1.5 0 0 1 3.5 15.5V7A1.5 1.5 0 0 1 5 5.5z"/>"#
        }
        "sparkles" => {
            r#"<path d="M10 4c0 3.5 2.5 6 6 6-3.5 0-6 2.5-6 6 0-3.5-2.5-6-6-6 3.5 0 6-2.5 6-6z"/><path d="M18 14.5c0 1.7 1.3 3 3 3-1.7 0-3 1.3-3 3 0-1.7-1.3-3-3-3 1.7 0 3-1.3 3-3z"/>"#
        }
        "tools" => r#"<path d="M14.5 6.5a4 4 0 0 0-5 5L4 17l3 3 5.5-5.5a4 4 0 0 0 5-5l-2.5 2.5-2-.5-.5-2z"/>"#,
        "paperclip" => r#"<path d="m20 11-8 8a5 5 0 0 1-7-7l8.5-8.5a3.5 3.5 0 0 1 5 5L10 17a2 2 0 0 1-3-3l7.5-7.5"/>"#,
        "send" => r#"<path d="M12 19V5M6 11l6-6 6 6"/>"#,
        "stop" => r#"<rect x="7" y="7" width="10" height="10" rx="2"/>"#,
        "terminal" => r#"<rect x="3.5" y="5" width="17" height="14" rx="2.5"/><path d="m7.5 10 3 2-3 2M12.5 15h4"/>"#,
        "copy" => {
            r#"<rect x="8.5" y="8.5" width="11" height="11" rx="2"/><path d="M15.5 8.5V6A1.5 1.5 0 0 0 14 4.5H6A1.5 1.5 0 0 0 4.5 6v8A1.5 1.5 0 0 0 6 15.5h2.5"/>"#
        }
        "back" => r#"<path d="M15 6l-6 6 6 6"/>"#,
        "trash" => r#"<path d="M5 7h14M9.5 7V5h5v2M7 7l.8 12h8.4L17 7"/>"#,
        "sort-up" => r#"<path d="m7 14 5-5 5 5"/>"#,
        "sort-down" => r#"<path d="m7 10 5 5 5-5"/>"#,
        _ => "",
    }
}

#[component]
pub fn Icon(name: &'static str, size: Option<u32>) -> Element {
    let s = size.unwrap_or(18);
    rsx! {
        svg { class: "icon", view_box: "0 0 24 24", width: "{s}", height: "{s}", dangerous_inner_html: paths(name) }
    }
}

/// The Glyphra mark: hash molecule with the AI spark. Colours come from the theme variables,
/// so the same markup is the light and the dark icon.
#[component]
pub fn Logo(class: Option<String>) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        svg { class: "{class}", view_box: "0 0 512 512",
            rect { x: "4", y: "4", width: "504", height: "504", rx: "112", style: "fill:var(--logo-tile);stroke:var(--logo-border);stroke-width:4" }
            g {
                transform: "translate(26,26) scale(3.83)",
                style: "fill:none;stroke:var(--logo-accent);stroke-width:5;stroke-linecap:round;stroke-linejoin:round",
                polygon { points: "60,20 94.6,40 94.6,80 60,100 25.4,80 25.4,40" }
                path { d: "M54 45 L50 75 M70 45 L66 75 M44 55 H76 M42 65 H74" }
                circle { cx: "60", cy: "20", r: "6", style: "fill:var(--logo-accent)" }
                circle { cx: "94.6", cy: "80", r: "6", style: "fill:var(--logo-accent)" }
                circle { cx: "25.4", cy: "80", r: "6", style: "fill:var(--logo-accent)" }
                g { style: "stroke:none;fill:var(--logo-spark)",
                    path { d: "M100 8 Q100 24 116 24 Q100 24 100 40 Q100 24 84 24 Q100 24 100 8 Z" }
                    path { d: "M84 2 Q84 8 90 8 Q84 8 84 14 Q84 8 78 8 Q84 8 84 2 Z" }
                }
            }
        }
    }
}
