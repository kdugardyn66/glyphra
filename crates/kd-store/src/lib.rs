//! kd-store: everything the start screen needs that is not UI.
//!
//! The module catalogue, the Crafting Apps catalogue, the recent/starred library
//! (SQLite), user settings (`home.json`) and file helpers. This crate never depends on a
//! UI toolkit, so the CLI and the server can reuse it.

pub mod craft;
pub mod db;
pub mod doctype;
pub mod files;
pub mod installed;
pub mod library;
pub mod modules;
pub mod settings;
pub mod store;

pub use craft::{CRAFT_APPS, CraftApp, LAUNCHER_REPO, SOLIDCRAFT_REPO};
pub use db::LibraryDb;
pub use doctype::{DocGroup, DocKind, Opener, all_extensions, open_dialog_types};
pub use files::{
    FileEntry, blank_drawing, create_folder, create_new_document, duplicate_file, human_size, list_documents, list_subfolders, now,
    relative_time, rename_file, rename_folder, unique_path,
};
pub use installed::{InstalledApp, Launch, detect_installed};
pub use library::{Library, RecentEntry};
pub use modules::ModuleKind;
pub use settings::{OpenIn, Settings, StoredShortcut, ThemePref};
pub use store::{HomeState, Store};

/// The source code on GitHub (placeholder until the repository exists).
pub const SOURCE_REPO: &str = "https://github.com/kdugardyn66/glyphra";
/// Product name shown in the UI.
pub const APP_NAME: &str = "Glyphra";
/// Version from Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
