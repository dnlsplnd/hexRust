use gtk::gdk;
use gtk::prelude::*;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

// Terminal green on black everywhere.
pub fn apply_terminal_theme() {
    let provider = CssProvider::new();

    #[allow(deprecated)]
    provider.load_from_data(
        r#"
        * {
            background-color: #000000;
            color: #00ff00;
            caret-color: #00ff00;
            font-family: monospace;
        }

        window, box, paned, scrolledwindow, viewport {
            background-color: #000000;
        }

        entry, textview, textview text {
            background-color: #000000;
            color: #00ff00;
        }

        entry {
            border: 1px solid #005500;
            border-radius: 0px;
            padding: 6px;
            font-size: 12pt;
        }

        button {
            background-color: #000000;
            color: #00ff00;
            border: 1px solid #005500;
            border-radius: 0px;
            padding: 6px 10px;
        }

        button:hover {
            background-color: #001900;
        }

        notebook > header {
            background-color: #000000;
            border-bottom: 1px solid #005500;
        }

        notebook tab {
            background-color: #000000;
            border: 1px solid #005500;
            border-bottom: none;
            border-radius: 0px;
            padding: 6px 10px;
            margin-right: 4px;
            font-size: 11pt;
        }

        notebook tab:checked {
            background-color: #001900;
        }

        textview text selection {
            background-color: #00ff00;
            color: #000000;
        }

        list, list row {
            background-color: #000000;
            color: #00ff00;
            border-color: #005500;
            font-size: 11pt;
        }

        list row:selected {
            background-color: #001900;
        }
        "#,
    );

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Prefer dark theme without using deprecated GTK API calls.
    if let Some(settings) = gtk::Settings::default() {
        settings.set_property("gtk-application-prefer-dark-theme", &true);
    }
}
