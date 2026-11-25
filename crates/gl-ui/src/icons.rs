//! Inline icons (24x24, stroke) and the Glyphra logo.

use dioxus::prelude::*;

fn paths(name: &str) -> &'static str {
    match name {
        "home" => r#"<path d="M3 11.5 12 4l9 7.5"/><path d="M5.5 10v10h4.5v-6h4v6h4.5V10"/>"#,
        "folder-open" => {
            r#"<path d="M3 18.5V7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h6.5A2.5 2.5 0 0 1 20 9.5V11"/><path d="M3.5 19 6 12h15l-2.6 6.4a1.5 1.5 0 0 1-1.4 1.1H5a1.5 1.5 0 0 1-1.5-.5z"/>"#
        }
        "folder" => {
            r#"<path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5z"/>"#
        }
        "templates" => {
            r#"<rect x="4" y="4" width="6.5" height="6.5" rx="1.5"/><rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5"/><rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5"/><rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5"/>"#
        }
        "collaboration" => {
            r#"<circle cx="9" cy="8.5" r="3"/><path d="M3.5 19a5.5 5.5 0 0 1 11 0"/><circle cx="17" cy="9.5" r="2.4"/><path d="M16 14.2a4.5 4.5 0 0 1 5 4.3"/>"#
        }
        "sparkles" => {
            r#"<path d="M10 4c0 3.5 2.5 6 6 6-3.5 0-6 2.5-6 6 0-3.5-2.5-6-6-6 3.5 0 6-2.5 6-6z"/><path d="M18 14.5c0 1.7 1.3 3 3 3-1.7 0-3 1.3-3 3 0-1.7-1.3-3-3-3 1.7 0 3-1.3 3-3z"/>"#
        }
        "chat" => {
            r#"<path d="M5 5.5h14A1.5 1.5 0 0 1 20.5 7v8.5A1.5 1.5 0 0 1 19 17h-7l-4.5 3.5V17H5A1.5 1.5 0 0 1 3.5 15.5V7A1.5 1.5 0 0 1 5 5.5z"/><path d="M17.5 2.5v3M16 4h3"/>"#
        }
        "settings" => {
            r#"<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>"#
        }
        "search" => r#"<circle cx="11" cy="11" r="6.5"/><path d="m20 20-4.2-4.2"/>"#,
        "x" => r#"<path d="m6.5 6.5 11 11M17.5 6.5l-11 11"/>"#,
        "plus" => r#"<path d="M12 5v14M5 12h14"/>"#,
        "chev-down" => r#"<path d="m6 9.5 6 6 6-6"/>"#,
        "chev-left" => r#"<path d="m14.5 6-6 6 6 6"/>"#,
        "chev-right" => r#"<path d="m9.5 6 6 6-6 6"/>"#,
        "pin" => r#"<path d="m14.5 4 5.5 5.5-2.2.9-3 3 .4 3.6-1.3 1.3-3.4-3.4L5 19.5l-.5-.5 4.6-5.5L5.7 10l1.3-1.3 3.6.4 3-3z"/>"#,
        "minus-circle" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M8 12h8"/>"#,
        "more" => r#"<circle cx="5.5" cy="12" r="1.3"/><circle cx="12" cy="12" r="1.3"/><circle cx="18.5" cy="12" r="1.3"/>"#,
        "eye" => r#"<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="2.8"/>"#,
        "eye-off" => {
            r#"<path d="M3 3l18 18"/><path d="M10.2 5.7A9 9 0 0 1 12 5.5c6 0 9.5 6.5 9.5 6.5a16 16 0 0 1-3 3.8M6.3 7.6A15.5 15.5 0 0 0 2.5 12S6 18.5 12 18.5a8.7 8.7 0 0 0 3.3-.7"/><path d="M9.9 10a2.8 2.8 0 0 0 4 4"/>"#
        }
        "history" => r#"<path d="M4 12a8 8 0 1 0 2.5-5.8"/><path d="M4 4v4.5h4.5M12 8v4.5l3 1.8"/>"#,
        "copy" => {
            r#"<rect x="8.5" y="8.5" width="11" height="11" rx="2"/><path d="M15.5 8.5V6.5a2 2 0 0 0-2-2h-7a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h2"/>"#
        }
        "refresh" => r#"<path d="M19.5 12a7.5 7.5 0 1 1-2.3-5.4"/><path d="M19.5 4.5v4h-4"/>"#,
        "sliders" => {
            r#"<path d="M4 7h9M17 7h3M4 12h3M11 12h9M4 17h11M19 17h1"/><circle cx="15" cy="7" r="2"/><circle cx="9" cy="12" r="2"/><circle cx="17" cy="17" r="2"/>"#
        }
        "expand" => r#"<path d="M14 4h6v6M10 20H4v-6M20 4l-7 7M4 20l7-7"/>"#,
        "send" => r#"<path d="M12 19V5M6 11l6-6 6 6"/>"#,
        "terminal" => r#"<rect x="3.5" y="5" width="17" height="14" rx="2.5"/><path d="m7.5 10 3 2-3 2M12.5 15h4"/>"#,
        "trash" => r#"<path d="M5 7h14M9.5 7V5h5v2M7 7l.8 12h8.4L17 7"/>"#,
        "star" => r#"<path d="m12 3.5 2.6 5.5 6 .8-4.4 4.2 1.1 6-5.3-2.9-5.3 2.9 1.1-6L3.4 9.8l6-.8z"/>"#,
        "help" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M9.7 9.6a2.4 2.4 0 1 1 3.5 2.1c-.8.5-1.2 1-1.2 1.8M12 16.4h.01"/>"#,
        "info" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5M12 8h.01"/>"#,
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

/// The Glyphra mark. Colours come from the theme variables, so one markup serves both themes.
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
                }
            }
        }
    }
}
