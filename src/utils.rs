use crate::SessionMessage;
use actix_session::Session;
use actix_web::http::header;
use actix_web::{get, web, HttpResponse, Responder};

/// Mailto
#[get("/mail/{to}")]
pub async fn mailto(web::Path(to): web::Path<String>) -> impl Responder {
    redirect_to(&*format!("mailto:{}@mountainbikers.club", &to))
}

/// Redirect to
pub fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::Found()
        .header(header::LOCATION, location)
        .finish()
}

pub fn consume_message(session: &Session) -> Option<SessionMessage> {
    match session.get::<SessionMessage>("message") {
        Ok(session_message) => {
            // Once the message is consumed, remove it.
            // An error message is shown only once
            session.remove("message");
            session_message
        }
        Err(_) => None,
    }
}
