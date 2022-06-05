use std::path::PathBuf;

use rocket::fairing::{self, AdHoc};
use rocket::figment::Figment;
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::status;
use rocket::{Build, Request, Rocket};

use rocket_db_pools::{sqlx, Connection, Database};

use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[macro_use]
extern crate rocket;

#[derive(Database)]
#[database("sqlite_logs")]
struct Data(sqlx::SqlitePool);

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

#[derive(Debug, FromForm)]
struct FormData<'r> {
    name: &'r str,
    email: &'r str,
}

#[post("/subscriptions", data = "<form>")]
async fn subscribe(form: Form<FormData<'_>>, mut db: Connection<Data>) -> Status {
    println!("{} {}", form.name, form.email);
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    match sqlx::query(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(form.email)
    .bind(form.name)
    .bind(since_the_epoch.as_millis().to_string())
    // We use `get_ref` to get an immutable reference to the `PgConnection`
    // wrapped by `web::Data`.
    .execute(&mut *db)
    .await
    {
        Ok(_) => {
            info!("Inserted {form:?} into subscriptions table");
            Status::Ok
        }
        Err(e) => {
            error!("Failed to execute query: {}", e);
            Status::InternalServerError
        }
    }
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Data::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/sqlx/migrations").run(&**db).await {
            Ok(_) => {
                info!("Initialized SQLx database");
                Ok(rocket)
            }
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

fn sqlx_stage() -> AdHoc {
    AdHoc::on_ignite("SQLx stage", |rocket| async {
        rocket
            .attach(Data::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .mount("/", routes![hello, health_check, subscribe])
    })
}

pub fn run(figment: Figment) -> rocket::Rocket<rocket::Build> {
    rocket::custom(figment)
        .attach(sqlx_stage())
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
        let response = client.get(uri!(super::health_check)).dispatch();
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
