use rocket::launch;
use zero2prod::run;

#[launch]
fn rocket() -> _ {
    let port = 8080;
    let figment = rocket::Config::figment().merge(("port", port));

    run(figment)
}
