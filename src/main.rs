#[macro_use] extern crate rocket;

use std::{sync::Mutex, collections::LinkedList, vec, fs::{File, self}, io::{Write, Cursor, Read}, fmt::Display, ops::Add, path::Path, process::{self}};
use data::filter_a_scan;
use ndarray::{s, OwnedRepr, Dim, ArrayBase};
use rocket::{Config, data::ToByteUnit, Data, State, serde::{json::Json, Serialize}, fs::FileServer, response::status::BadRequest};
use rocket_dyn_templates::{context, Template};
use zip::write::FileOptions;

mod data;
mod test;

/// Response struct for A-Scans
#[derive(Serialize)]
struct AScanJson {
    /// Values of an A-Scan
    scan: LinkedList<f32>,
    /// Start time of the A-Scan
    time_start: f32,
    /// Time axis resolution
    time_step: f32,
    /// Filtered A-Scan
    filtered_scan: LinkedList<f64>
}

/// Structure for the export config
#[derive(Serialize)]
struct ExportHeader {
    /// List containing the aperture start and end
    aperture: LinkedList<f32>,
    /// Scaling of the horizontal axis
    x_step: f32,
    /// Scaling of the vertical axis
    y_step: f32,
    /// Gain of the current channel
    gain: f64
}

/// Internal handler for the loaded dataset
struct DataHandler {
    /// Mutex for the (loaded) dataset
    dataset: Mutex<Option<data::UsData>>
}

/// Converts a Vector into a LinkedList
/// 
/// # Arguments
/// * `vector`: Vector of type `T`
/// 
/// # Returns
/// A `LinkedList<T>` containing all values of `vector`
fn vec_to_list<T>(vector: Vec<T>) -> LinkedList<T> {
    let mut new_list = LinkedList::new();

    for element in vector {
        new_list.push_back(element);
    }

    new_list
}

/// Converts a 2-D-Array into a CSV representation
/// 
/// # Arguments
/// * `array`: 2-D-Array with data
/// * `start`: Additional offset for the data
/// * `scale`: Additional scaling of the data
/// 
/// # Returns
/// A `String` containing `start + data * scale` for all values in CSV format
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

fn csv_to_array(csv: &String) -> Option<LinkedList<LinkedList<f32>>> {
    let mut array = LinkedList::new();

    for line in csv.lines() {
        let mut new_row = LinkedList::new();

        for element in line.split(',') {
            match element.parse() {
                Ok(number) => {
                    new_row.push_back(number);
                }
                Err(_) => {
                    return None;
                }
            }
        }

        array.push_back(new_row);
    }

    Some(array)
}

/// Converts a Vector into a List of Lists
/// 
/// # Arguments
/// * `vector`: Vector with data
/// * `cols`: Max Length for the inner lists
/// 
/// # Returns
/// A List of Lists with `cols` values
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

/// Returns an A-Scan of a specific channel and position
/// 
/// # Arguments
/// * `c`: Channel index
/// * `x`: Column index
/// * `y`: Row index
/// * `data_accessor`: Internal handler for the loaded data
/// 
/// # Returns
/// If no error occurs, a JSON object will be returned containg the
/// values of the A-Scan and the `start time` and `time resolution`
/// 
/// # Errors
/// An error code will be returned if one the following issues occurs:
/// * The dataset can't be locked
/// * No data is loaded
/// * The channel hasn't been recorded
/// * Any coordinate is invalid
#[get("/a_scan?<c>&<x>&<y>")]
fn get_a_scan(c: usize, x: usize, y: usize, data_accessor: &State<DataHandler>) -> Result<Json<AScanJson>, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let loaded_data = dataset.as_ref();

            match loaded_data {
                Some(data) => {
                    match data.get_channel(c) {
                        Some(channel) => {
                            let channel_subset = data.get_channel_subset(c).expect("Subset not found!");
                            let a_scan = channel.slice(s![y, x, ..]);

                            let a_scan_list = vec_to_list(a_scan.to_vec());

                            Ok(Json(AScanJson { 
                                scan: a_scan_list.clone(),
                                time_start: channel_subset.min_sample_pos, 
                                time_step: channel_subset.sample_resolution,
                                filtered_scan: filter_a_scan(&(a_scan_list.iter().map(|x| *x).collect()), 1).unwrap().iter().map(|x| *x).collect()
                            }))
                        }
                        None => {
                            Err(BadRequest(String::from("Channel not recorded!")))
                        }
                    }
                }
                None => {
                    Err(BadRequest(String::from("No data loaded")))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(String::from("Dataset already used!")))
        }
    }
}

