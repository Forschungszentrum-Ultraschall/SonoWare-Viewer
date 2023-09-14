#[macro_use] extern crate rocket;

use rocket::form::Form;
use rocket_dyn_templates::{context, Template};

mod data;
mod test;

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {})
}

#[post("/data", data = "<data_request>")]
fn load_data(data_request: Option<Form<Vec<u8>>>) -> &'static str {
    match data_request {
        Some(form) => {
            let data = data::UsData::load_sonoware(form.into_inner());

            match data {
                Some(us_data) => {
                    println!("{:?}", us_data.c_scan(0));
                    println!("{:?}", us_data.d_scan(0));
                    "data loaded"
                }
                None => {
                    println!("Failed to load data");
                    "processing failed"
                } 
            }
        }
        None => "no data provided"
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, load_data])
        .attach(Template::fairing())
}
