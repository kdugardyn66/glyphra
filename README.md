# Glyphra

Markdown documents and presentations in one workspace, with AI and team
collaboration. Rust and Dioxus; macOS, Linux and Windows.

**Version 0.1.0** — the start screen and the left menu bar:

- **Home**: Create new (Document, Presentation) and your recent files
- **Open**: recent folders and Browse
- **Templates**: 19 Markdown templates in two tabs (Documents, Presentations), with search, Category / Purpose / Language filters and live thumbnails
- **Settings › AI Models**: 19 providers by region (Europe, Asia-Pacific, North America, local) with 55 models, ported from Glyphra; custom providers pick an API and region
- **AI Tools**: the catalogue of planned tools (they arrive in a later version)
- **Collaboration**: placeholder for the server part
- **AI Chat**: the right-hand panel, toggled from the menu bar (conversation arrives in 0.3.0)
- **Settings**: General (applied with *Apply*; Appearance (System, Light or Dark), a light theme (Modern Light, Solarized Light, Light+) and a dark theme (Modern Dark, Solarized Dark, Dark+); default System with Modern Light and Modern Dark), AI Models (providers, API keys encrypted (AES-256-GCM) inside the settings file), Model Assignment, MCP Servers (validated JSON editor)

Opening a document shows a read-only preview in a tab; the editor arrives in 0.2.0. See
[docs/consolidation.md](docs/consolidation.md) for the roadmap and for what came from mdCraft,
Glyphra 0.2 and ONLYOFFICE's layout.

## Run

```
cargo run -p glyphra
```

Linux needs `libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev`. Settings live in the platform's config
folder (`GLYPHRA_CONFIG_DIR` overrides it); new documents go to `~/Documents/Glyphra`.

## Build packages

```
./build.sh                 # on macOS: Glyphra.app and Glyphra-0.1.0-macos-<arch>.dmg in dist/
./build.sh --universal     # Apple silicon + Intel in one binary
./build.sh --no-dmg        # the .app only
./build.sh --run           # build and open the app
```

**Linux:** `./build.sh` makes a `.tar.gz`, plus `.deb`, `.rpm` and AppImage when `dpkg-deb`, `rpmbuild` and
`appimagetool` are installed (`--tar-only` for the archive alone). **Windows:** `.\scripts\build-windows.ps1`
makes a portable `.zip`; `-Msi` adds an installer (WiX CLI), `-Arch arm64` targets Windows on ARM.

**macOS:** needs Rust and the Xcode command line tools. The `.dmg` is made with `diskutil image` on macOS 26 and newer
and with `hdiutil` on older systems; Finder lays out the window (allow Terminal to control Finder when
asked, or `DMG_LAYOUT=0` for a plain image, as in CI). Without `APPLE_SIGNING_IDENTITY` the app is ad-hoc
signed and runs on the Mac that built it; set `APPLE_SIGNING_IDENTITY` and `APPLE_NOTARY_PROFILE` to sign
and notarize for distribution. `./build.sh --help` lists every option. The window background is
`packaging/macos/dmg-background.svg` (`tools/gen_dmg_background.sh` renders the PNGs).

## Layout

| Path | What |
|---|---|
| `crates/gl-core` | Model, commands, menu bar as data, Markdown preview, AI providers (no UI) |
| `crates/gl-store` | Settings, recent files, atomic writes, encrypted API keys |
| `crates/gl-templates` | The Markdown templates |
| `crates/gl-ui` | Dioxus components and `glyphra.css` |
| `apps/glyphra` | The binary and the native menu |
| `deploy/` | Docker Compose and nginx for the server side (skeleton) |

## Licence

MIT or Apache-2.0, at your option.


## Where the models come from

Once a provider is connected (or while you type its key in the form), Glyphra asks the provider's own models endpoint which models it offers and lists exactly those in Settings › AI Models and in the AI Chat model picker. The endpoints, how the key is sent and how the answer is read are data in `crates/gl-core/data/catalog.json` (`models_api` per provider; code in `crates/gl-core/src/models_api.rs`):

