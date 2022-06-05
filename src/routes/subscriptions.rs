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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db),
    fields(
        request_id = % Uuid::new_v4(),
        subscriber_email = % form.email,
        subscriber_name = % form.name
    ),
)]
#[post("/subscriptions", data = "<form>")]
pub async fn subscribe(form: Form<FormData<'_>>, db: Connection<Data>) -> Status {
    match insert_subscriber(form, db).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

#[tracing::instrument(name = "Saving new subscriber details in the database", skip(form, db))]
async fn insert_subscriber(
    form: Form<FormData<'_>>,
    mut db: Connection<Data>,
) -> Result<(), sqlx::Error> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    sqlx::query(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(form.email)
    .bind(form.name)
    .bind(since_the_epoch.as_millis().to_string())
    .execute(&mut *db)
    .await?;
    Ok(())
}
