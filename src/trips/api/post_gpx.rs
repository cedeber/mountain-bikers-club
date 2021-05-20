use crate::models::{NewTrip, Pool, Trip};
use crate::schema::trips;
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use geo::prelude::*;
use geojson::GeoJson;
use gpx::read;
use serde_json::json;
use std::io::{BufReader, Cursor, Read};
use std::time::SystemTime;

/// Import and parse a GPX file. Create a new trip once uploaded and parsed => /api/trip/new
pub async fn gpx(
    pool: web::Data<Pool>, // DB
    user_id: Identity,     // Web token
    session: Session,      // Server session + Cookie
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    // Need to be identified
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    // while let Ok(Some(mut field)) = payload.try_next().await {
    if let Ok(Some(mut field)) = payload.try_next().await {
        let mut full = Vec::new();

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            full.extend_from_slice(&data);
        }

        let f = BufReader::new(Cursor::new(full));

        let new_trip = parse_gpx_data(f, &user.id).await;
        if new_trip.is_err() {
            session.set(
                "message",
                SessionMessage {
                    message: "Le fichier ne semble pas être valide ou est erroné.".to_string(),
                },
            )?;
            return Ok(redirect_to("/"));
        }

        // Save the trip metadata into the DB
        let new_trip = new_trip.unwrap();
        let trip = diesel::insert_into(trips::table)
            .values(new_trip)
            .get_result::<Trip>(connection);

        return match trip {
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
        };
    }

    session.set(
        "message",
        SessionMessage {
            message: "Désolé, une erreur inattendue s'est produite.".to_string(),
        },
    )?;
    Ok(redirect_to("/"))
}

async fn parse_gpx_data(reader: impl Read, user_id: &i32) -> Result<NewTrip, gpx::errors::Error> {
    let gpx = read(reader)?;
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut datetime: Option<DateTime<Utc>> = None;
    let mut location: Option<String> = None;
    let mut distance = 0.0;
    let mut uphill = 0.0;
    let mut _downhill = 0.0;

    if let Some(metadata) = gpx.metadata {
        name = metadata.name;
        description = metadata.description;
    }

    for track in gpx.tracks.iter() {
        if name.is_none() {
            // The name is usually saved on the first track (if not in metadata)
            name = track.name.clone();
        }

        if description.is_none() {
            description = track.description.clone();
        }

        for segment in track.segments.iter() {
            let mut waypoints_iter = segment.points.iter();
            let mut previous_waypoint = waypoints_iter.next().unwrap();

            if datetime.is_none() {
                datetime = previous_waypoint.time
            }

            if location.is_none() {
                if let Ok(req) = reqwest::get(&format!(
                    "https://photon.komoot.io/reverse?lon={}&lat={}&limit=1&lang=fr",
                    previous_waypoint.point().lng(),
                    previous_waypoint.point().lat(),
                ))
                .await
                {
                    if let Ok(resp) = req.json::<GeoJson>().await {
                        if let GeoJson::FeatureCollection(ref ctn) = resp {
                            for feature in &ctn.features {
                                if let Some(ref props) = feature.properties {
                                    let default = json!("");
                                    let name = props.get("name").unwrap_or(&default);
                                    let street = props.get("street").unwrap_or(&default);
                                    let city = props.get("city").unwrap_or(&default);
                                    let country = props.get("country").unwrap_or(&default);

                                    location = Some(
                                        format!("{} {} {} {}", name, street, city, country)
                                            .trim()
                                            .to_string()
                                            .replace("\"", ""),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            for current_waypoint in waypoints_iter {
                let geodesic_distance = previous_waypoint
                    .point()
                    .geodesic_distance(&current_waypoint.point());

                let mut elevation_diff: Option<f64> = None;
                if previous_waypoint.elevation.is_some() && current_waypoint.elevation.is_some() {
                    let previous_elevation = previous_waypoint.elevation.unwrap();
                    let current_elevation = current_waypoint.elevation.unwrap();
                    elevation_diff = Some(current_elevation - previous_elevation);
                }

                // thresholds
                // TODO probably also take speed into account?
                if geodesic_distance > 3.0
                    || (elevation_diff.is_some() && elevation_diff.unwrap() > 3.0)
                {
                    // distance
                    distance += geodesic_distance;

                    // elevation
                    if previous_waypoint.elevation.is_some() && current_waypoint.elevation.is_some()
                    {
                        let previous_elevation = previous_waypoint.elevation.unwrap();
                        let current_elevation = current_waypoint.elevation.unwrap();
                        let diff = current_elevation - previous_elevation;

                        if diff >= 0. {
                            uphill += diff
                        } else {
                            _downhill -= diff
                        }
                    }

                    previous_waypoint = current_waypoint;
                }
            }
        }
    }

    Ok(NewTrip {
        uuid: uuid::Uuid::new_v4().to_string(),
        name: name.unwrap_or_else(|| "".to_string()),
        date: SystemTime::from(datetime.unwrap_or_else(Utc::now)),
        description: description.unwrap_or_else(|| "".to_string()),
        author: *user_id,
        meeting_point: location.unwrap_or_else(|| "".to_string()),
        time: 0_i32,
        distance: distance as i32,
        elevation: uphill as i32,
    })
}
