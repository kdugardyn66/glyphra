//! Recent files and recent folders (the Home and Open pages).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MAX_RECENT: usize = 50;
pub const MAX_FOLDERS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentEntry {
    pub path: PathBuf,
    /// Unix seconds of the last open.
    pub opened: u64,
    /// Kept at the top of the list and never pushed out by newer files.
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub files: Vec<RecentEntry>,
    #[serde(default)]
    pub folders: Vec<RecentEntry>,
}

fn touch(list: &mut Vec<RecentEntry>, path: &Path, now: u64, max: usize) {
    let pinned = list.iter().find(|e| e.path == path).is_some_and(|e| e.pinned);
    list.retain(|e| e.path != path);
    list.insert(0, RecentEntry { path: path.to_path_buf(), opened: now, pinned });
    // the oldest entries that are not pinned make room
    while list.len() > max {
        match list.iter().rposition(|e| !e.pinned) {
            Some(i) => {
                list.remove(i);
            }
            None => break,
        }
    }
}

impl Library {
    /// A file was opened: it moves to the top, and so does its folder.
    pub fn touch_file(&mut self, path: &Path, now: u64) {
        touch(&mut self.files, path, now, MAX_RECENT);
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            touch(&mut self.folders, dir, now, MAX_FOLDERS);
        }
    }

    pub fn touch_folder(&mut self, dir: &Path, now: u64) {
        touch(&mut self.folders, dir, now, MAX_FOLDERS);
    }

    /// Drop folders that are gone; true when anything was dropped. Files that are gone stay in
    /// the list (shown light) until you clean up.
    pub fn prune(&mut self) -> bool {
        let before = self.folders.len();
        self.folders.retain(|e| e.path.is_dir());
        before != self.folders.len()
    }

    /// Drop the files that no longer exist; how many were dropped.
    pub fn cleanup_files(&mut self) -> usize {
        let before = self.files.len();
        self.files.retain(|e| e.path.is_file());
        before - self.files.len()
    }

    /// Pin or unpin a file; false when it is not in the list.
    pub fn toggle_pin(&mut self, path: &Path) -> bool {
        match self.files.iter_mut().find(|e| e.path == path) {
            Some(e) => {
                e.pinned = !e.pinned;
                true
            }
            None => false,
        }
    }

    /// Empty the list of files (folders stay); how many were dropped.
    pub fn clear_files(&mut self) -> usize {
        std::mem::take(&mut self.files).len()
    }

    pub fn forget_file(&mut self, path: &Path) {
        self.files.retain(|e| e.path != path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_a_file_records_its_folder() {
        let mut l = Library::default();
        l.touch_file(Path::new("/a/b/one.md"), 1);
        l.touch_file(Path::new("/a/c/two.md"), 2);
        l.touch_file(Path::new("/a/b/one.md"), 3);
        assert_eq!(l.files.len(), 2);
        assert_eq!(l.files[0].path, Path::new("/a/b/one.md"));
        assert_eq!(l.folders.iter().map(|e| e.path.clone()).collect::<Vec<_>>(), [Path::new("/a/b"), Path::new("/a/c")]);
    }

    #[test]
    fn lists_are_capped_and_pruned() {
        let mut l = Library::default();
        for i in 0..80 {
            l.touch_file(Path::new(&format!("/x/{i}.md")), i);
        }
        assert_eq!(l.files.len(), MAX_RECENT);
        assert!(l.prune());
        assert!(l.folders.is_empty());
        assert_eq!(l.files.len(), MAX_RECENT, "files that are gone stay until cleanup");
        assert_eq!(l.cleanup_files(), MAX_RECENT);
        assert!(l.files.is_empty());
    }

    #[test]
    fn pinned_files_survive_newer_ones_and_clear_all_empties_the_list() {
        let mut l = Library::default();
        l.touch_file(Path::new("/x/keep.md"), 1);
        assert!(l.toggle_pin(Path::new("/x/keep.md")));
        assert!(!l.toggle_pin(Path::new("/x/unknown.md")));
        for i in 0..80 {
            l.touch_file(Path::new(&format!("/x/{i}.md")), 10 + i);
        }
        l.touch_file(Path::new("/x/keep.md"), 500);
        assert_eq!(l.files.len(), MAX_RECENT);
        assert!(l.files.iter().any(|e| e.path == Path::new("/x/keep.md") && e.pinned), "opening keeps the pin");
        assert_eq!(l.clear_files(), MAX_RECENT);
        assert!(l.files.is_empty() && !l.folders.is_empty());
    }

    #[test]
    fn old_files_without_the_pin_field_still_load() {
        let l: Library = serde_json::from_str(r#"{"files":[{"path":"/a.md","opened":3}]}"#).unwrap_or_default();
        assert_eq!(l.files.len(), 1);
        assert!(!l.files[0].pinned);
    }
}
