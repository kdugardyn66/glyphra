#!/usr/bin/env python3
"""Writes locales/<tag>/main.ftl for the four built-in languages from one table, so every
language has exactly the same message ids. Run from the repository root: python3 tools/gen_locales.py
Adding a language later needs no code: copy locales/en to a new folder, translate, set locale.toml."""
import os, textwrap

LANGS = ["en", "nl", "fr", "de"]
META = {
    "en": ("English", "English"),
    "nl": ("Dutch", "Nederlands"),
    "fr": ("French", "Français"),
    "de": ("German", "Deutsch"),
}

# id -> (en, nl, fr, de). A value starting with "SEL:" is a Fluent plural block written out by hand.
T = {}
def add(id, en, nl, fr, de): T[id] = (en, nl, fr, de)

# --- splash, tabs, sidebar
add("app-name", "Glyphra", "Glyphra", "Glyphra", "Glyphra")
add("splash-tagline", "Documents, PDFs and drawings in one workspace", "Documenten, pdf's en tekeningen in één werkruimte", "Documents, PDF et dessins dans un seul espace de travail", "Dokumente, PDFs und Zeichnungen in einem Arbeitsbereich")
add("tab-home", "Home", "Start", "Accueil", "Start")
add("tab-theme", "Switch theme", "Thema wisselen", "Changer de thème", "Design wechseln")
add("nav-home", "Home", "Start", "Accueil", "Start")
add("nav-new", "New", "Nieuw", "Nouveau", "Neu")
add("nav-open", "Open", "Openen", "Ouvrir", "Öffnen")
add("nav-recent", "Recent", "Recent", "Récents", "Zuletzt")
add("nav-starred", "Starred", "Met ster", "Favoris", "Markiert")
add("nav-apps", "Apps", "Apps", "Apps", "Apps")
add("nav-folders", "Folders", "Mappen", "Dossiers", "Ordner")
add("nav-add-folder", "Add a folder", "Map toevoegen", "Ajouter un dossier", "Ordner hinzufügen")
add("nav-settings", "Settings", "Instellingen", "Réglages", "Einstellungen")

# --- start page
add("greeting-morning", "Good morning", "Goedemorgen", "Bonjour", "Guten Morgen")
add("greeting-afternoon", "Good afternoon", "Goedemiddag", "Bon après-midi", "Guten Tag")
add("greeting-evening", "Good evening", "Goedenavond", "Bonsoir", "Guten Abend")
add("home-greeting-named", "{ $part }, { $name }", "{ $part }, { $name }", "{ $part }, { $name }", "{ $part }, { $name }")
add("home-subtitle", "Start something new or pick up where you left off.", "Begin iets nieuws of ga verder waar je gebleven was.", "Commencez quelque chose de nouveau ou reprenez là où vous vous étiez arrêté.", "Beginnen Sie etwas Neues oder machen Sie dort weiter, wo Sie aufgehört haben.")
add("home-start-new", "Start something new", "Begin iets nieuws", "Commencer quelque chose de nouveau", "Etwas Neues beginnen")
add("home-start-new-into", "New documents go into “{ $folder }”", "Nieuwe documenten komen in “{ $folder }”", "Les nouveaux documents vont dans « { $folder } »", "Neue Dokumente landen in „{ $folder }“")
add("home-open-file", "Open file", "Bestand openen", "Ouvrir un fichier", "Datei öffnen")
add("home-open-file-tagline", "From this computer", "Van deze computer", "Depuis cet ordinateur", "Von diesem Computer")
add("tile-new-document", "New document", "Nieuw document", "Nouveau document", "Neues Dokument")
add("module-markdown", "Markdown", "Markdown", "Markdown", "Markdown")
add("module-pdf", "PDF", "PDF", "PDF", "PDF")
add("module-drawing", "Drawing", "Tekening", "Dessin", "Zeichnung")
add("module-markdown-tagline", "Documents and slides in plain text", "Documenten en dia's in platte tekst", "Documents et diapositives en texte brut", "Dokumente und Folien in reinem Text")
add("module-pdf-tagline", "View, annotate, organise and sign", "Bekijken, annoteren, ordenen en ondertekenen", "Afficher, annoter, organiser et signer", "Ansehen, kommentieren, ordnen und signieren")
add("module-drawing-tagline", "Diagrams and figures as SVG", "Diagrammen en figuren als SVG", "Diagrammes et figures en SVG", "Diagramme und Abbildungen als SVG")
add("apps-title", "Apps", "Apps", "Apps", "Apps")
add("apps-see-all", "See all", "Alles tonen", "Tout afficher", "Alle anzeigen")
add("apps-subtitle-none", "None installed yet · get them with the ArtCraft launcher", "Nog geen geïnstalleerd · haal ze op met de ArtCraft-launcher", "Aucune installée · obtenez-les avec le lanceur ArtCraft", "Noch keine installiert · über den ArtCraft-Launcher erhältlich")
add("apps-subtitle", "{ $n } of { $total } installed on this computer", "{ $n } van { $total } geïnstalleerd op deze computer", "{ $n } sur { $total } installées sur cet ordinateur", "{ $n } von { $total } auf diesem Computer installiert")
add("apps-empty", "Installed apps appear here; click one to open it.", "Geïnstalleerde apps verschijnen hier; klik om er een te openen.", "Les apps installées apparaissent ici ; cliquez pour en ouvrir une.", "Installierte Apps erscheinen hier; klicken Sie eine an, um sie zu öffnen.")
add("apps-not-installed", "Not installed", "Niet geïnstalleerd", "Non installée", "Nicht installiert")
add("apps-get-launcher", "Get it with the ArtCraft launcher", "Haal het met de ArtCraft-launcher", "À obtenir avec le lanceur ArtCraft", "Über den ArtCraft-Launcher erhältlich")

