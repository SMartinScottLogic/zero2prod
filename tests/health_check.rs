use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use rocket::Config;
use zero2prod::run;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let port = spawn_app().await;
    // Perform HTTP requests against our application, using reqwest with retry.
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    // Act
    let response = client
        .get(&format!("http://127.0.0.1:{port}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Launch our application in the background ~somehow~
async fn spawn_app() -> u16 {
    let config = Config {
        port: 8080,
        ..Config::debug_default()
    };
    let rocket = run().configure(&config).ignite().await.unwrap();
    let port = rocket.config().port;
    let _ = tokio::spawn(rocket.launch());
    port
}