/// Returns the header of a loaded dataset
/// 
/// # Arguments
/// * `data_accessor`: Internal handler of the dataset
/// 
/// # Returns
/// If no error occurs the `Header` will be returned in JSON representation
/// 
/// # Errors
/// An error code will be returned if one of the following issues occurs:
/// * The dataset can't be locked
/// * No data has been loaded
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
                    Err(BadRequest(String::from("No data loaded")))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(String::from("Failed to lock dataset")))
        }
    }
}

/// Get the C-Scan for a specific channel
/// 
/// # Arguments
/// * `c`: Channel index
/// * `start`: start index of the aperture
/// * `end`: end index of the aperture
/// * `data_accessor`: Internal handler for the data
/// 
/// # Returns
/// The JSON representation of the C-Scan values as a 2-D-Array
/// 
/// # Errors
/// An error code will be returned if one of the following issues occurs:
/// * The dataset can't be locked
/// * No data is loaded
/// * The channel hasn't been recorded
#[get("/c_scan?<c>&<start>&<end>&<as_decibel>")]
fn get_c_scan(c: usize, start: usize, end: usize, as_decibel: usize, data_accessor: &State<DataHandler>) -> Result<Json<LinkedList<LinkedList<f64>>>, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let us_data = dataset.as_ref();

            match us_data {
                Some(loaded_data) => {
                    match loaded_data.c_scan(c, start, end, as_decibel == 1) {
                        Some(c_scan) => { 
                            Ok(Json(vec_to_2d_list(c_scan.into_raw_vec().as_mut(), loaded_data.header.samples_x.into()))) 
                        }
                        None => {
                            println!("Failed to create c-scan");
                            Err(BadRequest(String::from("C-Scan can't be created")))
                        }
                    }
                }
                None => {
                    println!("No data loaded");
                    Err(BadRequest(String::from("No data loaded!")))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(String::from("Data already used")))
        }
    }
}

/// Get the D-Scan for a specific channel
/// 
/// # Arguments
/// * `c`: Channel index
/// * `start`: Start index of the aperture
/// * `end`: End index of the aperture
/// * `data_accessor`: Internal handler for the data
/// 
/// # Returns
/// JSON representation of the D-Scan as a 2-D-Array
/// 
/// # Errors
/// An error code is returned if one of the following issues occurs:
/// * The dataset can't be locked
/// * No data is loaded
/// * The channel hasn't been recorded
#[get("/d_scan?<c>&<start>&<end>")]
fn get_d_scan(c: usize, start: usize, end: usize, data_accessor: &State<DataHandler>) -> Result<Json<LinkedList<LinkedList<u32>>>, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let us_data = dataset.as_ref();
            
            match us_data {
                Some(loaded_data) => {
                    match loaded_data.d_scan(c, start, end) {
                        Some(d_scan) => {
                            Ok(Json(vec_to_2d_list(d_scan.into_raw_vec().as_mut(), loaded_data.header.samples_x.into())))
                        }
                        None => {
                            Err(BadRequest(String::from("Failed to generate D-Scan")))
                        }
                    }
                }
                None => {
                    println!("No data loaded!");
                    Err(BadRequest(String::from("No data loaded")))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(String::from("Failed to lock dataset")))
        }
    }
}

