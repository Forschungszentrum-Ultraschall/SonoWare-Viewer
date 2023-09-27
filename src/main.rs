#[macro_use] extern crate rocket;

use std::{sync::Mutex, collections::LinkedList, vec, fs::{File, self}, io::Write, fmt::Display, ops::Add, path::Path, process::{self}};

use ndarray::{s, OwnedRepr, Dim, ArrayBase};
use rocket::{Config, data::ToByteUnit, Data, State, serde::{json::Json, Serialize}, fs::FileServer, response::status::BadRequest};
use rocket_dyn_templates::{context, Template};
use zip::write::FileOptions;

mod data;
mod test;

#[derive(Serialize)]
struct AScanJson {
    scan: LinkedList<i16>,
    time_start: f32,
    time_step: f32
}

struct DataHandler {
    dataset: Mutex<Option<data::UsData>>
}

struct ViewState {
    single_view: Mutex<bool>
}

fn vec_to_list<T>(vector: Vec<T>) -> LinkedList<T> {
    let mut new_list = LinkedList::new();

    for element in vector {
        new_list.push_back(element);
    }

    new_list
}

fn array_to_csv<T>(array: ArrayBase<OwnedRepr<T>, Dim<[usize; 2]>>, start: f64, scale: f64) -> String 
    where T: Clone + Copy + Display + Into<f64> {
    let mut output = String::new();
    for row in array.outer_iter() {
        let values = row.to_vec();
        let first_value = format!("{}", (values[0]).into() * scale + start);
        output = output.add(&first_value);
        for value in values {
            output = output.add(",").add(&format!("{}", value.into() * scale + start));
        }

        output = output.add("\n");
    }

    output
}

fn vec_to_2d_list<T>(vector: &Vec<T>, cols: usize) -> LinkedList<LinkedList<T>>
    where T: Clone {
    let mut scan = LinkedList::new();

    for (index, value) in vector.iter().enumerate() {
        let row = index / cols;

        if row >= scan.len() {
            scan.push_back(LinkedList::new());
        }

        let current_row_list = scan.back_mut().unwrap();
        current_row_list.push_back(value.clone());
    }

    scan
}

#[get("/a_scan/<c>/<x>/<y>")]
fn get_a_scan(c: u8, x: usize, y: usize, data_accessor: &State<DataHandler>) -> Result<Json<AScanJson>, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let loaded_data = dataset.as_ref();

            match loaded_data {
                Some(data) => {
                    let channel = data.get_channel(c.into());
                    let channel_subset = data.get_channel_subset(c.into()).expect("Subset not found!");
                    let a_scan = channel.slice(s![y, x, ..]);

                    Ok(Json(AScanJson { scan: vec_to_list(a_scan.to_vec()), time_start: channel_subset.min_sample_pos, time_step: channel_subset.sample_resolution }))
                }
                None => {
                    Err(BadRequest(Some(String::from("No data loaded"))))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(Some(String::from("Dataset already used!"))))
        }
    }
}

#[get("/header")]
fn get_data_header(data_accessor: &State<DataHandler>) -> Result<Json<data::Header>, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let us_data = dataset.as_ref();

            match us_data {
                Some(loaded_data) => {
                    Ok(Json(loaded_data.header.clone()))
                }
                None => {
                    println!("No data loaded!");
                    Err(BadRequest(Some(String::from("No data loaded"))))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(Some(String::from("Failed to lock dataset"))))
        }
    }
}

#[get("/c_scan/<c>/<start>/<end>")]
fn get_c_scan(c: u8, start: usize, end: usize, data_accessor: &State<DataHandler>) -> Result<Json<LinkedList<LinkedList<i16>>>, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let us_data = dataset.as_ref();

            match us_data {
                Some(loaded_data) => {
                    let c_scan = loaded_data.c_scan(c.into(), start, end).unwrap();

                    Ok(Json(vec_to_2d_list(c_scan.into_raw_vec().as_mut(), loaded_data.header.samples_x.into())))
                }
                None => {
                    println!("No data loaded");
                    Err(BadRequest(Some(String::from("No data loaded!"))))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(Some(String::from("Data already used"))))
        }
    }
}

#[get("/d_scan/<c>/<start>/<end>")]
fn get_d_scan(c: u8, start: usize, end: usize, data_accessor: &State<DataHandler>) -> Result<Json<LinkedList<LinkedList<u32>>>, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let us_data = dataset.as_ref();
            
            match us_data {
                Some(loaded_data) => {
                    let d_scan = loaded_data.d_scan(c.into(), start, end).unwrap();

                    Ok(Json(vec_to_2d_list(d_scan.into_raw_vec().as_mut(), loaded_data.header.samples_x.into())))
                }
                None => {
                    println!("No data loaded!");
                    Err(BadRequest(Some(String::from("No data loaded"))))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(Some(String::from("Failed to lock dataset"))))
        }
    }
}