# --- pages
add("page-recent-title", "Recent", "Recent", "Récents", "Zuletzt geöffnet")
add("page-recent-sub", "Documents you opened, newest first", "Documenten die je opende, nieuwste eerst", "Documents ouverts, les plus récents d'abord", "Zuletzt geöffnete Dokumente, neueste zuerst")
add("page-starred-title", "Starred", "Met ster", "Favoris", "Markiert")
add("page-starred-sub", "Documents you marked with a star", "Documenten die je met een ster markeerde", "Documents marqués d'une étoile", "Dokumente, die Sie mit einem Stern markiert haben")
add("page-new-title", "New document", "Nieuw document", "Nouveau document", "Neues Dokument")
add("page-new-sub", "Pick a type, or press its letter", "Kies een type, of druk op de letter ervan", "Choisissez un type ou appuyez sur sa lettre", "Typ wählen oder den Buchstaben drücken")
add("page-apps-title", "Apps", "Apps", "Apps", "Apps")
add("page-apps-sub", "The ArtCraft apps and SolidCraft that work with Glyphra", "De ArtCraft-apps en SolidCraft die met Glyphra samenwerken", "Les apps ArtCraft et SolidCraft qui fonctionnent avec Glyphra", "Die ArtCraft-Apps und SolidCraft, die mit Glyphra zusammenarbeiten")
add("folder-new", "New folder", "Nieuwe map", "Nouveau dossier", "Neuer Ordner")
add("folder-reveal", "Show in folder", "Tonen in map", "Afficher dans le dossier", "Im Ordner zeigen")
add("folder-remove", "Remove from list", "Uit lijst verwijderen", "Retirer de la liste", "Aus Liste entfernen")
add("folder-rename", "Rename folder", "Map hernoemen", "Renommer le dossier", "Ordner umbenennen")
add("folder-delete", "Move folder to Trash", "Map naar prullenbak", "Mettre le dossier à la corbeille", "Ordner in den Papierkorb")
add("kind-folder", "Folder", "Map", "Dossier", "Ordner")

