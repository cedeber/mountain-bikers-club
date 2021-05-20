use super::JoinForm;
use crate::models::{Pool, TripUser};
use crate::schema::trips_users;
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;

/// Allow a user to join a trip form another member => /api/trip/join
pub async fn join(
    pool: web::Data<Pool>, // DB
    user_id: Identity,     // Web token
    session: Session,      // Server session + Cookie
    form: web::Form<JoinForm>,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    // Need to be identified
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    let trip_user = diesel::insert_into(trips_users::table)
        .values(TripUser {
            trip_id: form.trip_id,
            user_id: user.id,
            will_join: form.will_join,
        })
        // .returning(trips_users::all_columns)
        .get_result::<TripUser>(connection);

    match trip_user {
        Ok(_) => Ok(redirect_to(&*format!(
            "/{}/trip/{}",
            form.redirect_trip_username, form.redirect_trip_uuid
        ))),
        // Message "message-sorry-unexpected-error"
        Err(_) => {
            session.set(
                "message",
                SessionMessage {
                    message: "Désolé, une erreur inattendue s'est produite.".to_string(),
                },
            )?;
            Ok(redirect_to(&*format!(
                "/{}/trip/{}",
                form.redirect_trip_username, form.redirect_trip_uuid
            )))
        }
    }
}
