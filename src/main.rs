pub mod load;

use gtk::prelude::*;
use gtk::{self, glib, Application, ApplicationWindow, Button, Orientation};
use crate::load::data;

const APP_ID: &str = "org.gtk_rs.GObjectMemoryManagement5";

fn main() { // -> glib::ExitCode {
    //let app = Application::builder().application_id(APP_ID).build();

    //app.connect_activate(build_ui);

    //app.run()

    data::load_sonoware();
}

fn build_ui(app: &Application) {
    let button_load_data = Button::builder()
        .label("Messdaten laden")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let gtk_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    gtk_box.append(&button_load_data);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&gtk_box)
        .build();

    window.present();
}