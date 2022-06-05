use rocket::{
    fairing::{Fairing, Info, Kind},
    Data, Request, Response,
};
use std::io::Cursor;

pub struct LoggingMiddleware;

#[rocket::async_trait]
impl Fairing for LoggingMiddleware {
    fn info(&self) -> Info {
        Info {
            name: "Request/Response logging",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, payload: &mut Data<'_>) {
        let payload = payload.peek(1024).await;
        let payload =
            String::from_utf8(payload.to_vec()).unwrap_or_else(|_| format!("{payload:?}"));
        info!(
            "REQ: {} {} {:?} {}",
            request.method(),
            request.uri(),
            request.headers(),
            payload
        );
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if response.body_mut().is_none() {
            info!(
                "RESP: {} {} {} {:?}",
                request.method(),
                request.uri(),
                response.status(),
                response.headers()
            );
        } else {
            let payload = response.body_mut().to_string();
            let payload = payload.await.unwrap();
            info!(
                "RESP: {} {} {} {:?} {}",
                request.method(),
                request.uri(),
                response.status(),
                response.headers(),
                payload
            );
            response.set_sized_body(payload.len(), Cursor::new(payload));
        }
    }
}
