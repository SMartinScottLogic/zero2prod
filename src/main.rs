use rocket::{launch, Config};
use zero2prod::run;

#[launch]
fn rocket() -> _ {
    let port = 8080;
    let config = Config {
        port,
        ..Config::debug_default()
    };
    run().configure(&config)
}
