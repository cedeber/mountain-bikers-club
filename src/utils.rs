use crate::SessionMessage;
use actix_session::Session;
use actix_web::http::header;
use actix_web::{get, web, HttpResponse, Responder};

/// Mail handler => /mail/{to}. Simply create a mailto: url
/// Avoid to have email addresses in the webpage to prevent spam.
#[get("/mail/{to}")]
pub async fn mailto(web::Path(to): web::Path<String>) -> impl Responder {
    redirect_to(&*format!("mailto:{}@mountainbikers.club", &to))
}

/// Redirect to helper (301)
pub fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::Found()
        .header(header::LOCATION, location)
        .finish()
}

/// Helper to pass one message from one page to another.
/// Used as an easy notification system after a redirect.
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
