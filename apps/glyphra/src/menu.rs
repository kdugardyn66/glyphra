//! The native menu bar, built from `kd_commands::menu::menu_bar`.
//!
//! macOS shows it at the top of the screen, Linux and Windows at the top of the window. Item
//! ids are command ids, so a click arrives in the window as a [`kd_commands::Command`]. The
//! labels are Fluent ids; [`relabel`] renames every item when the language changes.

use dioxus::desktop::muda::{
    Menu, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};
use glyphra_i18n::I18n;
use kd_commands::menu::{Entry, Menu as Spec, Native, menu_bar, menu_id};
use kd_commands::{Chord, Command, Keymap, Os};
use std::cell::RefCell;

enum Handle {
    Item(MenuItem),
    Predefined(PredefinedMenuItem),
    Submenu(Submenu),
}

struct Labelled {
    handle: Handle,
    key: String,
}

thread_local! {
    /// The menu runs on the main thread together with the window, so a thread-local is enough.
    static ITEMS: RefCell<Vec<Labelled>> = const { RefCell::new(Vec::new()) };
}

/// A chord as a menu accelerator (single chords only; sequences such as Cmd+N M are handled
/// by the New page).
fn accelerator(chord: &Chord) -> Option<Accelerator> {
    let mut mods = Modifiers::empty();
    if chord.ctrl {
        mods |= Modifiers::CONTROL;
    }
    if chord.alt {
        mods |= Modifiers::ALT;
    }
    if chord.shift {
        mods |= Modifiers::SHIFT;
    }
    if chord.meta {
        mods |= Modifiers::SUPER;
    }
    let code = match chord.key.as_str() {
        "Tab" => Code::Tab,
        "Comma" => Code::Comma,
        k if k.len() == 1 => {
            let c = k.chars().next()?.to_ascii_uppercase();
            match c {
                'A'..='Z' => letter(c)?,
                '0'..='9' => digit(c)?,
                _ => return None,
            }
        }
        _ => return None,
    };
    Some(Accelerator::new(if mods.is_empty() { None } else { Some(mods) }, code))
}

fn letter(c: char) -> Option<Code> {
    use Code::*;
    Some(match c {
        'A' => KeyA,
        'B' => KeyB,
        'C' => KeyC,
        'D' => KeyD,
        'E' => KeyE,
        'F' => KeyF,
        'G' => KeyG,
        'H' => KeyH,
        'I' => KeyI,
        'J' => KeyJ,
        'K' => KeyK,
        'L' => KeyL,
        'M' => KeyM,
        'N' => KeyN,
        'O' => KeyO,
        'P' => KeyP,
        'Q' => KeyQ,
        'R' => KeyR,
        'S' => KeyS,
        'T' => KeyT,
        'U' => KeyU,
        'V' => KeyV,
        'W' => KeyW,
        'X' => KeyX,
        'Y' => KeyY,
        'Z' => KeyZ,
        _ => return None,
    })
}

fn digit(c: char) -> Option<Code> {
    use Code::*;
    Some(match c {
        '0' => Digit0,
        '1' => Digit1,
        '2' => Digit2,
        '3' => Digit3,
        '4' => Digit4,
        '5' => Digit5,
        '6' => Digit6,
        '7' => Digit7,
        '8' => Digit8,
        '9' => Digit9,
        _ => return None,
    })
}

/// The accelerator of a command: its first default shortcut, if that is a single chord.
pub fn accelerator_for(cmd: &Command, keymap: &Keymap) -> Option<Accelerator> {
    let keys = keymap.keys_for(cmd)?;
    if keys.len() == 1 { accelerator(&keys[0]) } else { None }
}

/// Build the menu bar for the current system and language.
pub fn build(i18n: &I18n) -> Menu {
    let mac = cfg!(target_os = "macos");
    let keymap = Keymap::defaults(Os::current());
    let menu = Menu::new();
    ITEMS.with(|items| items.borrow_mut().clear());
    for spec in menu_bar(mac) {
        let sub = submenu(&spec, i18n, &keymap);
        let _ = menu.append(&sub);
        #[cfg(target_os = "macos")]
        {
            if spec.label_key == "menu-window" {
                sub.set_as_windows_menu_for_nsapp();
            }
            if spec.label_key == "menu-help" {
                sub.set_as_help_menu_for_nsapp();
            }
        }
    }
    menu
}