/// Get the frontend template
/// 
/// # Returns
/// Returns the rendered `index.html`
#[get("/")]
fn index() -> Template {
    Template::render("index", context!{})
}

/// Load the reference view template
/// 
/// # Returns
/// The rendered `reference.html`
#[get("/reference")]
fn reference() -> Template {
    Template::render("reference", context! {})
}

/// Get the help page
/// 
/// # Returns
/// The rendered `help.html`
#[get("/help")]
fn help() -> Template {
    Template::render("help", context! {})
}

/// Export the loaded data as a ZIP file
/// 
/// # Arguments
/// * `channel`: Channel index
/// * `start`: Start index of the aperture
/// * `end`: End index of the aperture
/// * `name`: Export file name
/// * `data_accessor`: Internal handler for the data
/// 
/// # Returns
/// Message containing the file name. A ZIP file has been created containing
/// the following files:
/// * c_scan.csv
/// * d_scan.csv
/// * config.json
/// 
/// # Errors
/// An error code is returned if one of the following errors occurs:
/// * The dataset can't be locked
/// * No data is loaded
/// * The channel hasn't been recorded
/// * The output file can't be created
#[post("/export?<channel>&<start>&<end>&<name>")]
fn export_data(channel: usize, start: usize, end: usize, name: String, data_accessor: &State<DataHandler>) -> Result<String, BadRequest<String>> {
    let ds = data_accessor.dataset.lock();

    match ds {
        Ok(dataset) => {
            let us_data = dataset.as_ref();

            match us_data {
                Some(loaded_data) => {
                    match loaded_data.get_channel_subset(channel) {
                        Some(header) => {
                            let c_scan_norm = loaded_data.c_scan(channel, start, end, false).unwrap();
                            let d_scan_norm = loaded_data.d_scan(channel, start, end).unwrap();

                            let c_scan_db = loaded_data.c_scan(channel, start, end, true).unwrap();

                            let output_file_path = Path::new("export/").join(format!("{}.zip", name));

                            match File::create(output_file_path) {
                                Ok(file) => {
                                    let output_config = ExportHeader {
                                        aperture: LinkedList::from([header.sample_resolution * start as f32 / 1000.0,
                                            header.sample_resolution * end as f32 / 1000.0]),
                                        x_step: loaded_data.header.res_x,
                                        y_step: loaded_data.header.res_y,
                                        gain: header.gain
                                    };
                                    let json_data = serde_json::to_string_pretty(&output_config).unwrap();

                                    let mut zip = zip::ZipWriter::new(file);
                                    let options = FileOptions::default()
                                        .compression_method(zip::CompressionMethod::DEFLATE)
                                        .unix_permissions(0o755);

                                    zip.start_file("c_scan_norm.csv", options).expect("Failed to start c-scan file");
                                    zip.write_all(array_to_csv::<f64>(c_scan_norm, 0.0, 1.0).as_bytes()).expect("Failed to write c-scan CSV");
                                    
                                    zip.start_file("d_scan.csv", options).expect("Failed to start d-scan file");
                                    zip.write_all(array_to_csv::<u32>(d_scan_norm, 0.0, (header.sample_resolution / 1000.0).into()).as_bytes()).expect("Failed to write d-scan CSV");

                                    zip.start_file("c_scan_db.csv", options).expect("Failed to start c-scan file");
                                    zip.write_all(array_to_csv::<f64>(c_scan_db, 0.0, 1.0).as_bytes()).expect("Failed to write c-scan CSV");

                                    zip.start_file("config.json", options).expect("Failed to create config file");
                                    zip.write_all(json_data.as_bytes()).expect("Failed to write JSON config file.");

                                    zip.finish().expect("Failed to finish file generation");

                                    Ok(format!("Created output {} in the programs 'export' directory!", name))
                                }
                                Err(error) => {
                                    println!("{}", error);
                                    Err(BadRequest(String::from("Failed to create output file!")))
                                }
                            }
                        }
                        None => {
                            println!("Invalid channel provided");
                            Err(BadRequest(String::from("The channel hasn't been recorded!")))
                        }
                    }
                }
                None => {
                    println!("No data loaded");
                    Err(BadRequest(String::from("No data loaded!")))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(String::from("Failed to lock dataset")))
        }
    }
}

/// Import the C-Scan with dB scaling from a `export` file
/// 
/// # Arguments
/// * `data`: Binary data of the export ZIP file
/// 
/// # Returns
/// If successfull a 2D-Array containing the C-Scan with dB scaling
/// will be returned.
/// 
/// # Errors
/// A Bad Request is returned, if one of the following errors occurs:
/// * The ZIP file is invalid
/// * The ZIP file doesn't contain the `c_scan_db.csv`
/// * The `c_scan_db.csv` is invalid or contains no valid float values
#[post("/import", data = "<data>")]
async fn import_data(data: Data<'_>) -> Result<Json<LinkedList<LinkedList<f32>>>, BadRequest<String>> {
    let binary_zip = data.open(1024.gibibytes()).into_bytes().await.unwrap().value;
    let zip_load = zip::ZipArchive::new(Cursor::new(binary_zip));

    match zip_load {
        Ok(mut zip) => {
            let c_scan_db_search = zip.by_name("c_scan_db.csv");

            match c_scan_db_search {
                Ok(mut c_scan_db_file) => {
                    let mut csv = String::new();
                    let c_scan_csv = c_scan_db_file.read_to_string(&mut csv);

                    match c_scan_csv {
                        Ok(_) => {
                            match csv_to_array(&csv) {
                                Some(data) => {
                                    Ok(Json(data))
                                }
                                None => {
                                    Err(BadRequest(String::from("Failed to parse CSV file!")))
                                }
                            }
                        }
                        Err(error) => {
                            println!("{}", error);
                            Err(BadRequest(String::from("Failed to read CSV file!")))
                        }
                    }
                }
                Err(error) => {
                    println!("{}", error);
                    Err(BadRequest(String::from("Failed to find C-Scan with dB scaling!")))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(String::from("Failed to read ZIP file!")))
        }
    }
}

/// Load SonoWare data
/// 
/// # Arguments
/// * `data_request`: Binary stream
/// * `data_accessor`: Internal handler for the data
/// 
/// # Returns
/// A success message
/// 
/// # Errors
/// An error code is returned if one of the following errors occurs:
/// * The dataset can't be locked
/// * The provided data is invalid
#[post("/data/sonoware", data = "<data_request>")]
async fn load_data(data_request: Data<'_>, data_accessor: &State<DataHandler>) -> Result<&'static str, BadRequest<&'static str>> {
    let data = data::UsData::load_sonoware(data_request.open(1024.gibibytes())
        .into_bytes().await.unwrap().value);

    match data_accessor.dataset.lock() {
        Ok(mut data_handler) => {
            match data {
                Some(us_data) => {
                    *data_handler = Some(us_data);
                    Ok("loading successfull")
                }
                None => {
                    *data_handler = None;
        
                    println!("Failed to load data");
                    Err(BadRequest("Loading provided data failed"))
                } 
            }
        }
        Err(error) => {
            println!("{}", error);
            Err(BadRequest("Data handler can't be locked"))
        }
    }
}

/// Exits the program
#[get("/exit")]
fn exit_program() {
    process::exit(0);
}

/// Check if data has been loaded
/// 
/// # Arguments
/// * `data_accessor`: Internal handler for the data
/// 
/// # Returns
/// `loaded data` if data has been loaded already else `free storage`
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
        get_d_scan, export_data, help, exit_program, import_data, reference])
        .mount("/js", FileServer::from("./static_files/js/"))
        .mount("/css", FileServer::from("./static_files/css/"))
        .mount("/img", FileServer::from("./static_files/img"))
        .attach(Template::fairing())
        .configure(Config::figment())
        .manage(DataHandler { dataset: Mutex::new(None) })
}
