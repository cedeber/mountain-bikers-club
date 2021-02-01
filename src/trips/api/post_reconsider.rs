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

// => /api/trip/reconsider
pub async fn reconsider(
    form: web::Form<JoinForm>,
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

    let trip_user = diesel::update(
        trips_users::table
            .filter(trips_users::trip_id.eq(form.trip_id))
            .filter(trips_users::user_id.eq(user.id)),
    )
    .set(trips_users::will_join.eq(form.will_join))
    .get_result::<TripUser>(connection);

    match trip_user {
        Ok(_) => Ok(redirect_to(&*format!(
            "/{}/trip/{}",
            form.redirect_trip_username, form.redirect_trip_uuid
        ))),
        Err(_) => {
            // Message "message-sorry-unexpected-error"
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
