use crate::models::{Comment, NewComment, Pool, User};
use crate::schema::{comments, users};
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;
use serde::Deserialize;

pub fn get_comments_for(connection: &PgConnection, trip_id: &i32) -> Vec<(Comment, User)> {
    comments::table
        .filter(comments::trip_id.eq(trip_id))
        .order(comments::date.asc())
        .inner_join(users::table)
        .get_results::<(Comment, User)>(connection)
        .unwrap_or_else(|_| vec![])
}

// --- POST ---
#[derive(Debug, Deserialize)]
pub struct NewCommentForm {
    // The user_id is retrieved via the Session Web Token
    trip_id: i32,    // Hidden field
    message: String, // Mandatory comment

    // Used to redirect to the trip page
    redirect_trip_username: String, // Hidden field
    redirect_trip_uuid: String,     // Hidden field
}

/// New trip comment handler => /api/comment/new
/// Need user authentification
pub async fn post_new(
    pool: web::Data<Pool>,           // DB
    user_id: Identity,               // Web token
    session: Session,                // Server session + Cookie
    form: web::Form<NewCommentForm>, // HTML Form
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    // TODO, make that generic (+ above?)
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    if form.message.is_empty() {
        // Session message in case it is empty or it fails.
        // This HTML input field is mandatory.

        // message "message-sorry-unexpected-error"
        let message = String::from("Désolé, une erreur inattendue s'est produite.");
        session.set("message", SessionMessage { message })?;
        Ok(redirect_to(&*format!(
            "/{}/trip/{}",
            form.redirect_trip_username, form.redirect_trip_uuid
        )))
    } else {
        let new_comment = NewComment {
            trip_id: form.trip_id,
            user_id: user.id,
            message: String::from(&form.message),
        };

        // Save comment to DB.
        let insert = diesel::insert_into(comments::table)
            .values(new_comment)
            .execute(connection);

        match insert {
            Ok(_) => Ok(redirect_to(&*format!(
                "/{}/trip/{}",
                form.redirect_trip_username, form.redirect_trip_uuid
            ))),
            Err(_) => {
                // Send a message in case the save fails.
                // message "message-sorry-unexpected-error"
                let message = String::from("Désolé, une erreur inattendue s'est produite.");
                session.set("message", SessionMessage { message })?;
                Ok(redirect_to(&*format!(
                    "/{}/trip/{}",
                    form.redirect_trip_username, form.redirect_trip_uuid
                )))
            }
        }
    }
}
