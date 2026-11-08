//! Glyphra's window: splash, start screen, tabs, settings and the command dispatcher.
//!
//! * [`model`] holds the state and runs every [`kd_commands::Command`] (no UI, fully tested).
//! * [`app`] is the root component: splash while things load, then the window.
//! * The other modules are Dioxus components that only read the model and send commands.

pub mod ai;
mod ai_chat;
mod ai_tools;
pub mod app;
mod doc;
mod home;
mod icons;
mod modal;
pub mod model;
mod preview;
mod settings;
mod sidebar;
mod tabs;

pub use app::{DocWindow, Root, mark_process_start, set_language_hook, set_menu_hook};
pub use model::{AppModel, Effect};

/// The stylesheet with the design tokens and all component styles.
pub const CSS: &str = include_str!("../assets/glyphra.css");
