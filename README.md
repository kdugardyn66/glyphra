# Glyphra

Markdown, PDF and Drawing in one workspace, with Glyphra AI and team collaboration on your own server.

**Version 0.1.3** — splash screen, start screen, native menu bar, four languages; settings with Apply/Close, a changeable documents folder, documents in tabs or windows, the icons of installed apps, a File, Window and Help menu as on a Mac, all 18 Crafting Apps with versions and a refresh button.
Editing arrives in 0.3.0; see the plan for the road to 1.0.0.

## Run

```
cargo run -p glyphra
```

Linux needs `libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev`. macOS and Windows need nothing extra.

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
