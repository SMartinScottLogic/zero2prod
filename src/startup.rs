use rocket::response::status;
use rocket::{
    fairing::{self, AdHoc},
    figment::Figment,
    Build, Request, Rocket,
};
use rocket_db_pools::Database;

use crate::routes::health;
use crate::routes::subscribe;
use crate::{hello, Data};

#[catch(422)]
fn unprocessable_error(_req: &Request) -> status::BadRequest<String> {
    status::BadRequest(Some("unprocessable".to_string()))
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
            .mount("/", routes![hello, health, subscribe])
    })
}

pub fn run(figment: Figment) -> rocket::Rocket<rocket::Build> {
    rocket::custom(figment)
        .attach(crate::logging::LoggingMiddleware)
        .attach(sqlx_stage())
        .register("/", catchers![unprocessable_error])
}
