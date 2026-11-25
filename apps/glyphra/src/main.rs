//! Glyphra for macOS, Linux and Windows.
// Release builds on Windows are GUI programs: no console window behind the app.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod menu;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

fn main() {
    // the language comes first: the native menu is built from it
    gl_core::init_language(&gl_store::Store::platform());
    let window = WindowBuilder::new()
        .with_title("Glyphra")
        .with_inner_size(LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(LogicalSize::new(900.0, 600.0));
    let window = platform(window);
    let config = Config::new()
        .with_window(window)
        .with_menu(menu::build())
        // the colour behind the web view, so there is no white flash before the first paint
        .with_background_color((228, 228, 228, 255));
    dioxus::LaunchBuilder::desktop().with_cfg(config).launch(gl_ui::Root);
}

/// macOS: no native title bar; the tab strip of the page sits next to the traffic lights.
#[cfg(target_os = "macos")]
fn platform(w: WindowBuilder) -> WindowBuilder {
    use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;
    w.with_titlebar_transparent(true).with_fullsize_content_view(true).with_title_hidden(true)
}

#[cfg(not(target_os = "macos"))]
fn platform(w: WindowBuilder) -> WindowBuilder {
    w
}
