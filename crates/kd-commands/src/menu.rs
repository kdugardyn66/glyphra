//! The menu bar as data.
//!
//! The shell turns this into the native menu (macOS menu bar, window menu on Linux and
//! Windows); tests here prove that every entry is a real command with a label and that no
//! shortcut is shown twice. Labels are Fluent ids: `menu-file` for a menu, [`Command::label_key`]
//! for a command, [`Native::label_key`] for a system item.

use crate::{Command, Page, SettingsSection};
use kd_store::{ModuleKind, ThemePref};

/// Items the operating system or the web view implements itself. Edit commands must stay
/// native on macOS: the web view's text fields only get them through the responder chain.
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
}

impl Native {
    pub fn label_key(self) -> &'static str {
        match self {
            Native::Undo => "native-undo",
            Native::Redo => "native-redo",
            Native::Cut => "native-cut",
            Native::Copy => "native-copy",
            Native::Paste => "native-paste",
            Native::SelectAll => "native-select-all",
            Native::Minimize => "native-minimize",
            Native::Zoom => "native-zoom",
            Native::Fullscreen => "native-fullscreen",
            Native::BringAllToFront => "native-bring-all-to-front",
            Native::Hide => "native-hide",
            Native::HideOthers => "native-hide-others",
            Native::ShowAll => "native-show-all",
            Native::Services => "native-services",
        }
    }
}

/// One line of a menu.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    /// Runs a command; the shortcut shown comes from the key map.
    Command(Command),
    /// Same, with an explicit Fluent id for the label (the same command under another name).
    Labelled(&'static str, Command),
    Native(Native),
    Separator,
    Submenu(Menu),
}

/// A menu: its Fluent id and entries.
#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    /// `menu-file`, `menu-new` …
    pub label_key: &'static str,
    pub entries: Vec<Entry>,
}

impl Menu {
    /// Every command in this menu and its submenus, in order.
    pub fn commands(&self) -> Vec<&Command> {
        let mut out = Vec::new();
        for e in &self.entries {
            match e {
                Entry::Command(c) | Entry::Labelled(_, c) => out.push(c),
                Entry::Submenu(m) => out.extend(m.commands()),
                Entry::Native(_) | Entry::Separator => {}
            }
        }
        out
    }
}

/// Where the Help menu sends people (placeholders until the site exists).
pub const HELP_URL: &str = "https://github.com/kdugardyn66/glyphra#readme";
pub const ISSUES_URL: &str = "https://github.com/kdugardyn66/glyphra/issues";

/// One line of the Window menu's list of open windows and tabs.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowItem {
    /// Shown as is: the document's file name, or the app name for the home window.
    pub title: String,
    /// What choosing it does: [`Command::ActivateTab`] or [`Command::OpenPath`] (which brings
    /// that document's window to the front).
    pub command: Command,
    pub checked: bool,
}

/// What the native menu shows that depends on the window's state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MenuView {
    /// A document is in front: Save, Save As and Print are available.
    pub document: bool,
    /// The home window, then every open tab and document window.
    pub windows: Vec<WindowItem>,
}

/// The id a native menu item carries: the command id, plus the address for link items.
pub fn menu_id(c: &Command) -> String {
    match c {
        Command::OpenUrl(u) => format!("app.open_url:{u}"),
        other => other.id(),
    }
}

/// The id of an item in the Window menu's list (these carry a tab number or a file path).
pub fn window_item_id(c: &Command) -> String {
    match c {
        Command::ActivateTab(n) => format!("app.window:tab:{n}"),
        Command::OpenPath(p) => format!("app.window:file:{}", p.display()),
        other => other.id(),
    }
}

/// The command behind a native menu item's id (the inverse of [`menu_id`] and [`window_item_id`]).
pub fn command_for_menu_id(id: &str) -> Option<Command> {
    if let Some(rest) = id.strip_prefix("app.window:") {
        if let Some(n) = rest.strip_prefix("tab:") {
            return n.parse().ok().map(Command::ActivateTab);
        }
        return rest.strip_prefix("file:").map(|p| Command::OpenPath(p.into()));
    }
    if let Some(url) = id.strip_prefix("app.open_url:") {
        return Some(Command::OpenUrl(url.to_string()));
    }
    Command::from_id(id)
}

