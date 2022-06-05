use std::time::{SystemTime, UNIX_EPOCH};

use rocket::{form::Form, http::Status};
use rocket_db_pools::{sqlx, Connection};
use uuid::Uuid;

use crate::Data;

#[derive(Debug, FromForm)]
pub struct FormData<'r> {
    name: &'r str,
    email: &'r str,
}

#[post("/subscriptions", data = "<form>")]
pub async fn subscribe(form: Form<FormData<'_>>, mut db: Connection<Data>) -> Status {
    info!("Saving new subscriber details in the database");
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
    .execute(&mut *db)
    .await
    {
        Ok(_) => {
            info!("New subscriber details have been saved");
            Status::Ok
        }
        Err(e) => {
            error!("Failed to execute query: {}", e);
            Status::InternalServerError
        }
    }
}