fn remember(handle: Handle, key: &str) {
    ITEMS.with(|items| items.borrow_mut().push(Labelled { handle, key: key.to_string() }));
}

fn submenu(spec: &Spec, i18n: &I18n, keymap: &Keymap) -> Submenu {
    let sub = Submenu::new(i18n.t(spec.label_key), true);
    for entry in &spec.entries {
        match entry {
            Entry::Separator => {
                let _ = sub.append(&PredefinedMenuItem::separator());
            }
            Entry::Command(c) => append_item(&sub, c, &c.label_key(), i18n, keymap),
            Entry::Labelled(key, c) => append_item(&sub, c, key, i18n, keymap),
            Entry::Native(n) => {
                let label = i18n.t(n.label_key());
                let item = native(*n, &label);
                let _ = sub.append(&item);
                remember(Handle::Predefined(item), n.label_key());
            }
            Entry::Submenu(inner) => {
                let s = submenu(inner, i18n, keymap);
                let _ = sub.append(&s);
            }
        }
    }
    remember(Handle::Submenu(sub.clone()), spec.label_key);
    sub
}

fn append_item(sub: &Submenu, cmd: &Command, label_key: &str, i18n: &I18n, keymap: &Keymap) {
    let item = MenuItem::with_id(menu_id(cmd), i18n.t(label_key), true, accelerator_for(cmd, keymap));
    let _ = sub.append(&item);
    remember(Handle::Item(item), label_key);
}

fn native(n: Native, label: &str) -> PredefinedMenuItem {
    let l = Some(label);
    match n {
        Native::Undo => PredefinedMenuItem::undo(l),
        Native::Redo => PredefinedMenuItem::redo(l),
        Native::Cut => PredefinedMenuItem::cut(l),
        Native::Copy => PredefinedMenuItem::copy(l),
        Native::Paste => PredefinedMenuItem::paste(l),
        Native::SelectAll => PredefinedMenuItem::select_all(l),
        Native::Minimize => PredefinedMenuItem::minimize(l),
        Native::Zoom => PredefinedMenuItem::maximize(l),
        Native::Fullscreen => PredefinedMenuItem::fullscreen(l),
        Native::BringAllToFront => PredefinedMenuItem::bring_all_to_front(l),
        Native::Hide => PredefinedMenuItem::hide(l),
        Native::HideOthers => PredefinedMenuItem::hide_others(l),
        Native::ShowAll => PredefinedMenuItem::show_all(l),
        Native::Services => PredefinedMenuItem::services(l),
    }
}

/// Rename every menu item for a new language.
pub fn relabel(i18n: &I18n) {
    ITEMS.with(|items| {
        for it in items.borrow().iter() {
            let text = i18n.t(&it.key);
            match &it.handle {
                Handle::Item(i) => i.set_text(text),
                Handle::Predefined(p) => p.set_text(text),
                Handle::Submenu(s) => s.set_text(text),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_single_chord_shortcut_in_the_menus_becomes_an_accelerator() {
        for os in [Os::Mac, Os::Windows, Os::Linux] {
            let keymap = Keymap::defaults(os);
            for mac in [true, false] {
                for menu in menu_bar(mac) {
                    for c in menu.commands() {
                        if let Some(keys) = keymap.keys_for(c)
                            && keys.len() == 1
                        {
                            assert!(accelerator(&keys[0]).is_some(), "{} ({os:?}) has no accelerator", c.id());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn cmd_q_and_cmd_comma_map_to_the_right_keys() {
        let k = Keymap::defaults(Os::Mac);
        let q = accelerator_for(&Command::Quit, &k).unwrap();
        assert_eq!(q, Accelerator::new(Some(Modifiers::SUPER), Code::KeyQ));
        let comma = accelerator_for(&Command::OpenSettings(kd_commands::SettingsSection::General), &k).unwrap();
        assert_eq!(comma, Accelerator::new(Some(Modifiers::SUPER), Code::Comma));
        // sequences are not menu accelerators
        assert!(accelerator_for(&Command::NewDocument(kd_store::ModuleKind::Markdown), &k).is_none());
    }
}