# --- documents table
add("docs-title", "Documents", "Documenten", "Documents", "Dokumente")
T["home-files-count"] = (
    "SEL:{ $n } file|{ $n } files",
    "SEL:{ $n } bestand|{ $n } bestanden",
    "SEL:{ $n } fichier|{ $n } fichiers",
    "SEL:{ $n } Datei|{ $n } Dateien",
)
add("filter-all", "All", "Alle", "Tous", "Alle")
add("group-markdown", "Markdown", "Markdown", "Markdown", "Markdown")
add("group-pdf", "PDF", "PDF", "PDF", "PDF")
add("group-drawing", "Drawing", "Tekening", "Dessin", "Zeichnung")
add("group-office", "Office", "Office", "Bureautique", "Office")
add("group-graphics", "Graphics", "Beeld", "Graphisme", "Grafik")
add("group-media", "Media", "Media", "Médias", "Medien")
add("group-cad", "CAD", "CAD", "CAO", "CAD")
add("search-placeholder", "Search documents", "Documenten zoeken", "Rechercher des documents", "Dokumente suchen")
add("refresh", "Refresh", "Vernieuwen", "Actualiser", "Aktualisieren")
add("col-name", "Name", "Naam", "Nom", "Name")
add("col-kind", "Type", "Type", "Type", "Typ")
add("col-modified", "Modified", "Gewijzigd", "Modifié", "Geändert")
add("col-opened", "Opened", "Geopend", "Ouvert", "Geöffnet")
add("col-size", "Size", "Grootte", "Taille", "Größe")
add("docs-empty", "No documents yet. Start something new above.", "Nog geen documenten. Begin hierboven iets nieuws.", "Pas encore de documents. Commencez quelque chose ci-dessus.", "Noch keine Dokumente. Beginnen Sie oben etwas Neues.")
add("docs-empty-filter", "No documents match.", "Geen documenten gevonden.", "Aucun document ne correspond.", "Keine passenden Dokumente.")
add("row-star", "Star", "Ster geven", "Ajouter aux favoris", "Markieren")
add("row-unstar", "Remove star", "Ster weghalen", "Retirer des favoris", "Markierung entfernen")
add("row-menu-open", "Open", "Openen", "Ouvrir", "Öffnen")
add("row-menu-reveal", "Show in folder", "Tonen in map", "Afficher dans le dossier", "Im Ordner zeigen")
add("row-menu-copy-path", "Copy path", "Pad kopiëren", "Copier le chemin", "Pfad kopieren")
add("row-menu-duplicate", "Duplicate", "Dupliceren", "Dupliquer", "Duplizieren")
add("row-menu-rename", "Rename…", "Hernoemen…", "Renommer…", "Umbenennen…")
add("row-menu-remove-recent", "Remove from Recent", "Uit Recent verwijderen", "Retirer des récents", "Aus „Zuletzt“ entfernen")
add("row-menu-delete", "Move to Trash…", "Naar prullenbak…", "Mettre à la corbeille…", "In den Papierkorb…")

# --- time
add("time-just-now", "just now", "zojuist", "à l'instant", "gerade eben")
add("time-minutes-ago", "{ $n } min ago", "{ $n } min geleden", "il y a { $n } min", "vor { $n } Min.")
add("time-hours-ago", "{ $n } h ago", "{ $n } u geleden", "il y a { $n } h", "vor { $n } Std.")
add("time-yesterday", "yesterday", "gisteren", "hier", "gestern")
T["time-days-ago"] = ("SEL:{ $n } day ago|{ $n } days ago", "SEL:{ $n } dag geleden|{ $n } dagen geleden", "SEL:il y a { $n } jour|il y a { $n } jours", "SEL:vor { $n } Tag|vor { $n } Tagen")
T["time-months-ago"] = ("SEL:{ $n } month ago|{ $n } months ago", "SEL:{ $n } maand geleden|{ $n } maanden geleden", "SEL:il y a { $n } mois|il y a { $n } mois", "SEL:vor { $n } Monat|vor { $n } Monaten")
T["time-years-ago"] = ("SEL:{ $n } year ago|{ $n } years ago", "SEL:{ $n } jaar geleden|{ $n } jaar geleden", "SEL:il y a { $n } an|il y a { $n } ans", "SEL:vor { $n } Jahr|vor { $n } Jahren")

