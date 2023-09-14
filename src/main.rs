#[macro_use] extern crate rocket;
use rocket::serde::json::Json;

mod data;
mod test;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/data", format = "application/json", data = "<data_request>")]
fn load_data(data_request: Json<data::DataRequestBody>) -> &'static str {
    let read_attempt = std::fs::read(data_request.path);

    match read_attempt{
        Ok(data_binary) => {
            let data = data::UsData::load_sonoware(data_binary);

            match data {
                Some(us_data) => {
                    println!("{:?}", us_data.c_scan(0));
                    println!("{:?}", us_data.d_scan(0));
                }
                None => {
                    println!("Failed to load data");
                }
            }
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }

    "data loaded"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, load_data])
}
