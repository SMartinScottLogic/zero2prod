use std::time::{SystemTime, UNIX_EPOCH};

use rocket::{form::Form, http::Status};
use rocket_db_pools::{sqlx, Connection};
use tracing::Instrument;
use uuid::Uuid;

use crate::Data;

#[derive(Debug, FromForm)]
pub struct FormData<'r> {
    name: &'r str,
    email: &'r str,
}

#[post("/subscriptions", data = "<form>")]
pub async fn subscribe(form: Form<FormData<'_>>, mut db: Connection<Data>) -> Status {
    let request_id = Uuid::new_v4();
    // Spans, like logs, have an associated level
    // `info_span` creates a span at the info-level
    let request_span = tracing::info_span!(
    "Adding a new subscriber." ,
    % request_id ,
    subscriber_email = % form . email ,
    subscriber_name = % form . name
    );
    let _request_span_guard = request_span.enter();
    // We do not call `.enter` on query_span!
    // `.instrument` takes care of it at the right moments
    // in the query future lifetime
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
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
    // First we attach the instrumentation, then we `.await` it
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            Status::Ok
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            Status::InternalServerError
        }
    }
}
