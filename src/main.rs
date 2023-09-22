#[macro_use] extern crate rocket;

use std::{sync::Mutex, collections::LinkedList, vec, fs::{File, self}, io::Write, fmt::Display, ops::Add};

use ndarray::{s, OwnedRepr, Dim, ArrayBase};
use rocket::{Config, data::ToByteUnit, Data, State, serde::{json::Json, Serialize}, fs::FileServer};
use rocket_dyn_templates::{context, Template};
use uuid::Uuid;
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

fn array_to_csv<T>(array: ArrayBase<OwnedRepr<T>, Dim<[usize; 2]>>) -> String where T: Clone + Display {
    let mut output = String::new();
    for row in array.outer_iter() {
        let values = row.to_vec();
        let first_value = format!("{}", values[0]);
        output = output.add(&first_value);
        for value in values {
            output = output.add(",").add(&format!("{}", value));
        }

        output = output.add("\n");
    }

    output
}

fn vec_to_2d_list<T>(vector: &mut Vec<T>, cols: usize) -> LinkedList<LinkedList<T>>
    where T: Clone {
    let mut scan = LinkedList::new();

    for (index, value) in vector.iter_mut().enumerate() {
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
fn get_a_scan(c: u8, x: usize, y: usize, data_accessor: &State<DataHandler>) -> Json<AScanJson> {
    let dataset = data_accessor.dataset.lock().expect("Failed to lock dataset");
    let data = dataset.as_ref().expect("No data loaded");

    let channel = data.get_channel(c.into());
    let channel_subset = data.get_channel_subset(c.into()).expect("Subset not found!");
    let a_scan = channel.slice(s![y, x, ..]);

    Json(AScanJson { scan: vec_to_list(a_scan.to_vec()), time_start: channel_subset.min_sample_pos, time_step: channel_subset.sample_resolution })
}

#[get("/header")]
fn get_data_header(data_accessor: &State<DataHandler>) -> Json<data::Header> {
    let dataset = data_accessor.dataset.lock().expect("Failed to lock dataset");
    let us_data = dataset.as_ref().expect("No data loaded");

    Json(us_data.header.clone())
}

#[get("/c_scan/<c>/<start>/<end>")]
fn get_c_scan(c: u8, start: usize, end: usize, data_accessor: &State<DataHandler>) -> Json<LinkedList<LinkedList<i16>>> {
    let dataset = data_accessor.dataset.lock().expect("Failed to lock dataset");
    let us_data = dataset.as_ref().expect("No data loaded");

    let c_scan = us_data.c_scan(c.into(), start, end).unwrap();

    Json(vec_to_2d_list(c_scan.into_raw_vec().as_mut(), us_data.header.samples_x.into()))
}

#[get("/d_scan/<c>/<start>/<end>")]
fn get_d_scan(c: u8, start: usize, end: usize, data_accessor: &State<DataHandler>) -> Json<LinkedList<LinkedList<u32>>> {
    let dataset = data_accessor.dataset.lock().expect("Failed to lock dataset");
    let us_data = dataset.as_ref().expect("No data loaded");

    let d_scan = us_data.d_scan(c.into(), start, end).unwrap();

    Json(vec_to_2d_list(d_scan.into_raw_vec().as_mut(), us_data.header.samples_x.into()))
}

#[get("/")]
fn index(view_accessor: &State<ViewState>) -> Template {
    let view_state = view_accessor.single_view.lock().expect("Failed to lock view state");

    Template::render("index", context! {
        single_view: *view_state
    })
}

#[get("/export?<channel>&<start>&<end>")]
fn export_data(channel: u8, start: usize, end: usize, data_accessor: &State<DataHandler>) -> Vec<u8> {
    let dataset = data_accessor.dataset.lock().expect("Failed to lock dataset");
    let us_data = dataset.as_ref().expect("No data loaded");

    let random_name = Uuid::new_v4();

    let c_scan = us_data.c_scan(channel.into(), start, end).unwrap();
    let d_scan = us_data.d_scan(channel.into(), start, end).unwrap();

    let file = File::create(format!("{}.zip", random_name)).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::DEFLATE)
        .unix_permissions(0o755);

    zip.start_file("c_scan.csv", options).expect("Failed to start c-scan file");
    zip.write_all(array_to_csv::<i16>(c_scan).as_bytes()).expect("Failed to write c-scan CSV");
    
    zip.start_file("d_scan.csv", options).expect("Failed to start d-scan file");
    zip.write_all(array_to_csv::<u32>(d_scan).as_bytes()).expect("Failed to write d-scan CSV");
    zip.finish().expect("Failed to finish file generation");

    let zip_file_content = fs::read(format!("{}.zip", random_name)).unwrap();
    fs::remove_file(format!("{}.zip", random_name)).unwrap();
    zip_file_content
}

#[post("/data/sonoware", data = "<data_request>")]
async fn load_data(data_request: Data<'_>, data_accessor: &State<DataHandler>) -> &'static str {
    let data = data::UsData::load_sonoware(data_request.open(1024.gibibytes())
        .into_bytes().await.unwrap().value);

    let mut data_handler = data_accessor.dataset.lock().expect("Locking dataset failed");

    match data {
        Some(us_data) => {
            *data_handler = Some(us_data);
            "loading successfull"
        }
        None => {
            *data_handler = None;

            println!("Failed to load data");
            "loading failed"
        } 
    }
}

#[get("/state")]
fn get_state(data_accessor: &State<DataHandler>) -> &'static str {
    match *data_accessor.dataset.lock().expect("Locking dataset failed") {
        Some(_) => {"loaded data"}
        None => {"free storage"}
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, load_data, get_state, get_a_scan, get_data_header, get_c_scan,
        get_d_scan, export_data])
        .mount("/js", FileServer::from("./static_files/js/"))
        .mount("/css", FileServer::from("./static_files/css/"))
        .attach(Template::fairing())
        .configure(Config::figment())
        .manage(DataHandler { dataset: Mutex::new(None) })
        .manage(ViewState { single_view: Mutex::new(true) })
}
