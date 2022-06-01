use rocket::launch;
use zero2prod::run;

#[launch]
fn rocket() -> _ {
    run()
}
