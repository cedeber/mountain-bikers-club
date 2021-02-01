use super::DeleteForm;
use crate::models::{Pool, Trip};
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;

// => /api/trip/delete
pub async fn delete(
    form: web::Form<DeleteForm>,
    pool: web::Data<Pool>,
    user_id: Identity,
    session: Session,
) -> Result<HttpResponse> {
    use crate::schema::trips;

    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    let trip: QueryResult<Trip> = trips::table
        .filter(trips::id.eq(form.trip_id))
        .first::<Trip>(connection);

    match trip {
        Ok(trip) => {
            if trip.author == user.id {
                let deletion = diesel::delete(trips::table.filter(trips::id.eq(form.trip_id)))
                    .execute(connection);

                match deletion {
                    Ok(_) => {
                        session.set(
                            "message",
                            SessionMessage {
                                message: format!(
                                    "La sortie \"{}\" a bien été supprimée.",
                                    trip.name
                                ),
                            },
                        )?;
                        Ok(redirect_to("/"))
                    }
                    // Message "message-sorry-unexpected-error"
                    Err(_) => {
                        session.set(
                            "message",
                            SessionMessage {
                                message: "Une erreur est survenue. Suppression impossible."
                                    .to_string(),
                            },
                        )?;
                        Ok(redirect_to(&*format!(
                            "/{}/trip/{}",
                            form.redirect_trip_username, form.redirect_trip_uuid
                        )))
                    }
                }
            } else {
                // Message "trip-delete-not-allowed"
                session.set(
                    "message",
                    SessionMessage {
                        message: "Vous n'êtes pas autorisé⋅e à supprimer cette sortie.".to_string(),
                    },
                )?;
                Ok(redirect_to(&*format!(
                    "/{}/trip/{}",
                    form.redirect_trip_username, form.redirect_trip_uuid
                )))
            }
        }
        // Message "message-sorry-unexpected-error"
        Err(_) => {
            session.set(
                "message",
                SessionMessage {
                    message: "Mauvais nom d'utilisateur⋅trice ou mot de passe.".to_string(),
                },
            )?;
            Ok(redirect_to(&*format!(
                "/{}/trip/{}",
                form.redirect_trip_username, form.redirect_trip_uuid
            )))
        }
    }
}
