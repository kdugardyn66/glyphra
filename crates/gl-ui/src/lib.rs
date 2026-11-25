//! Glyphra's window, built with Dioxus. All behaviour lives in `gl-core`; these components
//! show the model and turn clicks into commands.

mod ai_panel;
mod ai_tools;
mod collaboration;
mod doc;
mod home;
mod icons;
mod open;
mod pick;
mod settings;
mod shell;
mod templates;

/// The stylesheet (design tokens, layout, dark theme).
pub const CSS: &str = include_str!("../assets/glyphra.css");

pub use shell::{Root, run};
