//! Persistence for Glyphra: settings and the library of recent files and folders are JSON
//! files written atomically; API keys are stored encrypted inside `settings.json` ([`secrets`]).
//!
//! Local data is SQLite from version 0.3 (offline cache for collaboration); until then plain
//! JSON keeps the first versions easy to inspect and to back up.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod files;
pub mod library;
pub mod region;
pub mod secrets;
pub mod settings;
mod store;

pub use files::{atomic_write, human_time, now, unique_path};
pub use library::{Library, RecentEntry};
pub use region::{CurrencyPattern, Length, RegionFormat, RegionSettings, Temperature, Weekday};
pub use settings::{
    AI_TASKS, Appearance, CustomProvider, ModelProfile, OpenIn, Settings, SpellingDetection, ThemeFamily, mcp_server_names, validate_mcp,
};
pub use store::Store;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