# --- dialogs
add("modal-cancel", "Cancel", "Annuleren", "Annuler", "Abbrechen")
add("modal-ok", "OK", "OK", "OK", "OK")
add("modal-delete", "Move to Trash", "Naar prullenbak", "Mettre à la corbeille", "In den Papierkorb")
add("modal-name-label", "Name", "Naam", "Nom", "Name")
add("modal-rename-file-title", "Rename document", "Document hernoemen", "Renommer le document", "Dokument umbenennen")
add("modal-rename-folder-title", "Rename folder", "Map hernoemen", "Renommer le dossier", "Ordner umbenennen")
add("modal-new-folder-title", "New folder", "Nieuwe map", "Nouveau dossier", "Neuer Ordner")
add("modal-delete-file-title", "Move to Trash?", "Naar de prullenbak?", "Mettre à la corbeille ?", "In den Papierkorb legen?")
add("modal-delete-file-text", "“{ $name }” will be moved to the Trash. You can put it back from there.", "“{ $name }” gaat naar de prullenbak. Je kunt het daar terugzetten.", "« { $name } » sera mis à la corbeille. Vous pourrez l'en retirer.", "„{ $name }“ wird in den Papierkorb gelegt. Dort können Sie es wiederherstellen.")
add("modal-delete-folder-title", "Move folder to Trash?", "Map naar de prullenbak?", "Mettre le dossier à la corbeille ?", "Ordner in den Papierkorb legen?")
add("modal-delete-folder-text", "The folder “{ $name }” and everything in it will be moved to the Trash.", "De map “{ $name }” en alles erin gaat naar de prullenbak.", "Le dossier « { $name } » et son contenu seront mis à la corbeille.", "Der Ordner „{ $name }“ und sein gesamter Inhalt werden in den Papierkorb gelegt.")

# --- document tab
add("doc-readonly", "Read-only preview", "Alleen-lezen voorbeeld", "Aperçu en lecture seule", "Schreibgeschützte Vorschau")
add("doc-unreadable", "This file cannot be shown", "Dit bestand kan niet getoond worden", "Ce fichier ne peut pas être affiché", "Diese Datei kann nicht angezeigt werden")
add("soon-title", "The { $module } editor is coming", "De { $module }-editor komt eraan", "L'éditeur { $module } arrive", "Der { $module }-Editor kommt")
add("soon-text", "Your file is saved and listed here. Editing arrives in version { $release }.", "Je bestand is opgeslagen en staat in de lijst. Bewerken komt in versie { $release }.", "Votre fichier est enregistré et listé ici. La modification arrive dans la version { $release }.", "Ihre Datei ist gespeichert und aufgelistet. Das Bearbeiten kommt in Version { $release }.")

