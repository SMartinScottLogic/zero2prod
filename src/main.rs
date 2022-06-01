use std::path::PathBuf;

use rocket::http::Status;

#[macro_use]
extern crate rocket;

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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, health_check])
}

#[cfg(test)]
mod test {
    use super::rocket;
    use crate::PathBuf;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn actual_path() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
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
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!(super::health_check)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(
            response.body().is_none(),
            "there should be no response payload"
        );
    }

    #[test]
    fn hello() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
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
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!(super::hello("Bob"))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(
            response.body().is_some(),
            "there should be a response payload"
        );
        assert_eq!(response.into_string().unwrap(), "Hello, Bob!");
    }
}