/// The menu bar: app (macOS only), File, Edit, View, Window, Help.
pub fn menu_bar(mac: bool) -> Vec<Menu> {
    use Entry::{Command as C, Native as N, Separator as Sep};
    let mut bar = Vec::new();
    if mac {
        bar.push(Menu {
            label_key: "menu-app",
            entries: vec![
                C(Command::OpenSettings(SettingsSection::About)),
                Sep,
                C(Command::OpenSettings(SettingsSection::General)),
                Sep,
                N(Native::Services),
                Sep,
                N(Native::Hide),
                N(Native::HideOthers),
                N(Native::ShowAll),
                Sep,
                C(Command::Quit),
            ],
        });
    }
    let new = Menu { label_key: "menu-new", entries: ModuleKind::ALL.into_iter().map(|k| C(Command::NewDocument(k))).collect() };
    // As on a Mac: New ▸, Open…, Close, Save, Save As…, Print… (Save, Save As and Print are
    // greyed out on the home screen). The other file commands keep their shortcuts and live
    // on the start screen.
    let mut file = vec![
        Entry::Submenu(new),
        C(Command::OpenDialog),
        Sep,
        Entry::Labelled("menu-close", Command::CloseActiveTab),
        C(Command::Save),
        C(Command::SaveAs),
        Sep,
        C(Command::Print),
    ];
    if !mac {
        file.extend([Sep, C(Command::OpenSettings(SettingsSection::General)), Sep, C(Command::Quit)]);
    }
    bar.push(Menu { label_key: "menu-file", entries: file });
    bar.push(Menu {
        label_key: "menu-edit",
        entries: vec![
            N(Native::Undo),
            N(Native::Redo),
            Sep,
            N(Native::Cut),
            N(Native::Copy),
            N(Native::Paste),
            N(Native::SelectAll),
            Sep,
            C(Command::FocusSearch),
        ],
    });
    let theme = Menu { label_key: "menu-theme", entries: ThemePref::ALL.into_iter().map(|t| C(Command::SetTheme(t))).collect() };
    bar.push(Menu {
        label_key: "menu-view",
        entries: vec![
            C(Command::ShowPage(Page::Home)),
            C(Command::ShowPage(Page::Recent)),
            C(Command::ShowPage(Page::Starred)),
            C(Command::ShowPage(Page::CraftApps)),
            C(Command::ShowPage(Page::AiChat)),
            C(Command::ShowPage(Page::AiTools)),
            Sep,
            Entry::Submenu(theme),
            C(Command::CycleTheme),
            Sep,
            C(Command::Refresh),
            N(Native::Fullscreen),
        ],
    });
    // The list of open windows and tabs follows these entries; the shell adds it (see [`MenuView`]).
    let mut window = vec![N(Native::Minimize), N(Native::Zoom)];
    if mac {
        window.extend([Sep, N(Native::BringAllToFront)]);
    }
    bar.push(Menu { label_key: "menu-window", entries: window });
    bar.push(Menu {
        label_key: "menu-help",
        entries: vec![
            Entry::Labelled("cmd-help-docs", Command::OpenUrl(HELP_URL.into())),
            Entry::Labelled("cmd-help-issues", Command::OpenUrl(ISSUES_URL.into())),
            Sep,
            C(Command::OpenSettings(SettingsSection::Keyboard)),
            C(Command::OpenSettings(SettingsSection::LicenseMit)),
            C(Command::OpenSettings(SettingsSection::LicenseApache)),
        ],
    });
    bar
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Keymap, Os, display_sequence};

    #[test]
    fn the_bar_has_the_six_menus_on_a_mac_and_five_elsewhere() {
        let keys = |bar: &[Menu]| bar.iter().map(|m| m.label_key).collect::<Vec<_>>();
        assert_eq!(keys(&menu_bar(true)), ["menu-app", "menu-file", "menu-edit", "menu-view", "menu-window", "menu-help"]);
        assert_eq!(keys(&menu_bar(false)), ["menu-file", "menu-edit", "menu-view", "menu-window", "menu-help"]);
    }

    #[test]
    fn menu_ids_round_trip_to_the_same_command() {
        for mac in [true, false] {
            for menu in menu_bar(mac) {
                for c in menu.commands() {
                    assert_eq!(command_for_menu_id(&menu_id(c)).as_ref(), Some(c), "{}", c.id());
                }
            }
        }
        assert_eq!(command_for_menu_id("no.such.thing"), None);
    }

    #[test]
    fn window_list_ids_round_trip() {
        for c in [Command::ActivateTab(0), Command::ActivateTab(7), Command::OpenPath("/a b/c:d.md".into())] {
            assert_eq!(command_for_menu_id(&window_item_id(&c)), Some(c));
        }
        assert_eq!(command_for_menu_id("app.window:tab:x"), None);
    }

    #[test]
    fn the_file_menu_matches_the_mac_layout() {
        let file = menu_bar(true).into_iter().find(|m| m.label_key == "menu-file").unwrap();
        let cmds: Vec<String> = file.commands().iter().map(|c| c.id()).collect();
        assert_eq!(
            cmds,
            [
                "file.new.markdown",
                "file.new.pdf",
                "file.new.drawing",
                "file.open",
                "tab.close_active",
                "file.save",
                "file.save_as",
                "file.print"
            ]
        );
        let help = menu_bar(true).into_iter().find(|m| m.label_key == "menu-help").unwrap();
        assert_eq!(help.commands().last().map(|c| c.id()), Some("settings.open.license-apache".to_string()));
    }

    #[test]
    fn every_menu_command_has_a_stable_id_and_label_key() {
        for mac in [true, false] {
            for menu in menu_bar(mac) {
                for c in menu.commands() {
                    assert!(!c.id().is_empty());
                    assert!(c.label_key().starts_with("cmd-"), "{}", c.id());
                    assert!(!c.label_key().contains('.'));
                }
            }
        }
    }

    #[test]
    fn menu_commands_are_bindable_except_the_link_ones() {
        let bindable: Vec<Command> = crate::bindable_commands().into_iter().map(|(c, _)| c).collect();
        for menu in menu_bar(true) {
            for c in menu.commands() {
                if matches!(c, Command::OpenUrl(_)) {
                    continue;
                }
                assert!(bindable.contains(c), "{} is in a menu but cannot be bound", c.id());
            }
        }
    }

    #[test]
    fn standard_shortcuts_are_where_a_mac_user_expects_them() {
        let k = Keymap::defaults(Os::Mac);
        let shown = |c: &Command| display_sequence(k.keys_for(c).unwrap(), Os::Mac);
        assert_eq!(shown(&Command::Quit), "Cmd+Q");
        assert_eq!(shown(&Command::OpenSettings(SettingsSection::General)), "Cmd+,");
        assert_eq!(shown(&Command::OpenDialog), "Cmd+O");
        assert_eq!(shown(&Command::CloseActiveTab), "Cmd+W");
        assert_eq!(shown(&Command::FocusSearch), "Cmd+F");
        assert_eq!(shown(&Command::Save), "Cmd+S");
        assert_eq!(shown(&Command::SaveAs), "Cmd+Shift+S");
        assert_eq!(shown(&Command::Print), "Cmd+P");
    }

    #[test]
    fn a_menu_never_lists_the_same_command_twice() {
        for menu in menu_bar(true) {
            let cmds = menu.commands();
            for (i, a) in cmds.iter().enumerate() {
                assert!(!cmds[i + 1..].contains(a), "{} twice in {}", a.id(), menu.label_key);
            }
        }
    }
}
