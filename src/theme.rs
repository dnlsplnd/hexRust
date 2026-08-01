use gtk::gdk;
use gtk::prelude::*;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

// TRON: cyan on black, with orange for anything that wants attention.
pub fn apply_tron_theme() {
    let provider = CssProvider::new();

    #[allow(deprecated)]
    provider.load_from_data(r#"
/* ------------------------------------------------------------------
   TRON theme

   Palette is defined once here; everything below refers to these names,
   so retuning the scheme means editing this block only.
   ------------------------------------------------------------------ */

@define-color tron_void      #000000;  /* the grid: pure black          */
@define-color tron_surface   #04080b;  /* raised panels, barely lifted  */
@define-color tron_cyan      #6fc3df;  /* body text, the signature blue */
@define-color tron_bright    #d6f6ff;  /* emphasis, selected text       */
@define-color tron_dim       #1f5c70;  /* idle borders, rules           */
@define-color tron_deep      #0a3a47;  /* selection / active fills      */
@define-color tron_amber     #ffa028;  /* alerts, active tab            */

* {
    font-family: monospace;
    font-size: 9pt;
    color: @tron_cyan;
    caret-color: @tron_bright;
    text-shadow: none;
}

window, dialog, .background {
    background-color: @tron_void;
}

box, grid, paned, notebook, headerbar, actionbar {
    background-color: @tron_void;
}

label, text, entry, button, list, row, treeview, textview, viewport, scrolledwindow {
    background-color: @tron_void;
    color: @tron_cyan;
}

/* Selected text reads as a lit region rather than an inverted block.
   GTK exposes this as a "selection" node, not a CSS pseudo-element. */
selection {
    background-color: @tron_deep;
    color: @tron_bright;
}

/* Inputs ---------------------------------------------------------- */

entry, textview, .view, text {
    background-color: @tron_void;
    color: @tron_cyan;
    border: 1px solid @tron_dim;
    border-radius: 0px;
    padding: 4px;
}

/* Focus is the one place a glow earns its keep: it tracks the caret. */
entry:focus, entry:focus-within {
    border-color: @tron_cyan;
    box-shadow: inset 0 0 0 1px @tron_deep;
}

/* Buttons --------------------------------------------------------- */

button {
    background-color: @tron_void;
    color: @tron_cyan;
    border: 1px solid @tron_dim;
    border-radius: 0px;
    padding: 4px 8px;
}

button:hover {
    background-color: @tron_surface;
    border-color: @tron_cyan;
    color: @tron_bright;
}

button:active, button:checked {
    background-color: @tron_deep;
    border-color: @tron_cyan;
    color: @tron_bright;
}

button:disabled {
    color: @tron_dim;
    border-color: @tron_dim;
}

/* Lists ----------------------------------------------------------- */

list row, listview row, listboxrow {
    background-color: @tron_void;
    color: @tron_cyan;
}

list row:hover, listview row:hover, listboxrow:hover {
    background-color: @tron_surface;
}

list row:selected, listview row:selected, listboxrow:selected {
    background-color: @tron_deep;
    color: @tron_bright;
}

/* Notebook tabs --------------------------------------------------- */

notebook > header {
    background-color: @tron_void;
    border-color: @tron_dim;
}

notebook tab {
    background-color: @tron_void;
    color: @tron_dim;
    border: 1px solid @tron_dim;
    border-radius: 0px;
    padding: 4px 8px;
}

notebook tab:hover {
    color: @tron_cyan;
    border-color: @tron_cyan;
}

/* The live tab is the one thing that goes amber, so it reads instantly
   against a wall of cyan. */
notebook tab:checked {
    background-color: @tron_surface;
    color: @tron_amber;
    border-color: @tron_amber;
}

/* Scrollbars ------------------------------------------------------ */

scrollbar {
    background-color: @tron_void;
}

scrollbar slider,
scale slider {
    background-color: @tron_dim;
    border: 0px;
    border-radius: 0px;
    /* Adwaita gives sliders negative margins for the overlay-indicator
       look; combined with our min size that computes to a negative
       minimum, so reset it. */
    margin: 0;
    min-width: 12px;
    min-height: 12px;
}

scrollbar slider:hover,
scale slider:hover {
    background-color: @tron_cyan;
}

scrollbar trough,
scale trough {
    background-color: @tron_void;
    outline: 1px solid @tron_dim;
    border: 0px;
    border-radius: 0px;
    min-height: 12px;
}

/* Rules ----------------------------------------------------------- */

separator {
    background-color: @tron_dim;
}

/* Tooltips and popovers sit above the grid, so lift them slightly. */
tooltip, popover > contents, menu {
    background-color: @tron_surface;
    color: @tron_cyan;
    border: 1px solid @tron_dim;
    border-radius: 0px;
}
"#);
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
