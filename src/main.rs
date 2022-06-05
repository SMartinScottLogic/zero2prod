use rocket::launch;
use zero2prod::{
    run,
    telemetry::{get_subscriber, init_subscriber},
};

#[launch]
fn rocket() -> _ {
    let subscriber = get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let port = 8080;
    let figment = rocket::Config::figment().merge(("port", port));

    run(figment)
}
