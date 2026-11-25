//! Glyphra's model. Every user action is a [`Command`] run by [`AppModel::dispatch`], which
//! returns [`Effect`]s (dialogs, quit) for the platform layer to carry out. Nothing in this
//! crate knows about Dioxus.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod chat;
pub mod i18n;
pub mod menu;
pub mod model;
pub mod models_api;
pub mod preview;
pub mod providers;
pub mod region;

pub use i18n::{LANGUAGES, t, tr, tx};
pub use menu::{command_for_menu_id, menu_bar};
pub use model::{
    AddModelForm, AppModel, Command, CustomProviderForm, Effect, GeneralDraft, HelpTopic, McpEditor, ModelChoice, Notice, NoticeKind,
    OPENABLE, Page, Row, SettingsSection, Step, TEMPLATES_PER_PAGE, Tab, TemplateView, TemplatesState, init_language,
};
