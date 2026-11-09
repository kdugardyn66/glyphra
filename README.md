# Glyphra

Markdown, PDF and Drawing in one workspace, with Glyphra AI and team collaboration on your own server.

**Version 0.2.4** — the macOS .dmg builds again, with a new window background; AI Tools can draft a document or a slide deck from a description (Markdown now, PDF with 0.7.0); the app icon follows the theme (light or dark, System follows the OS); Glyphra AI (providers in Settings › AI providers by region, with Test connection; models in Settings › AI models): an **AI Chat** page and an **AI Tools** page in the sidebar, and Settings › AI with providers (Anthropic, Gemini, OpenAI, Mistral, local Ollama and LM Studio, any OpenAI-compatible server, and more), models per part of the app and API keys in the system keychain. Before that: splash and start screen, native menu bar, four languages, settings with Apply/Close, documents in tabs or windows, all 18 Crafting Apps.
Editing arrives in 0.3.0; see the plan for the road to 1.0.0.

## Run

```
cargo run -p glyphra
```

Linux needs `libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev` (and a C compiler). macOS and Windows need nothing extra.

## Glyphra AI

Open **Settings › AI providers**, pick a provider, paste its API key and press Apply. Keys are stored in the system
keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service such as GNOME Keyring or KWallet) and never
in a file; `ai.toml` in the settings folder holds the rest. Without a keychain (a bare Linux session) the key lasts
until Glyphra closes. Environment variables such as `ANTHROPIC_API_KEY` are used when no key is stored. Local servers
(Ollama, LM Studio) need no key.

## Build installers

```
./build.sh              # macOS: Glyphra.app + .dmg    Linux: .tar.gz, .deb (.rpm, AppImage if the tools exist)
./build.sh --run        # build and start
.\scripts\build-windows.ps1 [-Msi] [-Arch arm64]    # Windows (PowerShell): .zip, optionally .msi
```

Results and `.sha256` checksums land in `dist/`. Build each platform on that platform. macOS: `--universal`
for Apple silicon + Intel, `APPLE_SIGNING_IDENTITY` / `APPLE_NOTARY_PROFILE` to sign and notarize (without
them the app is ad-hoc signed). Windows: `GLYPHRA_SIGN_CERT` / `GLYPHRA_SIGN_PASSWORD` to sign; `-Msi` needs the
WiX CLI (`dotnet tool install --global wix`). Run-time needs: Linux WebKitGTK 4.1, GTK 3 and libxdo;
Windows the WebView2 runtime (already on Windows 11).

## Add a language

Copy `locales/en` to a new folder named after the language tag, translate `main.ftl`, and fill in
`locale.toml` (name, native name, `direction = "rtl"` for Hebrew or Arabic). Put the folder in
`<settings folder>/locales/` (Settings › About shows the path) and restart. Messages you leave out
fall back to English.

## Licence

MIT or Apache-2.0, at your option.
