use gtk::gdk;
use gtk::prelude::*;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

// Terminal green on black everywhere.
pub fn apply_terminal_theme() {
    let provider = CssProvider::new();

    #[allow(deprecated)]
<<<<<<< HEAD
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
=======
    provider.load_from_data(r#"
/* Terminal-green theme: pure green on pure black, monospace size 9 */

* {
    font-family: monospace;
    font-size: 9pt;
    color: #00ff00;
    caret-color: #00ff00;
    text-shadow: none;
}

window, dialog, .background {
    background-color: #000000;
}

box, grid, paned, notebook, headerbar, actionbar {
    background-color: #000000;
}

label, text, entry, button, list, row, treeview, textview, viewport, scrolledwindow {
    background-color: #000000;
    color: #00ff00;
}

entry, textview, .view, text {
    background-color: #000000;
    color: #00ff00;
    border: 1px solid #00ff00;
    border-radius: 0px;
    padding: 4px;
}

/* Buttons */
button {
    background-color: #000000;
    color: #00ff00;
    border: 1px solid #00ff00;
    border-radius: 0px;
    padding: 4px 8px;
}

button:hover {
    background-color: #001900;
}

button:active {
    background-color: #003300;
}

/* List rows */
list row, listview row, listboxrow {
    background-color: #000000;
    color: #00ff00;
}

list row:selected, listview row:selected, listboxrow:selected {
    background-color: #003300;
    color: #00ff00;
}

/* Notebook tabs */
notebook > header {
    background-color: #000000;
}

notebook tab {
    background-color: #000000;
    color: #00ff00;
    border: 1px solid #00ff00;
    border-radius: 0px;
    padding: 4px 8px;
}

notebook tab:checked {
    background-color: #003300;
}

/* Scrollbars */
scrollbar {
    background-color: #000000;
}

/* Slider sizing (prevents GTK Gizmo negative minimum warnings) */
scrollbar slider,
scale slider {
    background-color: #000000;
    outline: 1px solid #00ff00;
    border: 0px;
    border-radius: 0px;
    min-width: 12px;
    min-height: 12px;
}

scrollbar trough,
scale trough {
    background-color: #000000;
    outline: 1px solid #00ff00;
    border: 0px;
    border-radius: 0px;
    min-height: 12px;
}

/* Separators */
separator {
    background-color: #00ff00;
}
"#);
gtk::style_context_add_provider_for_display(
>>>>>>> 843bf2f (v0.5.3 – persistent logs, search, ZNC profiles, terminal theme)
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Prefer dark theme without using deprecated GTK API calls.
    if let Some(settings) = gtk::Settings::default() {
        settings.set_property("gtk-application-prefer-dark-theme", &true);
    }
}
