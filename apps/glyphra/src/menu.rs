//! The native menu bar, built from `gl_core::menu_bar`. macOS shows it at the top of the
//! screen, Linux and Windows at the top of the window. Item ids are the ids in `gl_core`, so a
//! click arrives in the window as a command.

use dioxus::desktop::muda::{Menu, MenuItem, PredefinedMenuItem, Submenu, accelerator::Accelerator};
use gl_core::menu::{Entry, Native, menu_bar};

pub fn build() -> Menu {
    let bar = Menu::new();
    for spec in menu_bar(cfg!(target_os = "macos")) {
        let sub = Submenu::new(gl_core::t(spec.label), true);
        for entry in spec.entries {
            let _ = match entry {
                Entry::Separator => sub.append(&PredefinedMenuItem::separator()),
                Entry::Native(n) => sub.append(&native(n)),
                Entry::Item { id, label, accel, .. } => {
                    let accel = accel.and_then(|a| a.parse::<Accelerator>().ok());
                    sub.append(&MenuItem::with_id(id, gl_core::t(label), true, accel))
                }
            };
        }
        let _ = bar.append(&sub);
    }
    bar
}

fn native(n: Native) -> PredefinedMenuItem {
    match n {
        Native::Undo => PredefinedMenuItem::undo(None),
        Native::Redo => PredefinedMenuItem::redo(None),
        Native::Cut => PredefinedMenuItem::cut(None),
        Native::Copy => PredefinedMenuItem::copy(None),
        Native::Paste => PredefinedMenuItem::paste(None),
        Native::SelectAll => PredefinedMenuItem::select_all(None),
        Native::Minimize => PredefinedMenuItem::minimize(None),
        Native::Zoom => PredefinedMenuItem::maximize(None),
        Native::Fullscreen => PredefinedMenuItem::fullscreen(None),
        Native::BringAllToFront => PredefinedMenuItem::bring_all_to_front(None),
        Native::Hide => PredefinedMenuItem::hide(None),
        Native::HideOthers => PredefinedMenuItem::hide_others(None),
        Native::ShowAll => PredefinedMenuItem::show_all(None),
        Native::Services => PredefinedMenuItem::services(None),
        Native::About => PredefinedMenuItem::about(Some("About Glyphra"), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_accelerator_in_the_menu_data_parses() {
        for mac in [true, false] {
            for e in menu_bar(mac).into_iter().flat_map(|m| m.entries) {
                if let Entry::Item { id, accel: Some(a), .. } = e {
                    assert!(a.parse::<Accelerator>().is_ok(), "{id}: {a}");
                }
            }
        }
    }
}
