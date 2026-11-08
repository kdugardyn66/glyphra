# Glyphra — rules for working in this repository

Glyphra is a workspace for technical writing: Markdown, PDF and Drawing in one app, with Glyphra AI
and team collaboration. Rust and Dioxus (desktop web view), SQLite locally, PostgreSQL on the server.
The plan lives in the "Glyphra — Architecture & Implementation Plan" document; this file holds the
rules both earlier builds (mdCraft, KaDeePlatform) learned the hard way.

## Rules

1. **Everything is a command.** Every user action is a `kd_commands::Command` with a stable id
   (`file.new.markdown`). Menus, buttons, shortcuts, the AI and the CLI all go through
   `AppModel::dispatch`. A menu item without a command is a bug.
2. **No `unwrap`, `expect`, `panic!` or `unreachable!` outside tests.** Return errors; show them to the
   user as a message (`notify_error`), never swallow them.
3. **Never lose work.** Files are written atomically (temp file, then rename). Settings are saved on
   every change, not on exit.
4. **The model has no UI.** `glyphra-shell/src/model.rs` knows nothing about Dioxus. It returns `Effect`s
   (dialogs, Trash, quit) that the platform layer carries out. New behaviour gets a model test first.
5. **Every visible string is a Fluent id.** Add it to `tools/gen_locales.py` for English, Dutch, French
   and German, then run `python3 tools/gen_locales.py`. Tests fail when an id is missing from any
   language or when code uses an id that does not exist.
6. **Languages are data.** A language is a folder with `main.ftl` and `locale.toml`. Do not hard-code
   anything per language; use CSS logical properties (`margin-inline-start`) so right-to-left works.
7. **Colours come from CSS variables** in `glyphra.css`; never hard-code a colour in a component.
8. **Raw HTML from documents is never passed to the web view.** The preview escapes it.
9. No shared crates between this repository and the earlier ones: code is copied once, then owned here.

## Commands

```
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p glyphra                      # GLYPHRA_TRACE=1 prints start-up timings
GLYPHRA_CONFIG_DIR=/tmp/g cargo run -p glyphra   # separate settings and library
```

Linux needs `libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev` to build.

Packaging: `./build.sh` (macOS .app + .dmg, Linux .tar.gz/.deb/.rpm/AppImage, Windows via
`scripts/build-windows.ps1` .zip/.msi) into `dist/`. The bundle identifier `app.glyphra.Glyphra` appears in
the scripts, `packaging/` and `apps/glyphra/Dioxus.toml`: change all of them together.

## Layout

| Path | What |
| --- | --- |
| `crates/kd-commands` | Commands, key maps, the menu bar as data |
| `crates/kd-store` | Settings, Recent and Starred (SQLite), module and Crafting Apps catalogues, file helpers |
| `crates/glyphra-i18n` | Fluent loader, built-in and user language packs, fallback to English |
| `crates/glyphra-shell` | Model, dispatcher and the Dioxus components |
| `apps/glyphra` | The binary and the native menu |
| `locales/` | Generated from `tools/gen_locales.py` |
