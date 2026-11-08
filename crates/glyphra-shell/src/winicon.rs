//! The app icon follows the theme: the light icon on a light screen, the dark one on a dark
//! screen. "System" follows the operating system's appearance (which can itself switch at
//! sunset), so the icon changes with it and Glyphra needs no clock or location.
//!
//! * Windows and Linux: the window (and taskbar) icon is replaced.
//! * macOS: the Dock icon is replaced for as long as Glyphra runs. The icon of the .app in the
//!   Finder is fixed by macOS and stays as it is.

use dioxus::desktop::tao::window::Icon;
use dioxus::prelude::*;
use kd_store::ThemePref;

use crate::model::AppModel;

const LIGHT: &[u8] = include_bytes!("../assets/window-light.png");
const DARK: &[u8] = include_bytes!("../assets/window-dark.png");

/// The PNG of the icon for a light or a dark screen.
pub fn png(dark: bool) -> &'static [u8] {
    if dark { DARK } else { LIGHT }
}

/// Is the screen dark, given the user's choice and what the system says?
pub fn is_dark(pref: ThemePref, system_dark: bool) -> bool {
    match pref {
        ThemePref::Dark => true,
        ThemePref::Light => false,
        ThemePref::System => system_dark,
    }
}

/// PNG to width, height and RGBA bytes.
pub fn decode(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut reader = png::Decoder::new(bytes).read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some((info.width, info.height, buf))
}

/// Put the icon for this theme on the window this is called from.
pub fn apply(dark: bool) {
    if let Some((w, h, rgba)) = decode(png(dark))
        && let Ok(icon) = Icon::from_rgba(rgba, w, h)
    {
        dioxus::desktop::window().window.set_window_icon(Some(icon));
    }
    #[cfg(target_os = "macos")]
    dock(png(dark));
}

#[cfg(target_os = "macos")]
fn dock(png: &[u8]) {
    use objc2::AnyThread;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;
    let Some(mtm) = MainThreadMarker::new() else { return };
    let data = NSData::with_bytes(png);
    if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
        // SAFETY: called on the main thread with a valid image.
        unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&image)) };
    }
}

/// Keep the icon of this window in step with the theme and the system's appearance.
pub fn use_theme_icon(model: Signal<AppModel>) {
    let mut system_dark = use_signal(|| true);
    // The web view knows the system's appearance; ask once and listen for changes.
    use_future(move || async move {
        let mut feed = document::eval(
            "const q = matchMedia('(prefers-color-scheme: dark)'); dioxus.send(q.matches); q.addEventListener('change', e => dioxus.send(e.matches));",
        );
        while let Ok(v) = feed.recv::<bool>().await {
            system_dark.set(v);
        }
    });
    let dark = use_memo(move || is_dark(model.read().state.settings.theme, system_dark()));
    use_effect(move || apply(dark()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_icons_decode_and_differ() {
        for dark in [false, true] {
            let (w, h, rgba) = decode(png(dark)).unwrap();
            assert_eq!((w, h), (256, 256));
            assert_eq!(rgba.len(), 256 * 256 * 4);
        }
        assert_ne!(png(true), png(false));
    }

    #[test]
    fn the_theme_decides_and_system_follows_the_os() {
        assert!(is_dark(ThemePref::Dark, false));
        assert!(!is_dark(ThemePref::Light, true));
        assert!(is_dark(ThemePref::System, true));
        assert!(!is_dark(ThemePref::System, false));
    }
}
