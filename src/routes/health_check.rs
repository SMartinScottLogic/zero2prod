use rocket::http::Status;

#[get("/health_check")]
pub fn health() -> Status {
    Status::Ok
}
