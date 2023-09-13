mod data;
mod test;
mod main_view;
mod a_scan_view;

use gtk::prelude::*;
use gtk::{glib, Application, gio};

const APP_ID: &str = "org.fzu.Us_Viewer";

fn main() -> glib::ExitCode {
    gio::resources_register_include!("templates.gresource")
        .expect("Failed to load resources");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let win = main_view::MainView::new(app);
        win.present();
    });

    app.run()
}
