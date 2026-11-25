//! The Help page: the manual. Topics at the left like the sections of Settings, and for the
//! topic you choose a complete description with screenshots. The text is English (it is the
//! manual); the screenshots are compiled into the program, so Help works offline.

use crate::shell::run;
use dioxus::prelude::*;
use gl_core::{AppModel, Command, HelpTopic, t};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// One piece of a topic.
enum Block {
    /// A heading inside the topic.
    H(&'static str),
    P(&'static str),
    /// A list of points.
    List(&'static [&'static str]),
    /// A numbered list of steps.
    Steps(&'static [&'static str]),
    /// A screenshot (file name without extension) with its caption.
    Shot(&'static str, &'static str),
    /// A tip or a warning, set apart.
    Note(&'static str),
    /// Two columns: what, and what it does.
    Table(&'static [(&'static str, &'static str)]),
}
use Block::*;

macro_rules! shots {
    ($($name:literal),* $(,)?) => {
        fn shot_bytes(name: &str) -> Option<&'static [u8]> {
            match name {
                $($name => Some(include_bytes!(concat!("../assets/help/", $name, ".png")).as_slice()),)*
                _ => None,
            }
        }
        /// Every screenshot the manual can show (a test checks that the text only uses these).
        #[cfg(test)]
        const SHOT_NAMES: &[&str] = &[$($name),*];
    };
}
shots!(
    "home",
    "home-selected",
    "open",
    "templates",
    "collaboration",
    "ai-tools",
    "settings-general",
    "settings-region",
    "settings-ai-models",
    "settings-add-model",
    "settings-assignment",
    "settings-mcp",
    "chat-empty",
    "chat-answer",
    "chat-context",
    "chat-tools",
    "chat-history",
    "chat-wide",
    "doc-open",
    "help",
);

/// A screenshot as a `data:` address, made once.
fn shot_src(name: &str) -> String {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let Ok(mut map) = cache.lock() else { return String::new() };
    map.entry(name.to_string())
        .or_insert_with(|| shot_bytes(name).map(|b| format!("data:image/png;base64,{}", base64(b))).unwrap_or_default())
        .clone()
}

fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(ABC[(n >> 18) as usize & 63] as char);
        out.push(ABC[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ABC[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ABC[n as usize & 63] as char } else { '=' });
    }
    out
}

/// A piece of the manual. Most are a topic of their own; the topic *Glyphra* is made of four.
#[derive(Clone, Copy)]
enum Part {
    Start,
    Shortcuts,
    Privacy,
    Troubleshooting,
    Home,
    Open,
    Templates,
    AiChat,
    AiTools,
    Collaboration,
    Help,
    SettingsIntro,
    General,
    Region,
    AiModels,
    ModelAssignment,
    McpServers,
}

/// The text of a topic.
fn topic(t: HelpTopic) -> Vec<Block> {
    let one = |p| part(p);
    match t {
        HelpTopic::Glyphra => {
            let mut v = part(Part::Start);
            v.push(H("Menus and shortcuts"));
            v.extend(part(Part::Shortcuts));
            v.push(H("Your files and privacy"));
            v.extend(part(Part::Privacy));
            v.push(H("Troubleshooting"));
            v.extend(part(Part::Troubleshooting));
            v
        }
        HelpTopic::AiChat => one(Part::AiChat),
        HelpTopic::Home => one(Part::Home),
        HelpTopic::Open => one(Part::Open),
        HelpTopic::Templates => one(Part::Templates),
        HelpTopic::Collaboration => one(Part::Collaboration),
        HelpTopic::AiTools => one(Part::AiTools),
        HelpTopic::Help => one(Part::Help),
        HelpTopic::Settings => one(Part::SettingsIntro),
        HelpTopic::General => one(Part::General),
        HelpTopic::Region => one(Part::Region),
        HelpTopic::AiModels => one(Part::AiModels),
        HelpTopic::ModelAssignment => one(Part::ModelAssignment),
        HelpTopic::McpServers => one(Part::McpServers),
    }
}

