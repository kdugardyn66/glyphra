# What came from where

| Source | Taken | Not yet taken (planned) |
|---|---|---|
| **mdCraft** (base functionality) | The 19 document and deck templates and placeholder logic (`gl-templates`); the "everything is a command" and "never lose work" rules; atomic writes | `mdcraft-core` renderer (diagrams, charts, math, media), CodeMirror editor and its commands, slides/present mode, CLI and MCP server |
| **Glyphra 0.2** (AI) | Dioxus 0.7 desktop startup and native-menu pattern; key storage (`secrets.rs`, now AES-GCM in the settings file); logo and app icons; the settings and Apply idea | `kd-ai` provider layer (streaming chat, tasks), Fluent i18n (4 languages), packaging scripts |
| **ONLYOFFICE Desktop Editors** (layout) | Layout only, from screenshots: icon rail, Create new cards, Open, Templates gallery, AI Tools catalogue, Settings pages, right-hand AI panel. No code or assets | Left and right editor toolbars; document server (collaboration) |

ONLYOFFICE is AGPL-licensed. Glyphra uses none of its code, text or artwork; if that ever changes,
the licence consequences need to be settled first.

## Roadmap

| Version | Content |
|---|---|
| **0.1.0** | Start screen, left menu bar, Open, Templates, AI Tools catalogue, Settings (General, AI Models, Model Assignment, MCP Servers), AI panel layout, read-only document view, macOS `.app` + `.dmg`, Linux and Windows packages (`./build.sh`) |
| 0.2.0 | Editor: mdCraft's renderer and CodeMirror editor in a web view pane, left and right toolbars, save |
| 0.3.0 | AI Chat and AI Tools on `kd-ai` (providers, streaming, model assignment in use); local SQLite cache |
| 0.4.0 | i18n (en, nl, fr, de) |
| 0.5.0 | Collaboration server (Rust, PostgreSQL, nginx, Docker): accounts, shared folders, live editing, comments, chat |
| later | ChromeOS; iOS, iPadOS and Android apps |