# --- settings
add("settings-title", "Settings", "Instellingen", "Réglages", "Einstellungen")
add("settings-general", "General", "Algemeen", "Général", "Allgemein")
add("settings-keyboard", "Keyboard", "Toetsenbord", "Clavier", "Tastatur")
add("settings-about", "About", "Info", "À propos", "Über")
add("settings-license-mit", "License — MIT", "Licentie — MIT", "Licence — MIT", "Lizenz — MIT")
add("settings-license-apache", "License — Apache-2.0", "Licentie — Apache-2.0", "Licence — Apache-2.0", "Lizenz — Apache-2.0")
add("settings-name", "Your name", "Je naam", "Votre nom", "Ihr Name")
add("settings-name-note", "Used in the greeting on the start screen.", "Wordt gebruikt in de begroeting op het startscherm.", "Utilisé dans le message d'accueil de l'écran de démarrage.", "Wird in der Begrüßung auf dem Startbildschirm verwendet.")
add("settings-theme", "Theme", "Thema", "Thème", "Design")
add("theme-system", "System", "Systeem", "Système", "System")
add("theme-light", "Light", "Licht", "Clair", "Hell")
add("theme-dark", "Dark", "Donker", "Sombre", "Dunkel")
add("settings-language", "Language", "Taal", "Langue", "Sprache")
add("settings-language-system", "System default", "Systeemtaal", "Langue du système", "Systemsprache")
add("settings-language-note", "More languages can be added as language packs; see About.", "Meer talen kunnen als taalpakket worden toegevoegd; zie Info.", "D'autres langues peuvent être ajoutées sous forme de packs ; voir À propos.", "Weitere Sprachen lassen sich als Sprachpaket hinzufügen; siehe Über.")
add("settings-docs-folder", "Documents folder", "Documentenmap", "Dossier des documents", "Dokumentenordner")
add("settings-docs-note", "New documents are saved here. Existing documents stay where they are.", "Nieuwe documenten worden hier bewaard. Bestaande documenten blijven waar ze staan.", "Les nouveaux documents sont enregistrés ici. Les documents existants restent où ils sont.", "Neue Dokumente werden hier gespeichert. Vorhandene Dokumente bleiben, wo sie sind.")
add("settings-docs-choose", "Choose…", "Kies…", "Choisir…", "Auswählen…")
add("settings-docs-default", "Default", "Standaard", "Par défaut", "Standard")
add("settings-open-in", "Open documents", "Documenten openen", "Ouvrir les documents", "Dokumente öffnen")
add("settings-open-tabs", "Tabbed", "Met tabbladen", "Avec onglets", "Mit Tabs")
add("settings-open-windows", "Windowed", "Met vensters", "Avec fenêtres", "Mit Fenstern")
add("settings-open-note", "A document that is already open is brought to the front instead of opened twice.", "Een document dat al open is, komt naar voren in plaats van twee keer te openen.", "Un document déjà ouvert passe au premier plan au lieu de s'ouvrir deux fois.", "Ein bereits geöffnetes Dokument wird nach vorn geholt, statt zweimal geöffnet zu werden.")
add("settings-apply", "Apply", "Toepassen", "Appliquer", "Übernehmen")
add("settings-close", "Close", "Sluiten", "Fermer", "Schließen")
add("error-docs-folder", "The documents folder cannot be used: { $detail }", "De documentenmap kan niet worden gebruikt: { $detail }", "Le dossier des documents est inutilisable : { $detail }", "Der Dokumentenordner kann nicht verwendet werden: { $detail }")
add("settings-keyboard-note", "These are the shortcuts in this version. They also appear in the menus.", "Dit zijn de sneltoetsen in deze versie. Ze staan ook in de menu's.", "Voici les raccourcis de cette version. Ils figurent aussi dans les menus.", "Dies sind die Tastenkürzel dieser Version. Sie stehen auch in den Menüs.")
add("about-version", "Version", "Versie", "Version", "Version")
add("about-source", "Source code", "Broncode", "Code source", "Quellcode")
add("about-config", "Settings folder", "Instellingenmap", "Dossier des réglages", "Einstellungsordner")
add("about-language-packs", "Language packs", "Taalpakketten", "Packs de langue", "Sprachpakete")
add("about-language-packs-note", "A language is a folder with main.ftl and locale.toml. Put it here and restart; no other change is needed. Right-to-left and CJK languages work the same way.", "Een taal is een map met main.ftl en locale.toml. Zet die hier neer en herstart; meer is niet nodig. Talen van rechts naar links en CJK-talen werken op dezelfde manier.", "Une langue est un dossier contenant main.ftl et locale.toml. Placez-le ici et redémarrez ; rien d'autre n'est nécessaire. Les langues de droite à gauche et CJC fonctionnent de la même façon.", "Eine Sprache ist ein Ordner mit main.ftl und locale.toml. Legen Sie ihn hier ab und starten Sie neu; mehr ist nicht nötig. Rechts-nach-links- und CJK-Sprachen funktionieren genauso.")

# --- messages
add("toast-editor-later", "There is nothing to save yet; editing arrives in version 0.3.0.", "Er valt nog niets op te slaan; bewerken komt in versie 0.3.0.", "Rien à enregistrer pour l'instant ; la modification arrive dans la version 0.3.0.", "Es gibt noch nichts zu speichern; das Bearbeiten kommt in Version 0.3.0.")
add("toast-copied", "Copied", "Gekopieerd", "Copié", "Kopiert")
add("toast-craft-missing", "That app is not installed.", "Die app is niet geïnstalleerd.", "Cette app n'est pas installée.", "Diese App ist nicht installiert.")
add("error-save-settings", "Could not save your settings: { $detail }", "Je instellingen konden niet worden opgeslagen: { $detail }", "Impossible d'enregistrer vos réglages : { $detail }", "Die Einstellungen konnten nicht gespeichert werden: { $detail }")
add("error-file-missing", "This file no longer exists: { $detail }", "Dit bestand bestaat niet meer: { $detail }", "Ce fichier n'existe plus : { $detail }", "Diese Datei gibt es nicht mehr: { $detail }")
add("error-too-large", "This file is too large for the preview (over 5 MB).", "Dit bestand is te groot voor het voorbeeld (meer dan 5 MB).", "Ce fichier est trop volumineux pour l'aperçu (plus de 5 Mo).", "Diese Datei ist für die Vorschau zu groß (über 5 MB).")
add("error-create", "Could not create the document: { $detail }", "Het document kon niet worden gemaakt: { $detail }", "Impossible de créer le document : { $detail }", "Das Dokument konnte nicht erstellt werden: { $detail }")
add("error-duplicate", "Could not duplicate the file: { $detail }", "Het bestand kon niet worden gedupliceerd: { $detail }", "Impossible de dupliquer le fichier : { $detail }", "Die Datei konnte nicht dupliziert werden: { $detail }")
add("error-trash", "Could not move it to the Trash: { $detail }", "Het kon niet naar de prullenbak: { $detail }", "Impossible de le mettre à la corbeille : { $detail }", "Es konnte nicht in den Papierkorb gelegt werden: { $detail }")
add("error-open", "Could not open it: { $detail }", "Het kon niet worden geopend: { $detail }", "Impossible de l'ouvrir : { $detail }", "Es konnte nicht geöffnet werden: { $detail }")
add("error-launch", "Could not start the app: { $detail }", "De app kon niet worden gestart: { $detail }", "Impossible de lancer l'app : { $detail }", "Die App konnte nicht gestartet werden: { $detail }")
add("error-language-pack", "A language pack was skipped: { $detail }", "Een taalpakket is overgeslagen: { $detail }", "Un pack de langue a été ignoré : { $detail }", "Ein Sprachpaket wurde übersprungen: { $detail }")

