pub mod load;

use plotters::prelude::*;
use std::ffi::OsString;
use std::fs::{self, DirEntry};
use gtk::prelude::*;
use gtk::{self, glib, Application, ApplicationWindow, Button, Orientation};
use load::data::UsData;
use crate::load::data;

const APP_ID: &str = "org.gtk_rs.GObjectMemoryManagement5";

fn main() { // -> glib::ExitCode {
    //let app = Application::builder().application_id(APP_ID).build();

    //app.connect_activate(build_ui);

    //app.run()

    let path = "/home/oliver/Viewer_LuftUS/Daten/2022-06-15 MgCr Steine Radenthein/Serie 1";

    for entry in fs::read_dir(path).unwrap() {
        match entry {
            Ok(file) => {
                let name = file.file_name();

                if name.clone().into_string().unwrap().ends_with(".sdt") {
                    let sonoware_data = data::UsData::<i16>::load_sonoware(file);

                    match generate_plots(sonoware_data, name.clone().into_string().unwrap()) {
                        Ok(_) => {}
                        Err(error) => {
                            println!("Error: {}", error);
                        }
                    }
                }
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
}

fn generate_plots(data: UsData<i16>, path: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generate plots for {}...", path);
        
    let output_dir = path.replace(".sdt", "_rust_image.png");
    let root_area = BitMapBackend::new(output_dir.as_str(), (2048, 1536)).into_drawing_area();

    root_area.fill(&WHITE)?;
    let root_area = root_area.titled("Eingelesene Daten", ("sans-serif", 60))?;

    let (upper, lower) = root_area.split_vertically(1024);
    let x_axis = (-3.4f32..3.4).step(0.1);

    let mut cc = ChartBuilder::on(&upper)
        .margin(5)
        .set_all_label_area_size(50)
        .caption("Sin und Cos", ("sans-serif", 40))
        .build_cartesian_2d(-3.4f32..3.4, -1.2f32..1.2f32)?;

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    cc.draw_series(LineSeries::new(x_axis.values().map(|x| (x, x.sin())), &RED))?
        .label("Sinus")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    cc.draw_series(LineSeries::new(
        x_axis.values().map(|x| (x, x.cos())),
        &BLUE
    ))?
    .label("Kosinus")
    .legend(|(x, y)| PathElement::new(vec![(x ,y), (x + 20, y)], &BLUE));

    cc.configure_series_labels().border_style(&BLACK).draw()?;

    let drawing_areas = lower.split_evenly((1, 2));

    for (drawing_area, idx) in drawing_areas.iter().zip(1..) {
        let mut cc = ChartBuilder::on(&drawing_area)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .margin_right(20)
            .caption(format!("y = x^{}", 1 + 2 * idx), ("sans-serif", 40))
            .build_cartesian_2d(-1f32..1f32, -1f32..1f32)?;
        cc.configure_mesh()
            .x_labels(5)
            .y_labels(3)
            .max_light_lines(4)
            .draw()?;

        cc.draw_series(LineSeries::new(
            (-1f32..1f32)
                .step(0.01)
                .values()
                .map(|x| (x, x.powf(idx as f32 * 2.0 + 1.0))),
            &BLUE
        ))?;
    }

    root_area.present().expect("Unable to write result to file.");
    println!("Image stored at {}", output_dir);

    Ok(())
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