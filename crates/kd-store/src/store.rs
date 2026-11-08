//! Persistence of the start-screen state: settings in `home.json`, the document library
//! (Recent, Starred) in `library.sqlite3` (see [`crate::db`]).
//!
//! Location: the platform config directory for Glyphra
//! (`~/Library/Application Support/app.Glyphra.Glyphra` on macOS, `~/.config/glyphra` on Linux,
//! `%APPDATA%\Glyphra\Glyphra\config` on Windows). `GLYPHRA_CONFIG_DIR` overrides it.
//!
//! The qualifier and organisation are placeholders until the domain is registered; the
//! constants below are the only place that has to change.

use crate::db::LibraryDb;
use crate::library::Library;
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "home.json";
const WELCOME_NAME: &str = "Welcome to Glyphra.md";
/// Reverse-DNS parts of the config folder (placeholders until the domain exists).
const QUALIFIER: &str = "app";
const ORGANISATION: &str = "Glyphra";
const APPLICATION: &str = "Glyphra";
const WELCOME: &str = include_str!("../assets/welcome.md");

/// Everything the start screen persists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeState {
    pub version: u32,
    pub settings: Settings,
    /// Kept in SQLite (`library.sqlite3`); never written to `home.json`.
    #[serde(default, skip_serializing)]
    pub library: Library,
}

impl HomeState {
    pub fn new(documents_dir: PathBuf) -> Self {
        Self { version: 1, settings: Settings::with_documents_dir(documents_dir), library: Library::default() }
    }
}

/// Reads and writes `home.json` in one directory.
#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    /// The store in the platform config dir (or `GLYPHRA_CONFIG_DIR`).
    pub fn platform() -> Self {
        if let Some(dir) = std::env::var_os("GLYPHRA_CONFIG_DIR") {
            return Self::at(PathBuf::from(dir));
        }
        let dir = directories::ProjectDirs::from(QUALIFIER, ORGANISATION, APPLICATION)
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".glyphra"));
        Self::at(dir)
    }

    pub fn at(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Where the user's own language packs live: `locales/<tag>/` next to `home.json`.
    pub fn locales_dir(&self) -> PathBuf {
        self.dir.join("locales")
    }

    /// The interface language saved in `home.json` (empty = follow the system). Cheap and free
    /// of side effects: the menu bar needs it before the window exists.
    pub fn saved_language(&self) -> String {
        fs::read_to_string(self.file())
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v.pointer("/settings/language").and_then(|l| l.as_str().map(str::to_string)))
            .unwrap_or_default()
    }

    pub fn file(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    pub fn library_file(&self) -> PathBuf {
        self.dir.join(crate::db::FILE_NAME)
    }

    /// Default documents folder: `~/Documents/Glyphra`.
    pub fn default_documents_dir() -> PathBuf {
        directories::UserDirs::new()
            .and_then(|u| u.document_dir().map(Path::to_path_buf).or_else(|| Some(u.home_dir().join("Documents"))))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Glyphra")
    }

    /// Load the state. On a first launch (or when `home.json` is unreadable) create the
    /// defaults, make the documents folder and drop the welcome document into it.
    /// Returns the state and whether this was a first launch.
    pub fn load_or_init(&self) -> (HomeState, bool) {
        let read = fs::read_to_string(self.file()).ok().and_then(|s| serde_json::from_str::<HomeState>(&s).ok());
        let (mut state, first) = match read {
            Some(state) => (state, false),
            None => {
                let state = HomeState::new(Self::default_documents_dir());
                let _ = seed_documents(&state.settings.documents_dir);
                (state, true)
            }
        };
        // The library lives in SQLite.
        if let Ok(db) = LibraryDb::open(&self.library_file())
            && let Ok(lib) = db.load()
        {
            state.library = lib;
        }
        if first {
            let _ = self.save(&state);
        }
        (state, first)
    }

    /// Write `home.json` atomically (temp file + rename) and the library to SQLite.
    pub fn save(&self, state: &HomeState) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let tmp = self.dir.join(format!("{FILE_NAME}.tmp"));
        let json = serde_json::to_string_pretty(state).map_err(io::Error::other)?;
        fs::write(&tmp, json)?;
        fs::rename(tmp, self.file())?;
        let mut db = LibraryDb::open(&self.library_file()).map_err(io::Error::other)?;
        db.save(&state.library).map_err(io::Error::other)
    }
}

/// Create the documents folder with a welcome document if it is new.
pub fn seed_documents(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let welcome = dir.join(WELCOME_NAME);
    if !welcome.exists() {
        fs::write(welcome, WELCOME)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::at(tmp.path().join("cfg"));
        let mut state = HomeState::new(tmp.path().join("docs"));
        state.settings.display_name = "Kris".into();
        state.library.touch(Path::new("/x.md"), 42);
        store.save(&state).unwrap();
        let (loaded, first) = store.load_or_init();
        assert!(!first);
        assert_eq!(loaded, state);
    }

    #[test]
    fn corrupt_file_falls_back_to_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::at(tmp.path().to_path_buf());
        fs::write(store.file(), "{ not json").unwrap();
        let (state, first) = store.load_or_init();
        assert!(first);
        assert_eq!(state.version, 1);
    }

    #[test]
    fn seeding_writes_welcome_once() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("Glyphra");
        seed_documents(&dir).unwrap();
        let w = dir.join(WELCOME_NAME);
        fs::write(&w, "edited").unwrap();
        seed_documents(&dir).unwrap();
        assert_eq!(fs::read_to_string(w).unwrap(), "edited");
    }

    #[test]
    fn the_saved_language_can_be_read_without_loading_anything() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::at(tmp.path().join("cfg"));
        assert_eq!(store.saved_language(), "");
        let mut state = HomeState::new(tmp.path().join("docs"));
        state.settings.language = "nl".into();
        store.save(&state).unwrap();
        assert_eq!(store.saved_language(), "nl");
        assert!(!tmp.path().join("docs").exists(), "reading the language creates nothing");
    }

    #[test]
    fn language_packs_live_next_to_home_json() {
        let store = Store::at(PathBuf::from("/x/app.Glyphra.Glyphra"));
        assert_eq!(store.locales_dir(), PathBuf::from("/x/app.Glyphra.Glyphra/locales"));
    }

    #[test]
    fn recent_and_starred_survive_a_restart() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("cfg");
        let docs = tmp.path().join("docs");
        {
            let store = Store::at(dir.clone());
            let mut state = HomeState::new(docs.clone());
            state.library.touch(Path::new("/a.md"), 5);
            state.library.touch(Path::new("/b.md"), 9);
            state.library.toggle_star(Path::new("/a.md"));
            store.save(&state).unwrap();
        }
        // a new Store on the same folder is what the next launch sees
        let (state, first) = Store::at(dir).load_or_init();
        assert!(!first);
        assert_eq!(state.library.recent.len(), 2);
        assert_eq!(state.library.recent[0].path, PathBuf::from("/b.md"));
        assert!(state.library.is_starred(Path::new("/a.md")));
    }
}
