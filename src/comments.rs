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
    trip_id: i32,
    redirect_trip_username: String,
    redirect_trip_uuid: String,
    message: String,
}

// => /api/comment/new
pub async fn post_new(
    form: web::Form<NewCommentForm>,
    pool: web::Data<Pool>,
    user_id: Identity,
    session: Session,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    if form.message.is_empty() {
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

        let insert = diesel::insert_into(comments::table)
            .values(new_comment)
            .execute(connection);

        match insert {
            Ok(_) => Ok(redirect_to(&*format!(
                "/{}/trip/{}",
                form.redirect_trip_username, form.redirect_trip_uuid
            ))),
            Err(_) => {
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
