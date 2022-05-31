use std::{borrow::Cow, path::PathBuf};

use rocket::http::Status;

#[macro_use]
extern crate rocket;

#[get("/<name..>")]
fn hello(name: PathBuf) -> String {
    let name = name
        .iter()
        .next()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| Cow::from("World"));

    format!("Hello, {name}!")
}

#[get("/health_check")]
fn health_check() -> Status {
    Status::Ok
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, health_check])
}
