//! Where the files live: the settings folder (`settings.json`, `library.json`).

use crate::files::atomic_write;
use crate::library::Library;
use crate::settings::Settings;
use directories::{ProjectDirs, UserDirs};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    /// The platform's settings folder (macOS: `~/Library/Application Support/Glyphra`); `GLYPHRA_CONFIG_DIR` overrides it (handy for tests and
    /// for running two copies side by side).
    pub fn platform() -> Self {
        if let Some(d) = std::env::var_os("GLYPHRA_CONFIG_DIR") {
            return Self::at(PathBuf::from(d));
        }
        #[cfg(target_os = "macos")]
        if let Some(u) = UserDirs::new() {
            return Self::at(u.home_dir().join("Library/Application Support/Glyphra"));
        }
        let dir = ProjectDirs::from("app", "glyphra", "Glyphra")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".glyphra"));
        Self::at(dir)
    }

    pub fn at(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// `~/Documents/Glyphra`.
    pub fn default_documents_dir() -> PathBuf {
        UserDirs::new()
            .and_then(|u| u.document_dir().map(Path::to_path_buf))
            .or_else(|| UserDirs::new().map(|u| u.home_dir().join("Documents")))
            .unwrap_or_else(|| PathBuf::from("Documents"))
            .join("Glyphra")
    }

    fn settings_file(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    fn library_file(&self) -> PathBuf {
        self.dir.join("library.json")
    }

    /// Saved settings, or the defaults. A damaged file is kept as `settings.json.bad` rather
    /// than overwritten, so nothing the user typed is lost.
    pub fn load_settings(&self) -> Settings {
        let mut s = match std::fs::read_to_string(self.settings_file()) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| {
                let _ = std::fs::rename(self.settings_file(), self.dir.join("settings.json.bad"));
                Settings::default()
            }),
            Err(_) => Settings::default(),
        };
        if s.documents_dir.as_os_str().is_empty() {
            s.documents_dir = Self::default_documents_dir();
        }
        s
    }

    pub fn save_settings(&self, s: &Settings) -> io::Result<()> {
        let json = serde_json::to_vec_pretty(s).map_err(io::Error::other)?;
        atomic_write(&self.settings_file(), &json)
    }

    pub fn load_library(&self) -> Library {
        std::fs::read_to_string(self.library_file()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn save_library(&self, l: &Library) -> io::Result<()> {
        let json = serde_json::to_vec_pretty(l).map_err(io::Error::other)?;
        atomic_write(&self.library_file(), &json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_and_damage_is_kept() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::at(d.path().to_path_buf());
        let mut s = store.load_settings();
        assert!(s.documents_dir.ends_with("Glyphra"));
        s.user_name = "Kris".into();
        store.save_settings(&s).unwrap();
        assert_eq!(store.load_settings().user_name, "Kris");
        std::fs::write(d.path().join("settings.json"), "{ nope").unwrap();
        assert_eq!(store.load_settings().user_name, "");
        assert!(d.path().join("settings.json.bad").exists());
    }

    #[test]
    fn library_round_trip() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::at(d.path().to_path_buf());
        let mut l = store.load_library();
        l.touch_file(Path::new("/a/b.md"), 5);
        store.save_library(&l).unwrap();
        assert_eq!(store.load_library(), l);
    }
}
