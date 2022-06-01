use std::path::PathBuf;

use rocket::form::Form;
use rocket::http::Status;
use rocket::response::status;
use rocket::Request;

#[macro_use]
extern crate rocket;

#[catch(422)]
fn unprocessable_error(_req: &Request) -> status::BadRequest<String> {
    status::BadRequest(Some("unprocessable".to_string()))
}

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

#[get("/health_check")]
fn health_check() -> Status {
    Status::Ok
}

#[derive(FromForm)]
struct FormData<'r> {
    name: &'r str,
    email: &'r str,
}

#[post("/subscriptions", data = "<form>")]
fn subscribe(form: Form<FormData<'_>>) -> Status {
    println!("{} {}", form.name, form.email);
    Status::Ok
}

pub fn run() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", routes![hello, health_check, subscribe])
        .register("/", catchers![unprocessable_error])
}

#[cfg(test)]
mod test {
    use super::run;
    use crate::PathBuf;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn actual_path() {
        /*
        let rocket = run();
        let rocket = aw!(rocket.ignite()).unwrap();
        rocket.config().port;
        */
        let client = Client::tracked(run()).expect("valid rocket instance");
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
        let client = Client::tracked(run()).expect("valid rocket instance");
        let response = client.get(uri!(super::health_check)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(
            response.body().is_none(),
            "there should be no response payload"
        );
    }

    #[test]
    fn hello() {
        let client = Client::tracked(run()).expect("valid rocket instance");
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
        let client = Client::tracked(run()).expect("valid rocket instance");
        let response = client.get(uri!(super::hello("Bob"))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(
            response.body().is_some(),
            "there should be a response payload"
        );
        assert_eq!(response.into_string().unwrap(), "Hello, Bob!");
    }
}
