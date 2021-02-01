pub use api::post_delete::delete as post_delete;
pub use api::post_gpx::gpx as post_gpx;
pub use api::post_join::join as post_join;
pub use api::post_new::new as post_new;
pub use api::post_reconsider::reconsider as post_reconsider;
pub use api::post_update::update as post_update;
pub use pages::trip::trip as trip_page;

use crate::models::{Trip, User};
use crate::schema::trips_users;
use chrono::{prelude::*, DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use std::time::SystemTime;

mod api;
mod pages;

#[derive(Serialize)]
pub struct TripDateTime {
    day: String,
    month: String,
    year: String,
    hour: String,
    minute: String,
}

/// Used to split a date to render a dedicated from
/// You can found it in macros/form_trip_datetime.html
pub fn get_trip_datetime(date: &SystemTime) -> TripDateTime {
    let datetime: DateTime<Utc> = (*date).into();

    TripDateTime {
        day: datetime.day().to_string(),
        month: datetime.month().to_string(),
        year: datetime.year().to_string(),
        hour: datetime.hour().to_string(),
        minute: datetime.minute().to_string(),
    }
}

#[derive(Debug, Serialize)]
pub struct TripContext {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub date: String,
    pub description: String,
    pub meeting_point: String,
    pub time: Option<(String, String)>, // (hours, minutes)
    pub distance: Option<String>,
    pub elevation: i32,
    pub finished: bool,
    pub participate_status: ParticipateStatus,
}

#[derive(Debug, Serialize)]
pub enum ParticipateStatus {
    Organizer,
    Joined,
    Rejected,
    Unknown,
}

pub fn get_trip_context(conn: &PgConnection, member: &Option<User>, trip: &Trip) -> TripContext {
    let trip = trip.clone();

    let date: DateTime<Utc> = trip.date.into();
    let date_string = date.to_rfc3339();

    let time_hour = trip.time / 3600;
    let time_minute = (trip.time % 3600) / 60;
    let time = if time_hour + time_minute > 0 {
        Some((format!("{:02}", time_hour), format!("{:02}", time_minute)))
    } else {
        None
    };

    let distance = if trip.distance == 0 {
        None
    } else {
        let distance = trip.distance as f32;
        Some(format!("{:.2}", distance / 1000.))
    };

    let finished = date.timestamp() < Utc::now().timestamp();

    let participate_status = if let Some(member) = member {
        if member.id == trip.author {
            ParticipateStatus::Organizer
        } else {
            let result: QueryResult<Vec<bool>> = trips_users::table
                .select(trips_users::will_join)
                .filter(trips_users::trip_id.eq(trip.id))
                .filter(trips_users::user_id.eq(member.id))
                .load::<bool>(conn);

            if let Ok(result) = result {
                if result[0] {
                    ParticipateStatus::Joined
                } else {
                    ParticipateStatus::Rejected
                }
            } else {
                ParticipateStatus::Unknown
            }
        }
    } else {
        ParticipateStatus::Unknown
    };

    TripContext {
        id: trip.id,
        uuid: trip.uuid,
        name: trip.name,
        date: date_string,
        description: trip.description,
        meeting_point: trip.meeting_point,
        time,
        distance,
        elevation: trip.elevation,
        finished,
        participate_status,
    }
}