#[get("/")]
fn index(view_accessor: &State<ViewState>) -> Result<Template, BadRequest<String>> {
    let view_state = view_accessor.single_view.lock();

    match view_state {
        Ok(current_state) => {
            Ok(Template::render("index", context! {
                single_view: *current_state
            }))
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(Some(String::from("Failed to lock view state"))))
        }
    }
}

#[get("/help")]
fn help() -> Template {
    Template::render("help", context! {})
}

#[get("/export?<channel>&<start>&<end>&<name>")]
fn export_data(channel: u8, start: usize, end: usize, name: String, data_accessor: &State<DataHandler>) -> Result<String, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let us_data = dataset.as_ref();

            match us_data {
                Some(loaded_data) => {
                    let header = loaded_data.get_channel_subset(channel.into()).expect("Failed to find channel subset");

                    let c_scan = loaded_data.c_scan(channel.into(), start, end).unwrap();
                    let d_scan = loaded_data.d_scan(channel.into(), start, end).unwrap();

                    let output_file_path = Path::new("export/").join(format!("{}.zip", name));

                    match File::create(output_file_path) {
                        Ok(file) => {
                            let mut zip = zip::ZipWriter::new(file);
                            let options = FileOptions::default()
                                .compression_method(zip::CompressionMethod::DEFLATE)
                                .unix_permissions(0o755);

                            zip.start_file("c_scan.csv", options).expect("Failed to start c-scan file");
                            zip.write_all(array_to_csv::<i16>(c_scan, 0.0, 1.0).as_bytes()).expect("Failed to write c-scan CSV");
                            
                            zip.start_file("d_scan.csv", options).expect("Failed to start d-scan file");
                            zip.write_all(array_to_csv::<u32>(d_scan, 0.0, (header.sample_resolution / 1000.0).into()).as_bytes()).expect("Failed to write d-scan CSV");
                            zip.finish().expect("Failed to finish file generation");

                            Ok(format!("Created output {} in the programs 'export' directory!", name))
                        }
                        Err(error) => {
                            println!("{}", error);
                            Err(BadRequest(Some(String::from("Failed to create output file!"))))
                        }
                    }
                }
                None => {
                    println!("No data loaded");
                    Err(BadRequest(Some(String::from("No data loaded!"))))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(Some(String::from("Failed to lock dataset"))))
        }
    }
}

#[post("/data/sonoware", data = "<data_request>")]
async fn load_data(data_request: Data<'_>, data_accessor: &State<DataHandler>) -> Result<&'static str, BadRequest<&'static str>> {
    let data = data::UsData::load_sonoware(data_request.open(1024.gibibytes())
        .into_bytes().await.unwrap().value);

    let mut data_handler = data_accessor.dataset.lock().expect("Locking dataset failed");

    match data {
        Some(us_data) => {
            *data_handler = Some(us_data);
            Ok("loading successfull")
        }
        None => {
            *data_handler = None;

            println!("Failed to load data");
            Err(BadRequest(Some("loading failed")))
        } 
    }
}

#[get("/exit")]
fn exit_program() {
    process::exit(0);
}

#[get("/state")]
fn get_state(data_accessor: &State<DataHandler>) -> &'static str {
    let ds = data_accessor.dataset.lock();
    
    match ds {
        Ok(dataset) => {
            match dataset.as_ref() {
                Some(_) => { "loaded data" }
                None => { "free storage" }
            }
        }
        Err(_) => {"free storage"}
    }
}

#[launch]
fn rocket() -> _ {
    match fs::create_dir("export") {
        Ok(_) => {}
        Err(_) => { }
    }
    let _ = open::that("http://localhost:8000");

    rocket::build().mount("/", routes![index, load_data, get_state, get_a_scan, get_data_header, get_c_scan,
        get_d_scan, export_data, help, exit_program])
        .mount("/js", FileServer::from("./static_files/js/"))
        .mount("/css", FileServer::from("./static_files/css/"))
        .mount("/img", FileServer::from("./static_files/img"))
        .attach(Template::fairing())
        .configure(Config::figment())
        .manage(DataHandler { dataset: Mutex::new(None) })
        .manage(ViewState { single_view: Mutex::new(true) })
}
