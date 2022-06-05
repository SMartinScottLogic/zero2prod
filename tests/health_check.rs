#[macro_use]
extern crate lazy_static;

use tokio_test::block_on;
use tracing::debug;

use std::{
    process::Child,
    sync::{Arc, Mutex},
};

use ctor::{ctor, dtor};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use sqlx::sqlite::SqlitePoolOptions;

static PORT: u16 = 8080;

lazy_static! {
    static ref APP: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug, sqlx::FromRow)]
struct Subscription {
    email: String,
    name: String,
}

async fn clear_tables() {
    if let Ok(pool) = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite://database.sqlite")
        .await
    {
        sqlx::query("DELETE FROM subscriptions")
            .execute(&pool)
            .await
            .expect("Failed to fetch saved subscription.");
    }
}

fn client() -> reqwest_middleware::ClientWithMiddleware {
    // Perform HTTP requests against our application, using reqwest with retry.
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

#[ctor]
async fn spawn_rocket() {
    block_on(clear_tables());
    debug!("spawn rocket");
    let root = std::env::current_exe().unwrap();
    let mut root = root.parent().expect("executable's directory").to_path_buf();
    if root.ends_with("deps") {
        root.pop();
    }
    root.push("zero2prod");
    let mut app_lock = APP.lock().unwrap();
    *app_lock = Some(std::process::Command::new(root).spawn().unwrap());
}

#[dtor]
fn shutdown_rocket() {
    let mut app_lock = APP.lock().unwrap();
    if let Some(mut app) = app_lock.take() {
        app.kill().expect("shutting down rocket");
        println!("shutdown rocket");
    }
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let client = client();
    // Act
    let response = client
        .get(&format!("http://127.0.0.1:{PORT}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let client = client();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("http://127.0.0.1:{PORT}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite://database.sqlite")
        .await
        .unwrap();
    let saved = sqlx::query_as::<_, Subscription>("SELECT email, name FROM subscriptions")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let client = client();

    // Act
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("http://127.0.0.1:{PORT}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
        assert_eq!(400, response.status().as_u16());
    }
}
