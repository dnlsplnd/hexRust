use anyhow::Result;
use gtk::prelude::*;
use gtk::Application;

pub fn run() -> Result<()> {
    let app = Application::builder()
        .application_id("dj.dnsk.hexrust.multiserver")
        .build();

    app.connect_activate(|app| {
        crate::ui::build_ui(app);
    });

    app.run();
    Ok(())
}
