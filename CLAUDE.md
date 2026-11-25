# Glyphra — rules for working in this repository

Glyphra is a Markdown workspace: documents and presentations,
with AI and team collaboration. Rust and Dioxus (desktop web view). It consolidates mdCraft (base
functionality), Glyphra 0.2 (AI) and the layout of ONLYOFFICE Desktop Editors (see
`docs/consolidation.md`).

1. **Everything is a command.** User actions are `gl_core::Command`s run by `AppModel::dispatch`.
   Menus, buttons, shortcuts and later the AI all go through it. A menu item without a command is a bug.
2. **The model has no UI.** `gl-core` knows nothing about Dioxus. It returns `Effect`s (dialogs,
   quit) that `gl-ui` carries out. New behaviour gets a model test first.
3. **No `unwrap`, `expect`, `panic!` outside tests.** Return errors and show them with `notify`.
4. **Never lose work.** Files and settings are written with `gl_store::atomic_write`. A damaged
   `settings.json` is kept as `settings.json.bad`, never overwritten.
5. **Raw HTML from documents is never passed to the web view** (`gl_core::preview`).
6. **API keys are stored encrypted (AES-256-GCM) inside `settings.json`**, never as plain text and never in the system keychain; the random key is in `settings.key` beside it (mode 0600). Everything lives in the settings folder (macOS: `~/Library/Application Support/Glyphra/`).
7. **Text fields are uncontrolled** (`initial_value`, keyed by `form_gen`). A field re-filled on every
   keystroke loses characters typed faster than the round trip to Rust.
8. Colours come from CSS variables in `crates/gl-ui/assets/glyphra.css`; no hard-coded colours in components.
9. `deploy/` is the server side only; the desktop app never needs it.

```
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p glyphra            # GLYPHRA_CONFIG_DIR=/tmp/g to use a separate settings folder
```
Linux needs `libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev`.

7. Every user-visible string goes through `gl_core::t` / `tr` / `tx` (English text is the key). Add the new key to all
   full catalogues (es-419, pt-BR, fr-FR, de-DE, it-IT, nl-NL); the i18n test fails otherwise. Template content and
   catalogue/provider text stay English.

8. Dates, times, numbers, currency and units shown to the user go through `gl_core::region` (never `format!("{:.2}")`
   or chrono's own formatting), so they follow Settings › Region.
