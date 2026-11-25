//! File helpers: atomic writes, unique names and relative times.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch.
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Write `bytes` to `path` without ever leaving a half-written file: temp file, then rename.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = dir.join(format!(".{name}.tmp"));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
}

/// `dir/base.ext`, or `dir/base 2.ext`, `base 3.ext` … when that exists.
pub fn unique_path(dir: &Path, base: &str, ext: &str) -> PathBuf {
    let first = dir.join(format!("{base}.{ext}"));
    if !first.exists() {
        return first;
    }
    (2u32..).map(|n| dir.join(format!("{base} {n}.{ext}"))).find(|p| !p.exists()).unwrap_or(first)
}

/// "just now", "5 minutes ago", "1 hour ago", "2 days ago" …
pub fn human_time(then: u64, now: u64) -> String {
    let secs = now.saturating_sub(then);
    let (n, unit) = match secs {
        0..=59 => return "just now".into(),
        60..=3599 => (secs / 60, "minute"),
        3600..=86_399 => (secs / 3600, "hour"),
        86_400..=2_591_999 => (secs / 86_400, "day"),
        2_592_000..=31_535_999 => (secs / 2_592_000, "month"),
        _ => (secs / 31_536_000, "year"),
    };
    format!("{n} {unit}{} ago", if n == 1 { "" } else { "s" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_names_count_up() {
        let d = tempfile::tempdir().unwrap();
        let a = unique_path(d.path(), "Untitled", "md");
        assert!(a.ends_with("Untitled.md"));
        std::fs::write(&a, "x").unwrap();
        assert!(unique_path(d.path(), "Untitled", "md").ends_with("Untitled 2.md"));
    }

    #[test]
    fn atomic_write_replaces_and_leaves_no_temp() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a/b.json");
        atomic_write(&p, b"one").unwrap();
        atomic_write(&p, b"two").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "two");
        assert_eq!(std::fs::read_dir(p.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn times_read_naturally() {
        assert_eq!(human_time(100, 130), "just now");
        assert_eq!(human_time(0, 3600), "1 hour ago");
        assert_eq!(human_time(0, 7300), "2 hours ago");
        assert_eq!(human_time(0, 86_400 * 3), "3 days ago");
    }
}