# --- menus
add("menu-app", "Glyphra", "Glyphra", "Glyphra", "Glyphra")
add("menu-file", "File", "Bestand", "Fichier", "Ablage")
add("menu-edit", "Edit", "Bewerken", "Édition", "Bearbeiten")
add("menu-view", "View", "Weergave", "Présentation", "Darstellung")
add("menu-window", "Window", "Venster", "Fenêtre", "Fenster")
add("menu-help", "Help", "Help", "Aide", "Hilfe")
add("menu-new", "New", "Nieuw", "Nouveau", "Neu")
add("menu-close", "Close", "Sluiten", "Fermer", "Schließen")
add("menu-theme", "Theme", "Thema", "Thème", "Design")
add("native-undo", "Undo", "Ongedaan maken", "Annuler", "Widerrufen")
add("native-redo", "Redo", "Opnieuw", "Rétablir", "Wiederholen")
add("native-cut", "Cut", "Knippen", "Couper", "Ausschneiden")
add("native-copy", "Copy", "Kopiëren", "Copier", "Kopieren")
add("native-paste", "Paste", "Plakken", "Coller", "Einsetzen")
add("native-select-all", "Select All", "Selecteer alles", "Tout sélectionner", "Alles auswählen")
add("native-minimize", "Minimize", "Minimaliseren", "Réduire", "Im Dock ablegen")
add("native-zoom", "Zoom", "Zoomen", "Agrandir", "Zoomen")
add("native-fullscreen", "Enter Full Screen", "Volledig scherm", "Passer en plein écran", "Vollbildmodus")
add("native-bring-all-to-front", "Bring All to Front", "Alles naar voren", "Tout ramener au premier plan", "Alle nach vorne bringen")
add("native-hide", "Hide Glyphra", "Verberg Glyphra", "Masquer Glyphra", "Glyphra ausblenden")
add("native-hide-others", "Hide Others", "Verberg andere", "Masquer les autres", "Andere ausblenden")
add("native-show-all", "Show All", "Toon alles", "Tout afficher", "Alle einblenden")
add("native-services", "Services", "Voorzieningen", "Services", "Dienste")

