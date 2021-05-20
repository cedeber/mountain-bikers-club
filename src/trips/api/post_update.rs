use super::UpdateForm;
use crate::models::{Pool, Trip};
use crate::schema::trips;
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use chrono::{TimeZone, Utc};
use diesel::prelude::*;
use std::ops::Add;
use std::time::SystemTime;

/// Allow a user to modify its own trip metadata => /api/trip/update
pub async fn update(
    pool: web::Data<Pool>, // DB
    user_id: Identity,     // Web token
    session: Session,      // Server session + Cookie
    form: web::Form<UpdateForm>,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    // Need to be identified
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    if form.name.is_empty() {
        // Message "message-fill-mandatory-fields"
        session.set(
            "message",
            SessionMessage {
                message: "Merci de remplir tous les champs obligatoires.".to_string(),
            },
        )?;
        // TODO Redirect to current trip?
        return Ok(redirect_to("/"));
    }

    let mut dt = Utc
        .ymd(form.year as i32, form.month, form.day)
        .and_hms(form.hour, form.minute, 0);
    dt = dt.add(chrono::Duration::minutes(form.timezone_diff));

    let trip = diesel::update(
        trips::table
            .filter(trips::id.eq(form.trip_id))
            .filter(trips::author.eq(user.id)),
    )
    .set((
        trips::name.eq(&form.name),
        trips::description.eq(form.description.clone().unwrap_or_else(|| "".to_string())),
        trips::date.eq(SystemTime::from(dt)),
        trips::meeting_point.eq(form.meeting_point.clone().unwrap_or_else(|| "".to_string())),
        trips::time
            .eq((form.time_hour.unwrap_or(0) * 3600 + form.time_minute.unwrap_or(0) * 60) as i32),
        trips::distance.eq((form.distance.unwrap_or(0.) * 1000.) as i32),
        trips::elevation.eq(form.elevation.unwrap_or(0) as i32),
    ))
    .get_result::<Trip>(connection);

    match trip {
        Ok(trip) => Ok(redirect_to(&*format!(
            "/{}/trip/{}",
            user.username, trip.uuid
        ))),
        Err(_) => {
            // Message "message-sorry-unexpected-error"
            session.set(
                "message",
                SessionMessage {
                    message: "Désolé, une erreur inattendue s'est produite.".to_string(),
                },
            )?;
            Ok(redirect_to("/"))
        }
    }
}