fn part(p: Part) -> Vec<Block> {
    match p {
        Part::Start => vec![
            P(
                "Glyphra is an editor for Markdown documents and presentations, with an AI assistant next to your text. It runs on macOS, Windows and Linux. This manual describes version 0.1.3.",
            ),
            Shot("home", "The Home page: create a document, or go back to something you opened before."),
            H("What is where"),
            Table(&[
                (
                    "Left menu bar",
                    "The pages of the program: AI Chat (shows or hides the panel on the right), Home, Open, Templates, Collaboration, AI Tools, and Help and Settings at the bottom.",
                ),
                ("Tabs at the top", "GLYPHRA is the Home tab. Every document you open gets a tab of its own, with a × to close it."),
                ("Middle", "The page you chose, or the document you are reading."),
                (
                    "AI Chat panel",
                    "The right-hand panel where you talk to the AI model you connected. It stays visible while you move between pages.",
                ),
                (
                    "Menu bar",
                    "File, Edit, View, Window and Help. On macOS it is at the top of the screen; on Windows and Linux at the top of the window.",
                ),
            ]),
            H("The first five minutes"),
            Steps(&[
                "Press **Document** on the Home page. A new Markdown file is created in your documents folder and opens in a tab. Or choose **Templates** in the left menu bar and pick one that fits (an invoice, a report, a letter).",
                "Open **Settings › AI Models** and connect an AI model (see *AI Models*). Without a model, everything except AI Chat and AI Tools already works.",
                "Open the AI Chat panel and press a prompt such as **Summarize document**. The answer appears in the panel.",
                "Come back to your files later from the list on Home: it remembers what you opened.",
            ]),
            Note(
                "In this version a document opens as a read-only preview. The editor (writing and saving changes) arrives in version 0.2.0.",
            ),
            H("Finding your way"),
            List(&[
                "Every button has a tip: rest the mouse on it for a moment.",
                "Press **F1**, or choose **Help › Glyphra Help**, to come back here. The Help button in the left menu bar does the same.",
                "The topics on the left of this page describe each page of the program, from top to bottom.",
            ]),
        ],
        Part::Home => vec![
            P("Home is where Glyphra starts. At the top you create something new; below it is the list of everything you opened before."),
            Shot("home", "Home: Create new, the buttons Cleanup and Clear all, and the list of files."),
            H("Create new"),
            P(
                "**Document** makes a new, empty Markdown document and **Presentation** a new slide deck. The file is saved at once in the *Glyphra* folder inside your Documents folder, with a name such as *Untitled document*, and opens in its own tab. If a file with that name already exists, a number is added; nothing is overwritten.",
            ),
            H("The list of files"),
            P(
                "Each line shows the name, the folder and when you last opened the file. The newest is on top; pinned files stay on top. Double-click a line to open it.",
            ),
            P(
                "Files that no longer exist on your disk (deleted, renamed or on a drive that is not connected) are shown in a lighter colour. Opening one tells you the file is missing.",
            ),
            Shot("home-selected", "A selected line shows four buttons at its right."),
            P("Click a line once to select it. Four buttons appear:"),
            Table(&[
                (
                    "Open in Chat",
                    "Attaches the file to AI Chat and shows the panel, so you can ask questions about it without opening it first.",
                ),
                ("Show in Folder", "Shows the file in Finder, Explorer or your file manager."),
                (
                    "Pin / Unpin",
                    "A pinned file keeps its place at the top of the list and is never pushed out by newer files. A small pin marks it.",
                ),
                ("Remove from list", "Takes the file off the list. The file itself stays on your disk."),
            ]),
            H("Cleanup and Clear all"),
            List(&[
                "**Cleanup** removes every line whose file no longer exists (the lighter lines). It is greyed out when there is nothing to clean up.",
                "**Clear all** empties the whole list. Your files stay where they are.",
            ]),
            Note("The list remembers the last 50 files and 20 folders. Pinned files are never pushed out by newer ones."),
        ],
        Part::Open => vec![
            P("Use Open to find a file on your computer."),
            Shot("open", "Open: your recent folders and the Browse button."),
            Steps(&[
                "Press **Browse**. A file dialog opens in your Documents folder.",
                "Choose a Markdown (.md, .markdown) or text (.txt) file. The file opens in a tab and is added to the list on Home.",
            ]),
            P(
                "Under **Recent folders** you see the folders you opened files from. Click one to open the file dialog right there. The same command is in the menu: **File › Open…** (Ctrl+O, ⌘O on macOS).",
            ),
            P("You can choose several files at once; each gets a tab. A file that is already open is not opened twice: its tab is shown."),
            Note(
                "Documents that are not Markdown or text (Word, PDF, images) cannot be opened yet. You can still attach any file with a readable text to AI Chat.",
            ),
        ],
        Part::Templates => vec![
            P(
                "Templates are ready-made documents and presentations. Choosing one creates a new file from it, in the Glyphra folder inside your Documents folder; the template itself never changes.",
            ),
            Shot("templates", "Templates: search, filters, the Documents and Presentations tabs and a page bar."),
            H("Choosing"),
            Steps(&[
                "Choose the **Documents** or **Presentations** tab.",
                "Type in the search box (for example *invoice*) or narrow the list with **Category**, **Purpose** and **Language**.",
                "Click a template. A new document is created, filled in with your name (from Settings › General) and today's date where the template has such fields, and opened in a tab.",
            ]),
            H("Pages"),
            P(
                "The grid shows as many complete rows as fit in the window, so no template is cut off. Use the page bar below the grid (the numbers and the arrows) to see the rest. Make the window taller or wider and more templates show per page.",
            ),
            P("If nothing matches, the page says so; clear the search or a filter."),
        ],
        Part::AiChat => vec![
            P(
                "AI Chat is a conversation with the AI model you connected, next to your document. It can read the document you are looking at and files you attach, and answer about them or rewrite them.",
            ),
            Shot("chat-empty", "AI Chat before the first message: the prompt buttons, the message box and the model picker."),
            H("Before you start"),
            List(&[
                "Connect a model in **Settings › AI Models** (see *AI Models*). Without one, pressing Send takes you to Settings.",
                "AI features must be switched on in Settings › General (they are, by default).",
                "Show or hide the panel with the **AI Chat** button at the top of the left menu bar, or with **View › AI Chat** (Ctrl+Alt+A, ⌘⌥A on macOS).",
            ]),
            H("Asking something"),
            P(
                "Type in the message box and press **Enter** to send. **Shift+Enter** starts a new line. While the model works the panel shows *Thinking…*; the answer then appears as a whole. Long answers can take a while: after three minutes without an answer Glyphra stops waiting and tells you.",
            ),
            Shot("chat-answer", "A question and its answer. The small button under the answer copies it."),
            P(
                "Your next message continues the same conversation, so you can say *shorter* or *in French*. The last 30 messages of the conversation are sent along. Press **+** at the top of the panel for a fresh conversation.",
            ),
            H("The prompt buttons"),
            P(
                "On an empty conversation the panel shows nine ready-made prompts. Pressing one sends it at once; the text of the prompt is hidden and your chat shows just its name.",
            ),
            Table(&[
                ("Summarize document", "A short summary of the document."),
                ("Improve writing", "Rewrites the text to read better and keeps its meaning and its language."),
                ("Extract key info", "Names, dates, amounts, decisions and open questions."),
                ("Explain simply", "Explains the document in plain language."),
                ("Generate outline", "An outline with headings, from the document."),
                ("Proofread & fix", "Corrects spelling and grammar and lists what changed."),
                ("Brainstorm ideas", "Ideas on a subject you describe. It needs no document."),
                ("Adapt tone", "Rewrites the text in a tone you name (formal, friendly …)."),
                ("Paraphrase", "Says the same in other words."),
            ]),
            P(
                "Buttons that work on a document need an open document or an attached file; if there is none, a message tells you so and nothing is sent.",
            ),
            H("Giving the model something to read"),
            Shot("chat-context", "The + button: include the current document, or attach a file."),
            P("Above the message box small labels show what is sent with your message:"),
            List(&[
                "**The current document** (the tab you are looking at) is included automatically. Press its × to leave it out, or tick **Include the current document** in the **+** menu to bring it back.",
                "**Attached files**: choose **Attach a file…** in the **+** menu, or press **Open in Chat** on a line of the Home list. Press the × on a file to remove it. Files must contain readable text; the ones that do not (a picture, say) are reported and not sent.",
            ]),
            Note(
                "Everything shown above the message box leaves your computer when you press Send: it goes to the provider of the model you chose, and to nobody else. See *Your files and privacy* in the topic *Glyphra*.",
            ),
            H("Tools"),
            Shot("chat-tools", "The Tools button lists the AI Tools; choosing one runs it on the current document."),
            P(
                "The terminal-style button next to **+** opens the list of AI Tools (translate, review, proofread, make slides …). Choosing one sends its prompt for the current document. See the topic *AI Tools*.",
            ),
            H("History"),
            Shot("chat-history", "History: earlier conversations."),
            P(
                "The clock button lists your earlier conversations, newest first, with the time. Click one to continue it, press the bin to delete it, or choose **Delete all chats**. Glyphra keeps up to 50 conversations of up to 200 messages each, on your computer.",
            ),
            H("Model, expand, settings"),
            Shot("chat-wide", "The expanded panel, for long answers."),
            List(&[
                "The picker above the buttons chooses the model for AI Chat. The arrows beside it step to a cheaper or a more capable model of the same provider; the provider and the key stay the same. **Add another provider…** at the bottom of the list opens Settings.",
                "**Expand** (the arrows at the top) makes the panel wider; press it again to make it narrower.",
                "The sliders button opens **Settings › AI Models**.",
                "The **×** hides the panel.",
            ]),
            H("When something goes wrong"),
            P(
                "A problem is shown in the conversation in plain words, for example that the key is wrong, that the model is unknown or that you hit a limit. Press **Try again** to send the same message again. See *Troubleshooting* in the topic *Glyphra*.",
            ),
        ],
        Part::AiTools => vec![
            P(
                "AI Tools is a catalogue of jobs that AI can do on a document or a presentation. Each tool is a ready-made prompt that runs in AI Chat on the document you are looking at.",
            ),
            Shot("ai-tools", "AI Tools: Document, Presentation and Other Tools."),
            H("Using a tool"),
            Steps(&[
                "Open the document (or presentation) you want the tool to work on.",
                "Open **AI Tools** in the left menu bar and click the tool, or press the Tools button in AI Chat and choose it there.",
                "The AI Chat panel opens, the tool's request is sent with your document, and the answer arrives in the conversation.",
            ]),
            P(
                "If no document is open, a message asks you to open one first. **Compare Documents** needs two: the document you are looking at and a second file you attach, or two attached files.",
            ),
            H("The tools"),
            Table(&[
                ("Translate Document", "Translates and keeps the formatting. Say which language you want."),
                ("Compare Documents", "Finds the differences between two documents."),
                ("Review Document", "Finds factual errors, inconsistencies and typos."),
                ("Proofread Document", "Fixes typos and grammar and gives suggestions."),
                ("Fix Formatting", "Makes headings, lists and spacing consistent."),
                ("Summarize Document", "The key points, figures and dates."),
                ("Slides from Document", "Drafts a slide deck from a document."),
                ("Improve Slides", "Tightens titles and bullet points."),
                ("Speaker Notes", "Writes notes for every slide."),
                ("Translate Presentation", "Translates a deck and keeps its layout."),
                ("Check Before Publishing", "Finds local paths, broken links and the like before you share a file."),
                ("Remove Personal Data", "Clears names, emails and phone numbers."),
            ]),
            Note(
                "**Convert to Book** (e-books) arrives in a later version; its button says so. The answers come back in the chat: in this version they are not written into your document, copy what you want to keep.",
            ),
        ],
        Part::Collaboration => vec![
            P("Collaboration will let a team work on documents together, with shared folders, comments and chat, on a server of your own."),
            Shot("collaboration", "The Collaboration page tells you it is coming in a later version."),
            P("Nothing on this page works yet. It is in the menu so you can see where it will be."),
        ],
        Part::SettingsIntro => vec![
            P(
                "Settings has five parts, listed under *Settings* at the left of this page: General, Region, AI Models, Model Assignment and MCP Servers. Open Settings with the button at the bottom of the left menu bar, or with Ctrl+, (⌘, on macOS).",
            ),
            Shot("settings-general", "Settings: the five parts are listed at the left."),
            H("Saving"),
            P(
                "In General and Region, changes are kept when you press **Apply** at the bottom of the page. **Reset to default** puts that part back to how Glyphra came. AI Models, Model Assignment and MCP Servers save as you press their own buttons.",
            ),
            Note("An asterisk (\\*) after an option means it is applied after you restart the program."),
        ],
        Part::General => vec![
            P("General holds the options for you and for the look of the program."),
            Shot("settings-general", "Settings › General."),
            Table(&[
                ("User Name", "Your name. Templates use it for fields such as *Author*."),
                (
                    "Interface language \\*",
                    "The language of the menus and pages. Eleven languages are available. The manual stays in English.",
                ),
                ("Appearance", "System follows your operating system, or choose Light or Dark."),
                (
                    "Light theme / Dark theme",
                    "Modern, Solarized or Light+/Dark+. The Light theme is used in a light appearance, the Dark theme in a dark one.",
                ),
                (
                    "Open file",
                    "In its own tab, or in a new window. Separate windows arrive with the editor; until then documents open in tabs either way.",
                ),
                ("Spelling language detection", "Auto lets Glyphra find the language of a document; Off does not."),
                ("Use graphics acceleration \\*", "Makes drawing faster on most computers. Switch it off if you see display problems."),
                ("Enable AI features \\*", "Turns the AI Chat panel and AI Tools on or off."),
            ]),
            P("Press **Apply** to keep your changes. Options with an asterisk are applied after a restart."),
        ],
        Part::Region => vec![
            P("Region decides how dates, times, numbers, units and currency are written."),
            Shot("settings-region", "Settings › Region."),
            P(
                "Choose a region and everything follows it; the examples under each field show the result. To adjust a single format yourself, tick **Change settings manually**: you can then change the unit of temperature and measurement, the first day of the week, the short and long date and time formats, the number format (decimal symbol, thousands separator) and the currency.",
            ),
            P("Templates and documents created from now on use these formats. **Reset to default** goes back to the region's own formats."),
        ],
        Part::AiModels => vec![
            P(
                "Glyphra does not come with an AI model. You connect your own: an account at a provider (such as OpenAI, Anthropic or Google) with an API key, or a model that runs on your own computer (such as Ollama). You pay the provider directly, for what you use.",
            ),
            H("Connect a model"),
            Shot("settings-ai-models", "Settings › AI Models: the models you connected."),
            Steps(&[
                "Press **Add Model**.",
                "Choose the **Provider**. The list is grouped by region and has the well-known providers and local servers.",
                "Paste your **API key** (the eye shows or hides it). Local models such as Ollama need no key. The provider's page where you get a key is mentioned under the field.",
                "Check the **Base URL** (Glyphra fills it in; change it only for a special address) and choose the **Model**. The list is what the provider itself reports for your key. If your model is missing, choose **Other — type a model ID…**.",
                "Give the connection a **Profile name** if you want another label and press **Add Model**.",
            ]),
            Shot("settings-add-model", "The form for a new model."),
            P(
                "Glyphra asks the provider which models it offers as soon as you choose it, when you open the model list, and with the **Load models from the provider** link. The answer is kept, so the list is there when you are offline. If the provider cannot be asked (a wrong key, no internet), the reason is shown under the Model field and the built-in suggestions stay.",
            ),
            H("Manage the list"),
            List(&[
                "The **⋯** button on a model offers **Edit**, **Make default** and **Delete**. The default model is used for everything that has no model of its own.",
                "In AI Chat you switch between your models with the picker, without opening Settings.",
                "**Custom Providers**: add any server that speaks the OpenAI chat API (a company gateway, vLLM, LiteLLM). Give it a name, the address and the API it speaks. **Import Custom Provider** reads such a definition from a file.",
            ]),
            H("Your key is protected"),
            P(
                "API keys are never shown again after you save them, are stored encrypted on your computer and are sent only to the provider they belong to. See *Your files and privacy* in the topic *Glyphra*.",
            ),
        ],
        Part::ModelAssignment => vec![
            P("Model Assignment lets you use a different model for a different job."),
            Shot("settings-assignment", "Settings › Model Assignment."),
            P(
                "The **Default model** is used everywhere. Under **Custom models per task** you can choose another model for one job, for example a cheaper model for summaries; **Default model** in a line means that job uses the default. In this version AI Chat uses the model you pick in the panel (or the AI Chat line here); the other tasks (Code, Summarization, Translation, Text Analysis, Image generation, OCR, Vision) are saved for the features that arrive later.",
            ),
        ],
        Part::McpServers => vec![
            P("MCP servers give an AI model extra tools (searching files, reading a database)."),
            Shot("settings-mcp", "Settings › MCP Servers."),
            Steps(&[
                "Press **Edit configuration**.",
                "Enter the configuration as JSON, in the form the server's documentation gives.",
                "Press **Save**. Glyphra checks that it is valid and lists the servers it found.",
            ]),
            Note("In this version the configuration is only saved: the tools are not offered to the model yet."),
        ],
        Part::Help => vec![
            P(
                "Help is this manual. It is always one step away: the **Help** button above Settings in the left menu bar, **Help › Glyphra Help** in the menu bar, or **F1**.",
            ),
            Shot("help", "Help: the topics at the left, the manual at the right."),
            List(&[
                "The list at the left has **Glyphra** (an overview, the menus and shortcuts, privacy and troubleshooting), then the pages of the program, then **Settings** with its five parts.",
                "Click a topic to read it. Each topic describes the page of that name, with screenshots of it.",
                "The manual is built into the program, so it works without internet.",
                "The manual is written in English; the names of the topics, menus and buttons follow your interface language.",
            ]),
        ],
        Part::Shortcuts => vec![
            P("Everything in the menu bar can also be done from the pages. On macOS use ⌘ where this table says Ctrl, and ⌥ for Alt."),
            H("Shortcuts"),
            Table(&[
                ("Ctrl+N", "New document"),
                ("Ctrl+O", "Open a file"),
                ("Ctrl+W", "Close the tab"),
                ("Ctrl+1 … Ctrl+5", "Home, Open, Templates, Collaboration, AI Tools"),
                ("Ctrl+Alt+A", "Show or hide AI Chat"),
                ("Ctrl+,", "Settings"),
                ("F1", "This manual"),
                ("Ctrl+Q", "Quit"),
                ("Enter / Shift+Enter", "In AI Chat: send / new line"),
            ]),
            H("The menus"),
            Table(&[
                ("File", "New Document, New Presentation, New from Template…, Open…, Close Tab, and (not on macOS) Settings… and Quit."),
                ("Edit", "Undo, Redo, Cut, Copy, Paste, Select All."),
                ("View", "Home, Open, Templates, Collaboration, AI Tools, AI Chat, full screen."),
                ("Window", "Minimize, Zoom."),
                ("Help", "Glyphra Help, Menus and shortcuts."),
                ("Glyphra (macOS only)", "About, Settings…, Hide, Quit."),
            ]),
        ],
        Part::Privacy => vec![
            P("Glyphra keeps your files and settings on your computer and talks to the internet only for the things below."),
            H("What leaves your computer"),
            List(&[
                "**When you send a message in AI Chat** (or run a prompt button or tool): your message, the earlier messages of that conversation, the document you are looking at and the files you attached go to the provider of the model you chose. Nobody else sees them. Use the × on the document label to leave the document out.",
                "**When Glyphra asks which models exist**: the provider gets a request with your API key (if it needs one) and nothing about your documents.",
                "Nothing is sent when you only read, open or create documents, and nothing is sent in the background.",
            ]),
            P(
                "What the provider does with the text is governed by your agreement with them. If a document must not leave your computer, use a local model (such as Ollama): then nothing leaves at all.",
            ),
            H("What is stored, and where"),
            Table(&[
                (
                    "Your documents",
                    "In the Glyphra folder inside your Documents folder, as ordinary Markdown files. Glyphra never changes a file you did not ask it to change, and writes in a safe way so that an interrupted save cannot destroy a file.",
                ),
                ("Settings", "settings.json in the settings folder. API keys in it are encrypted, with a key file kept beside it."),
                ("Recent files", "library.json in the settings folder."),
                ("Chat history", "chats.json in the settings folder. Delete a conversation (or all) in the History menu."),
                ("Model lists", "models.json in the settings folder."),
            ]),
            P(
                "The settings folder is *~/Library/Application Support/Glyphra* on macOS, *%APPDATA%\\\\glyphra\\\\Glyphra\\\\config* on Windows and *~/.config/glyphra* on Linux. To start over, close Glyphra and delete the folder.",
            ),
        ],
        Part::Troubleshooting => vec![
            P("The messages Glyphra shows are written to say what to do. Here are the common ones in more detail."),
            Table(&[
                ("Connect an AI model first", "No model is connected. Open Settings › AI Models and press Add Model."),
                ("There is no API key for …", "The model is listed but its key is missing. Edit the model and paste the key."),
                (
                    "Open a document or attach a file first",
                    "A prompt button or tool needs something to work on. Open a document, or attach a file with **+**.",
                ),
                (
                    "The provider answered with an error (401 / 403)",
                    "The key is wrong, expired or not allowed to use that model. Check it at the provider.",
                ),
                (
                    "The provider does not know this model or address (404)",
                    "The model name or the Base URL is wrong. Choose the model from the list, or check the address in Settings › AI Models.",
                ),
                (
                    "Too many requests or the quota is used up (429)",
                    "You hit the provider's limit or your credit is finished. Wait a moment, or add credit.",
                ),
                (
                    "Couldn't reach the provider",
                    "No internet, a firewall, or a local server (Ollama) that is not running. Start it, then press **Try again**.",
                ),
                (
                    "This model cannot chat",
                    "Some providers (such as DeepL, for translation, or image makers) offer models that do not chat. Choose a chat model.",
                ),
                (
                    "The answer was not understood",
                    "The address answered, but not like a chat model. Check that the Base URL ends where the provider says (for example …/v1).",
                ),
                ("Files could not be read as text", "The attached file is a picture or a binary format. Only text files can be sent."),
            ]),
            H("Other problems"),
            List(&[
                "**A window that stays empty or flickers**: in Settings › General switch off graphics acceleration and restart.",
                "**A file on Home is light grey**: it no longer exists at that place. Press Cleanup to take such lines off the list.",
                "**Settings do not change a language or AI switch**: the options with an asterisk are applied after a restart.",
                "**A damaged settings file**: Glyphra keeps it as settings.json.bad and starts with defaults, so nothing is lost; you can open the .bad file in a text editor.",
            ]),
            P(
                "If something here does not help, note the exact message and what you did before it, and report it on the project page at github.com/kdugardyn66/glyphra.",
            ),
        ],
    }
}

