use super::DeleteForm;
use crate::models::{Pool, Trip};
use crate::schema::trips;
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;

/// Delete trip handler => /api/trip/delete
pub async fn delete(
    pool: web::Data<Pool>, // DB
    user_id: Identity,     // Web token
    session: Session,      // Server session + Cookie
    form: web::Form<DeleteForm>,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    // Need to be identified
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    // Check that trip exists and that the current authenticated user is the author
    let trip: QueryResult<Trip> = trips::table
        .filter(trips::id.eq(form.trip_id))
        .filter(trips::author.eq(user.id))
        .first::<Trip>(connection);

    // You are probably not the author
    // It's ok to fail badly as it should never happen with the UI
    if trip.is_err() {
        // TODO Redirect back to trip?
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let trip = trip.unwrap();

    // Deletion of the trip
    let deletion = diesel::delete(trips::table.filter(trips::id.eq(trip.id))).execute(connection);

    match deletion {
        Ok(_) => {
            session.set(
                "message",
                SessionMessage {
                    message: format!("La sortie \"{}\" a bien été supprimée.", trip.name),
                },
            )?;
            Ok(redirect_to("/"))
        }
        // Message "message-sorry-unexpected-error"
        Err(_) => {
            session.set(
                "message",
                SessionMessage {
                    message: "Une erreur est survenue. Suppression impossible.".to_string(),
                },
            )?;
            Ok(redirect_to(&*format!(
                "/{}/trip/{}",
                form.redirect_trip_username, form.redirect_trip_uuid
            )))
        }
    }
}