add("cmd-app-quit", "Quit Glyphra", "Stop Glyphra", "Quitter Glyphra", "Glyphra beenden")
add("cmd-settings-open-about", "About Glyphra", "Over Glyphra", "À propos de Glyphra", "Über Glyphra")
add("cmd-settings-open-general", "Settings…", "Instellingen…", "Réglages…", "Einstellungen…")
add("cmd-settings-open-keyboard", "Keyboard shortcuts", "Sneltoetsen", "Raccourcis clavier", "Tastenkürzel")
add("cmd-settings-open-license-mit", "License — MIT", "Licentie — MIT", "Licence — MIT", "Lizenz — MIT")
add("cmd-settings-open-license-apache", "License — Apache-2.0", "Licentie — Apache-2.0", "Licence — Apache-2.0", "Lizenz — Apache-2.0")
add("cmd-file-new", "New document…", "Nieuw document…", "Nouveau document…", "Neues Dokument…")
add("cmd-file-new-markdown", "Markdown document", "Markdown-document", "Document Markdown", "Markdown-Dokument")
add("cmd-file-new-pdf", "PDF document", "PDF-document", "Document PDF", "PDF-Dokument")
add("cmd-file-new-drawing", "Drawing", "Tekening", "Dessin", "Zeichnung")
add("cmd-file-open", "Open…", "Openen…", "Ouvrir…", "Öffnen…")
add("cmd-file-save", "Save", "Bewaar", "Enregistrer", "Sichern")
add("cmd-file-save-as", "Save As…", "Bewaar als…", "Enregistrer sous…", "Sichern unter…")
add("cmd-file-print", "Print…", "Druk af…", "Imprimer…", "Drucken…")
add("cmd-folder-pin", "Add a Folder to the Sidebar…", "Map aan zijbalk toevoegen…", "Ajouter un dossier à la barre latérale…", "Ordner zur Seitenleiste hinzufügen…")
add("cmd-tab-close-active", "Close Tab", "Sluit tabblad", "Fermer l'onglet", "Tab schließen")
add("cmd-tab-reopen", "Reopen Closed Tab", "Heropen gesloten tabblad", "Rouvrir l'onglet fermé", "Geschlossenen Tab wieder öffnen")
add("cmd-tab-next", "Next Tab", "Volgend tabblad", "Onglet suivant", "Nächster Tab")
add("cmd-tab-previous", "Previous Tab", "Vorig tabblad", "Onglet précédent", "Vorheriger Tab")
add("cmd-tab-go-1", "Home Tab", "Starttabblad", "Onglet Accueil", "Start-Tab")
for i in range(2, 10):
    add("cmd-tab-go-%d" % i, "Go to Tab %d" % i, "Ga naar tabblad %d" % i, "Aller à l'onglet %d" % i, "Zu Tab %d" % i)
add("cmd-home-show-home", "Home", "Start", "Accueil", "Start")
add("cmd-home-show-new", "New Document Page", "Pagina Nieuw document", "Page Nouveau document", "Seite „Neues Dokument“")
add("cmd-home-show-recent", "Recent", "Recent", "Récents", "Zuletzt geöffnet")
add("cmd-home-show-starred", "Starred", "Met ster", "Favoris", "Markiert")
add("cmd-home-show-craftapps", "Apps", "Apps", "Apps", "Apps")
add("cmd-home-show-folder", "Documents Folder", "Documentenmap", "Dossier des documents", "Dokumentenordner")
add("cmd-home-refresh", "Refresh the Document List", "Documentenlijst vernieuwen", "Actualiser la liste des documents", "Dokumentenliste aktualisieren")
add("cmd-home-search", "Find…", "Zoek…", "Rechercher…", "Suchen…")
add("cmd-view-theme-cycle", "Switch Theme", "Thema wisselen", "Changer de thème", "Design wechseln")
add("cmd-view-theme-system", "System", "Systeem", "Système", "System")
add("cmd-view-theme-light", "Light", "Licht", "Clair", "Hell")
add("cmd-view-theme-dark", "Dark", "Donker", "Sombre", "Dunkel")
add("cmd-help-docs", "Glyphra Help", "Glyphra-help", "Aide de Glyphra", "Glyphra-Hilfe")
add("cmd-help-issues", "Report an Issue", "Meld een probleem", "Signaler un problème", "Problem melden")

def render(i, lang):
    out = ["# Glyphra — %s. Generated by tools/gen_locales.py; edit the table there, or copy this folder to start a new language." % META[lang][0], ""]
    for id, vals in T.items():
        v = vals[i]
        if v.startswith("SEL:"):
            one, other = v[4:].split("|")
            out += ["%s = { $n ->" % id, "    [one] %s" % one, "   *[other] %s" % other, "}"]
        else:
            out.append("%s = %s" % (id, v))
    return "\n".join(out) + "\n"

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "locales")
for i, lang in enumerate(LANGS):
    d = os.path.join(root, lang)
    os.makedirs(d, exist_ok=True)
    open(os.path.join(d, "main.ftl"), "w", encoding="utf-8").write(render(i, lang))
    name, native = META[lang]
    open(os.path.join(d, "locale.toml"), "w", encoding="utf-8").write('tag = "%s"\nname = "%s"\nnative_name = "%s"\ndirection = "ltr"\n' % (lang, name, native))
print("wrote", len(T), "messages x", len(LANGS), "languages")