| Provider | Request |
|---|---|
| OpenAI, Mistral, Alibaba, DeepSeek, Moonshot, Sakana, Z.AI, Meta, xAI, LM Studio | `GET <base>/models`, `Authorization: Bearer <key>` |
| Anthropic | `GET /v1/models`, `x-api-key` + `anthropic-version` |
| Google Gemini | `GET /v1beta/models?key=<key>` |
| Cohere | `GET https://api.cohere.com/v1/models`, Bearer |
| Microsoft Foundry | `GET https://<resource>.openai.azure.com/openai/v1/models`, `api-key` |
| Ollama | `GET /api/tags` (no key) |
| Aleph Alpha | `GET <your endpoint>/models_available`, Bearer |
| Black Forest Labs | `GET https://api.bfl.ai/v1/models`, Bearer |
| DeepL | `GET /v2/languages?type=target`, `DeepL-Auth-Key` (lists languages, not models: a good answer only proves the key) |
| Your own OpenAI-compatible endpoint | `GET <base>/v1/models`, Bearer (providers you add yourself use the endpoint of the API they speak) |

It asks when a provider is chosen, when the model list is opened (Settings or AI Chat), and with the **Load models from the provider** link; the same address and key are not asked again within ten minutes. The answer is kept in `models.json` in the config folder, so the list is there when offline. Prices come from the built-in catalogue (the providers' endpoints do not publish them): models the provider offers that the catalogue does not know are listed without a price. If asking fails (wrong key, offline, no list at that address), the built-in suggestions stay and the reason is shown under the Model field. A model missing from the list can always be used: choose **Other — type a model ID…**.

Safety: the key is sent only to the provider's own address (or the endpoint you entered), over https (http only for `localhost`); redirects are never followed; answers are capped at 2 MB and 1000 models; the address with the key is never put in an error message.

## Languages

The interface is available in English (US/UK), Spanish (Latin America/Spain), Portuguese (Brazil/Portugal),
French (Canada/France), German, Italian and Dutch. Choose it in Settings › General › Interface language
(applies immediately; native menus follow after a restart). On first run Glyphra uses your system language
when it is supported, otherwise English (US).

Translations live in `crates/gl-core/i18n/<tag>.json` as `"English text": "translation"`.
Regional files (es-ES, pt-PT, fr-CA, en-GB) hold only the differences from their parent (es-419, pt-BR, fr-FR, English).
To add a language: add the JSON file, register it in `i18n.rs` (`LANGUAGES`, `CATALOGS`), and the tests check it
has every key with matching `{placeholders}`. Wrap new UI text in `t("…")`, `tr("… {x}", &[("x", v)])` or `tx("…")` for constants.

## Region

Settings › Region sets units (temperature, measurement), the first day of the week, short and long date and time
formats, decimal and thousands symbols and the currency (symbol, ISO code, text pattern for positive and negative
amounts). Pick one of 248 countries (or System) and its usual formats apply; tick "Change settings manually" to set
your own. Changes save at once. Date patterns use `yyyy MM dd dddd MMMM HH hh mm ss sss a`; text in `'quotes'` is kept.

The country data (currency, symbol, number, currency, date and time formats) comes from the World Currencies
workbook: `python3 tools/gen_region.py <workbook.xlsx> crates/gl-store/data/region.json` regenerates
`crates/gl-store/data/region.json`. Units and the first weekday are not in the workbook (`gl-store/src/region.rs`).
Known limits: one thousands symbol per region (Austria writes currency with dots, numbers with spaces; Glyphra
uses spaces for both), long dates have no weekday and month names come from the interface language.
Rendering lives in `crates/gl-core/src/region.rs` (`region::pattern`, `number`, `currency`, `temperature`, `length`);
set `GLYPHRA_REGION_EXPECTED` to run the test that compares all countries with the workbook's example strings.

A model of a connected provider can be changed or moved to another provider with "Edit" in the model's menu.
