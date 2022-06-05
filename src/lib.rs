use std::path::PathBuf;

use rocket_db_pools::{sqlx, Database};

#[macro_use]
extern crate rocket;

pub mod configuration;
pub mod logging;
pub mod routes;
pub mod startup;
pub mod telemetry;

pub use startup::run;

#[derive(Database)]
#[database("sqlite_logs")]
pub struct Data(sqlx::SqlitePool);

#[get("/<name..>")]
fn hello(name: PathBuf) -> String {
    let name = name
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    if name.is_empty() {
        "Hello world!".to_string()
    } else {
        format!("Hello, {name}!")
    }
}

#[cfg(test)]
mod test {
    use super::run;
    use crate::PathBuf;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn actual_path() {
        let figment = rocket::Config::figment();
        let client = Client::tracked(run(figment)).expect("valid rocket instance");
        let response = client.get(uri!("/Another/User")).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(
            response.body().is_some(),
            "there should be a response payload"
        );
        assert_eq!(response.into_string().unwrap(), "Hello, Another User!");
    }

    #[test]
    fn hello_world() {
        let figment = rocket::Config::figment();
        let client = Client::tracked(run(figment)).expect("valid rocket instance");
        let response = client.get(uri!(super::routes::health)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(
            response.body().is_none(),
            "there should be no response payload"
        );
    }

    #[test]
    fn hello() {
        let figment = rocket::Config::figment();
        let client = Client::tracked(run(figment)).expect("valid rocket instance");
        let response = client.get(uri!(super::hello(""))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(
            response.body().is_some(),
            "there should be a response payload"
        );
        assert_eq!(response.into_string().unwrap(), "Hello world!");
    }

    #[test]
    fn hello_person() {
        let figment = rocket::Config::figment();
        let client = Client::tracked(run(figment)).expect("valid rocket instance");
        let response = client.get(uri!(super::hello("Bob"))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(
            response.body().is_some(),
            "there should be a response payload"
        );
        assert_eq!(response.into_string().unwrap(), "Hello, Bob!");
    }
}