/// `**bold**` and `*italic*` inside a line of text; a backslash makes the next character plain.
fn inline(text: &'static str) -> Element {
    let parts = split_inline(text);
    rsx! {
        for (i , (b , it , s)) in parts.into_iter().enumerate() {
            if b {
                strong { key: "{i}", "{s}" }
            } else if it {
                em { key: "{i}", "{s}" }
            } else {
                span { key: "{i}", "{s}" }
            }
        }
    }
}

fn split_inline(text: &str) -> Vec<(bool, bool, String)> {
    let mut parts: Vec<(bool, bool, String)> = Vec::new();
    let (mut bold, mut italic) = (false, false);
    let mut buf = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            // a backslash makes the next character plain text
            if let Some(n) = chars.next() {
                buf.push(n);
            }
        } else if c == '*' {
            if !buf.is_empty() {
                parts.push((bold, italic, std::mem::take(&mut buf)));
            }
            if chars.peek() == Some(&'*') {
                chars.next();
                bold = !bold;
            } else {
                italic = !italic;
            }
        } else {
            buf.push(c);
        }
    }
    if !buf.is_empty() {
        parts.push((bold, italic, buf));
    }
    parts
}

#[component]
pub fn HelpPage() -> Element {
    let model = use_context::<Signal<AppModel>>();
    let current = model.read().help;
    let blocks = topic(current);
    rsx! {
        div { class: "settings help",
            div { class: "set-nav",
                h2 { {t("Help")} }
                for tp in HelpTopic::ALL {
                    button {
                        key: "{tp.label()}",
                        class: format!("set-item d{}{}", tp.depth(), if tp == current { " active" } else { "" }),
                        onclick: move |_| run(model, Command::OpenHelp(tp)),
                        {t(tp.label())}
                    }
                }
            }
            div { class: "set-pane",
                div { class: "pane-scroll",
                    // keyed by topic, so the page starts at the top for every topic
                    article { key: "{current.label()}", class: "manual selectable",
                        h1 { {t(current.label())} }
                        p { class: "note", {t("The manual is written in English.")} }
                        for (i , b) in blocks.into_iter().enumerate() {
                            match b {
                                H(text) => rsx! { h3 { key: "{i}", "{text}" } },
                                P(text) => rsx! { p { key: "{i}", {inline(text)} } },
                                Note(text) => rsx! { p { key: "{i}", class: "callout", {inline(text)} } },
                                List(items) => rsx! {
                                    ul { key: "{i}",
                                        for (j , it) in items.iter().enumerate() {
                                            li { key: "{j}", {inline(it)} }
                                        }
                                    }
                                },
                                Steps(items) => rsx! {
                                    ol { key: "{i}",
                                        for (j , it) in items.iter().enumerate() {
                                            li { key: "{j}", {inline(it)} }
                                        }
                                    }
                                },
                                Table(rows) => rsx! {
                                    table { key: "{i}", class: "kv",
                                        for (j , (k , v)) in rows.iter().enumerate() {
                                            tr { key: "{j}",
                                                th { {inline(k)} }
                                                td { {inline(v)} }
                                            }
                                        }
                                    }
                                },
                                Shot(name, caption) => rsx! {
                                    figure { key: "{i}",
                                        img { src: shot_src(name), alt: "{caption}" }
                                        figcaption { "{caption}" }
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_topic_has_text_and_every_screenshot_it_names_exists() {
        for tp in HelpTopic::ALL {
            let blocks = topic(tp);
            assert!(blocks.len() >= 3, "{tp:?} is too short");
            for b in blocks {
                if let Shot(name, caption) = b {
                    assert!(SHOT_NAMES.contains(&name), "{tp:?}: unknown screenshot {name}");
                    assert!(!caption.is_empty());
                }
            }
        }
    }

    #[test]
    fn inline_marks_and_escapes() {
        assert_eq!(
            split_inline("a **b** *c* \\* d"),
            vec![
                (false, false, "a ".to_string()),
                (true, false, "b".to_string()),
                (false, false, " ".to_string()),
                (false, true, "c".to_string()),
                (false, false, " * d".to_string()),
            ]
        );
        assert_eq!(split_inline("C:\\\\x"), vec![(false, false, "C:\\x".to_string())]);
    }

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"Glyphra!"), "R2x5cGhyYSE=");
    }
}
