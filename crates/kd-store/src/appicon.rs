//! The icons of installed Crafting Apps, so the start screen shows each app's own icon.
//!
//! The icon is read from the install itself; nothing is bundled or downloaded:
//! - **Linux:** the icon theme (`hicolor`, Flatpak exports, pixmaps), by bundle id or app id.
//! - **macOS:** the `.icns` named in the bundle's `Info.plist`, turned into a PNG by `sips`
//!   (cached in the Glyphra settings folder).
//! - **Windows:** a `.png` or `.ico` next to the program, when there is one. Icons inside an
//!   `.exe` are not read; the app then shows its badge.
//!
//! [`icon_data_uri`] returns a `data:` URI ready for an `<img>`, or `None` (use the badge).

use crate::installed::{InstalledApp, Launch};
use std::path::{Path, PathBuf};

/// Icons bigger than this are ignored (a data URI is held in memory for every tile).
const MAX_BYTES: u64 = 600_000;

/// The icon of an installed app as a `data:` URI. `cache` is a folder Glyphra may write to.
pub fn icon_data_uri(app: &InstalledApp, cache: &Path) -> Option<String> {
    let file = find_icon_file(app, cache)?;
    data_uri(&file)
}

fn find_icon_file(app: &InstalledApp, cache: &Path) -> Option<PathBuf> {
    match &app.launch {
        Launch::MacBundle(bundle) => mac_icon(app, bundle, cache),
        Launch::Program(exe) => {
            if cfg!(windows) {
                beside_exe(app, exe)
            } else {
                linux_icon(app, &icon_roots())
            }
        }
        Launch::Flatpak(_) => linux_icon(app, &icon_roots()),
    }
}

fn names_for(app: &InstalledApp) -> Vec<String> {
    let mut v = vec![app.app.bundle_id.to_string(), app.app.id.to_string()];
    for a in app.app.aliases {
        v.push(a.to_ascii_lowercase());
    }
    v
}

/// Folders holding icon themes on Linux.
fn icon_roots() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(b) = directories::BaseDirs::new() {
        v.push(b.data_dir().join("icons"));
        v.push(b.data_dir().join("flatpak/exports/share/icons"));
        v.push(b.home_dir().join(".icons"));
    }
    v.extend(["/usr/share/icons", "/usr/local/share/icons", "/var/lib/flatpak/exports/share/icons"].map(PathBuf::from));
    v
}

/// Largest PNG (up to 512 px) or the scalable SVG of the app in the icon themes.
pub fn linux_icon(app: &InstalledApp, roots: &[PathBuf]) -> Option<PathBuf> {
    let names = names_for(app);
    let sizes = [512, 256, 128, 96, 64, 48];
    for root in roots {
        for theme in ["hicolor", ""] {
            let base = if theme.is_empty() { root.clone() } else { root.join(theme) };
            for name in &names {
                for s in sizes {
                    let p = base.join(format!("{s}x{s}/apps/{name}.png"));
                    if p.is_file() {
                        return Some(p);
                    }
                }
                let svg = base.join(format!("scalable/apps/{name}.svg"));
                if svg.is_file() {
                    return Some(svg);
                }
            }
        }
    }
    for dir in ["/usr/share/pixmaps", "/usr/local/share/pixmaps"] {
        for name in &names {
            let p = Path::new(dir).join(format!("{name}.png"));
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn beside_exe(app: &InstalledApp, exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    for name in names_for(app) {
        for ext in ["png", "svg"] {
            let p = dir.join(format!("{name}.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// macOS: convert the bundle's `.icns` to a 128 px PNG once, then reuse it.
fn mac_icon(app: &InstalledApp, bundle: &Path, cache: &Path) -> Option<PathBuf> {
    let out = cache.join(format!("{}.png", app.app.id));
    if out.is_file() {
        return Some(out);
    }
    let plist = plist::Value::from_file(bundle.join("Contents/Info.plist")).ok()?;
    let name = plist.as_dictionary()?.get("CFBundleIconFile")?.as_string()?.to_string();
    let res = bundle.join("Contents/Resources");
    let icns = [res.join(&name), res.join(format!("{name}.icns"))].into_iter().find(|p| p.is_file())?;
    std::fs::create_dir_all(cache).ok()?;
    let status = std::process::Command::new("sips")
        .args(["-s", "format", "png", "-z", "128", "128"])
        .arg(&icns)
        .arg("--out")
        .arg(&out)
        .output()
        .ok()?;
    (status.status.success() && out.is_file()).then_some(out)
}

fn data_uri(file: &Path) -> Option<String> {
    let mime = match file.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    if std::fs::metadata(file).ok()?.len() > MAX_BYTES {
        return None;
    }
    let bytes = std::fs::read(file).ok()?;
    Some(format!("data:{mime};base64,{}", base64(&bytes)))
}

/// Standard base64 (RFC 4648) with padding.
pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (u32::from(c[0]) << 16) | (u32::from(*c.get(1).unwrap_or(&0)) << 8) | u32::from(*c.get(2).unwrap_or(&0));
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::craft::CRAFT_APPS;

    #[test]
    fn base64_matches_the_rfc_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn finds_the_largest_png_in_an_icon_theme() {
        let tmp = tempfile::tempdir().unwrap();
        let app = &CRAFT_APPS[0];
        for s in ["64x64", "256x256"] {
            let d = tmp.path().join(format!("hicolor/{s}/apps"));
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join(format!("{}.png", app.id)), b"x").unwrap();
        }
        let installed = InstalledApp { app, version: None, launch: Launch::Program("/bin/x".into()), source: "t" };
        let found = linux_icon(&installed, &[tmp.path().to_path_buf()]).unwrap();
        assert!(found.to_string_lossy().contains("256x256"));
    }
}
