use super::NewForm;
use crate::models::{NewTrip, Pool, Trip};
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

// => /api/trip/new
pub async fn new(
    form: web::Form<NewForm>,
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

    // Error Message "message-fill-mandatory-fields"
    if form.name.is_empty() {
        session.set(
            "message",
            SessionMessage {
                message: "Merci de remplir tous les champs obligatoires.".to_string(),
            },
        )?;
        return Ok(redirect_to("/-/new"));
    }

    let mut dt = Utc
        .ymd(form.year as i32, form.month, form.day)
        .and_hms(form.hour, form.minute, 0);
    // Remove timezone minutes from the browser value in minutes.
    // @see form_trip_datetime.html and current-timezone.js
    dt = dt.add(chrono::Duration::minutes(form.timezone_diff));
    let new_trip = NewTrip {
        uuid: uuid::Uuid::new_v4().to_string(),
        name: String::from(&form.name),
        date: SystemTime::from(dt),
        description: form.description.clone().unwrap_or_else(|| String::from("")),
        author: user.id,
        meeting_point: form.meeting_point.clone().unwrap_or_else(|| "".to_string()),
        time: (form.time_hour.unwrap_or(0) * 3600 + form.time_minute.unwrap_or(0) * 60) as i32,
        distance: (form.distance.unwrap_or(0.) * 1000.) as i32,
        elevation: form.elevation.unwrap_or(0) as i32,
    };

    let trip = diesel::insert_into(trips::table)
        .values(new_trip)
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
