//! Glyphra for macOS, Linux and Windows.
// Release builds on Windows are GUI programs: no console window behind the app.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod menu;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use glyphra_i18n::{Catalog, I18n};
use kd_store::Store;

fn main() {
    glyphra_shell::mark_process_start();
    // The menu bar needs its labels before the window exists, so it reads the saved language
    // itself (and the user's language packs). Later changes arrive through the language hook.
    let system = sys_locale::get_locale().unwrap_or_default();
    let store = Store::platform();
    let (catalog, _) = Catalog::load(Some(&store.locales_dir()));
    let bar = menu::build(&I18n::new(&catalog, &store.saved_language(), &system));
    glyphra_shell::set_language_hook(menu::relabel);
    glyphra_shell::set_menu_hook(menu::update);

    let window = WindowBuilder::new()
        .with_title("Glyphra")
        .with_inner_size(LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(LogicalSize::new(900.0, 600.0));
    let config = Config::new()
        .with_window(window)
        .with_menu(bar)
        // the colour behind the splash, so there is no white flash before the first paint
        .with_background_color((23, 24, 27, 255));
    dioxus::LaunchBuilder::desktop().with_cfg(config).launch(glyphra_shell::Root);
}
