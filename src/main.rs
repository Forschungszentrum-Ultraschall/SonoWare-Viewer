pub mod load;

use ndarray::s;
use plotters::prelude::*;
use std::fs::{self};
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
                    let sonoware_data = data::UsData::load_sonoware(file);

                    match generate_plots(&sonoware_data, name.clone().into_string().unwrap()) {
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

fn generate_plots(data: &UsData, path: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generate plots for {}...", path);
    let c_scan = data.c_scan(0);
    let cols = c_scan.shape()[1];

    let single_scan = data.get_channel(0).slice(s![40, 25, ..]);

    let output_dir = path.replace(".sdt", "_rust_image.png");
    let root_area = BitMapBackend::new(output_dir.as_str(), (1024, 768)).into_drawing_area();

    root_area.fill(&WHITE)?;
    let root_area = root_area.titled("Eingelesene Daten", ("sans-serif", 60))?;

    let (upper, lower) = root_area.split_horizontally(512);

    let mut chart = ChartBuilder::on(&upper)
        .margin(5)
        .set_all_label_area_size(50)
        .caption("C-Bild", ("sans-serif", 40))
        .build_cartesian_2d(0i32..c_scan.shape()[1] as i32, -(c_scan.shape()[0] as i32)..0i32)?;

    chart.configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .max_light_lines(4)
        .x_label_offset(35)
        .y_label_offset(25)
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style(("sans-serif", 20))
        .draw()?;

    chart.draw_series(
        c_scan.into_raw_vec()
        .iter().enumerate()
        .map(|(i, v)| {
            let y = i as i32 / cols as i32;
            let x = i as i32 % cols as i32;
            let ratio = (*v as f32 / i16::MAX as f32 * u8::MAX as f32) as u8; 

            Rectangle::new([(x, -y), (x + 1, -y - 1)],
            RGBColor(ratio, u8::MAX - ratio, 0).filled())
        })
    )?;

    let mut cc = ChartBuilder::on(&lower)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .margin_right(20)
        .caption("A-Bild", ("sans-serif", 40))
        .build_cartesian_2d(0f32..single_scan.len() as f32, i16::MIN as f32..i16::MAX as f32)?;
    cc.configure_mesh()
        .x_labels(5)
        .y_labels(3)
        .max_light_lines(4)
        .draw()?;

    cc.draw_series(LineSeries::new(
        single_scan.iter().enumerate().map(|(i, v)| (i as f32, *v as f32)),
        &BLUE
    ))?;

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