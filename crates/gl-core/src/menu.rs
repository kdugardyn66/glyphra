//! The native menu bar as data. The app turns it into a real menu; the tests here prove that
//! every item is a command, ids are unique and shortcuts are not used twice.

use crate::i18n::tx;
use crate::model::{Command, HelpTopic, Page, SettingsSection};
use gl_templates::Kind;

/// Items the operating system implements itself (edit commands must stay native on macOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Native {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Minimize,
    Zoom,
    Fullscreen,
    BringAllToFront,
    Hide,
    HideOthers,
    ShowAll,
    Services,
    About,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Item { id: &'static str, label: &'static str, accel: Option<&'static str>, command: Command },
    Native(Native),
    Separator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    pub label: &'static str,
    pub entries: Vec<Entry>,
}

fn item(id: &'static str, label: &'static str, accel: Option<&'static str>, command: Command) -> Entry {
    Entry::Item { id, label, accel, command }
}

/// The menu bar. On macOS the first menu is the application menu (About, Settings…, Quit);
/// elsewhere Settings and Quit live in File.
pub fn menu_bar(mac: bool) -> Vec<Menu> {
    let settings = item("app.settings", tx("Settings…"), Some("CmdOrCtrl+,"), Command::OpenSettings(SettingsSection::General));
    let quit = item("app.quit", if mac { tx("Quit Glyphra") } else { tx("Quit") }, Some("CmdOrCtrl+Q"), Command::Quit);
    let mut bar = Vec::new();
    if mac {
        bar.push(Menu {
            label: "Glyphra",
            entries: vec![
                Entry::Native(Native::About),
                Entry::Separator,
                settings.clone(),
                Entry::Separator,
                Entry::Native(Native::Services),
                Entry::Separator,
                Entry::Native(Native::Hide),
                Entry::Native(Native::HideOthers),
                Entry::Native(Native::ShowAll),
                Entry::Separator,
                quit.clone(),
            ],
        });
    }
    let mut file = vec![
        item("file.new.document", tx("New Document"), Some("CmdOrCtrl+N"), Command::NewDocument(Kind::Document)),
        item("file.new.presentation", tx("New Presentation"), None, Command::NewDocument(Kind::Presentation)),
        item("file.templates", tx("New from Template…"), None, Command::ShowPage(Page::Templates)),
        Entry::Separator,
        item("file.open", tx("Open…"), Some("CmdOrCtrl+O"), Command::OpenDialog(None)),
        Entry::Separator,
        item("file.close", tx("Close Tab"), Some("CmdOrCtrl+W"), Command::CloseActiveTab),
    ];
    if !mac {
        file.extend([Entry::Separator, settings, Entry::Separator, quit]);
    }
    bar.push(Menu { label: tx("File"), entries: file });
    bar.push(Menu {
        label: tx("Edit"),
        entries: vec![
            Entry::Native(Native::Undo),
            Entry::Native(Native::Redo),
            Entry::Separator,
            Entry::Native(Native::Cut),
            Entry::Native(Native::Copy),
            Entry::Native(Native::Paste),
            Entry::Native(Native::SelectAll),
        ],
    });
    bar.push(Menu {
        label: tx("View"),
        entries: vec![
            item("view.home", tx("Home"), Some("CmdOrCtrl+1"), Command::ShowPage(Page::Home)),
            item("view.open", tx("Open"), Some("CmdOrCtrl+2"), Command::ShowPage(Page::Open)),
            item("view.templates", tx("Templates"), Some("CmdOrCtrl+3"), Command::ShowPage(Page::Templates)),
            item("view.collaboration", tx("Collaboration"), Some("CmdOrCtrl+4"), Command::ShowPage(Page::Collaboration)),
            item("view.ai-tools", tx("AI Tools"), Some("CmdOrCtrl+5"), Command::ShowPage(Page::AiTools)),
            Entry::Separator,
            item("view.ai-chat", tx("AI Chat"), Some("CmdOrCtrl+Alt+A"), Command::ToggleAiPanel),
            Entry::Separator,
            Entry::Native(Native::Fullscreen),
        ],
    });
    bar.push(Menu {
        label: tx("Window"),
        entries: vec![
            Entry::Native(Native::Minimize),
            Entry::Native(Native::Zoom),
            Entry::Separator,
            Entry::Native(Native::BringAllToFront),
        ],
    });
    bar.push(Menu {
        label: tx("Help"),
        entries: vec![item("help.manual", tx("Glyphra Help"), Some("F1"), Command::OpenHelp(HelpTopic::Glyphra))],
    });
    bar
}

/// The command behind a menu item id.
pub fn command_for_menu_id(id: &str) -> Option<Command> {
    [true, false].iter().flat_map(|mac| menu_bar(*mac)).flat_map(|m| m.entries).find_map(|e| match e {
        Entry::Item { id: i, command, .. } if i == id => Some(command),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_and_shortcuts_are_unique_per_platform() {
        for mac in [true, false] {
            let mut ids = HashSet::new();
            let mut keys = HashSet::new();
            for e in menu_bar(mac).into_iter().flat_map(|m| m.entries) {
                if let Entry::Item { id, accel, .. } = e {
                    assert!(ids.insert(id), "duplicate id {id}");
                    if let Some(a) = accel {
                        assert!(keys.insert(a), "duplicate shortcut {a}");
                    }
                }
            }
        }
    }

    #[test]
    fn settings_and_quit_exist_on_every_platform() {
        for mac in [true, false] {
            let all: Vec<_> = menu_bar(mac).into_iter().flat_map(|m| m.entries).collect();
            for id in ["app.settings", "app.quit", "file.open", "file.new.document"] {
                assert!(all.iter().any(|e| matches!(e, Entry::Item { id: i, .. } if *i == id)), "{id} missing (mac={mac})");
            }
        }
        assert_eq!(menu_bar(true)[0].label, "Glyphra");
        assert_eq!(menu_bar(false)[0].label, "File");
    }

    #[test]
    fn ids_map_back_to_commands() {
        assert_eq!(command_for_menu_id("view.templates"), Some(Command::ShowPage(Page::Templates)));
        assert_eq!(command_for_menu_id("nope"), None);
    }
}
