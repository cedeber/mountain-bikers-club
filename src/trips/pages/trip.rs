use crate::comments::get_comments_for;
use crate::models::{Pool, Trip, TripUser, User};
use crate::p404;
use crate::schema::{trips, trips_users, users};
use crate::trips::{get_trip_context, get_trip_datetime};
use crate::users::utils::get_user;
use crate::utils::consume_message;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::error::ErrorInternalServerError;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, QueryResult};

/// Trip page handler => /{username}/trip/{uid}
pub async fn trip(
    tmpl: web::Data<tera::Tera>, // Tera
    pool: web::Data<Pool>,       // DB
    user_id: Identity,           // Web token
    session: Session,            // Server session + Cookie
    web::Path((username, uid)): web::Path<(String, String)>,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();

    let user = get_user(connection, &user_id.identity());
    let member = get_user(connection, &Some(username.clone()));

    if member.is_none() {
        // current username in path doesn't exist
        return p404(tmpl).await;
    }

    let member = member.unwrap();
    let trip: QueryResult<(Trip, User)> = trips::table
        .filter(trips::uuid.eq(uid))
        .filter(trips::author.eq(member.id))
        .inner_join(users::table)
        .first::<(Trip, User)>(connection);

    match trip {
        Ok((trip, member)) => {
            // Get participants
            // TODO I believe is_listed can be optimized via SQL
            let participants: Vec<(TripUser, User)> = trips_users::table
                .filter(trips_users::trip_id.eq(trip.id))
                .inner_join(users::table)
                .get_results::<(TripUser, User)>(connection)
                .unwrap_or_else(|_| vec![]);

            let is_listed: Vec<(i32, bool)> = participants
                .clone()
                .into_iter()
                .map(|p| (p.1.id, p.0.will_join))
                .collect();
            let is_listed = match user {
                Some(ref user) => is_listed.iter().find(|&&a| a.0 == user.id),
                None => None,
            };

            // TODO Merge: Trip Context + Default Context?
            let mut ctx = tera::Context::new();
            ctx.insert("user", &user);
            ctx.insert("member", &member);
            ctx.insert("trip", &get_trip_context(connection, &None, &trip));
            ctx.insert("participants", &participants);
            ctx.insert("is_listed", &is_listed);
            ctx.insert("comments", &get_comments_for(connection, &trip.id));
            ctx.insert("datetime", &get_trip_datetime(&trip.date));
            ctx.insert("message", &consume_message(&session));

            let body = tmpl
                .render("trip.html", &ctx)
                .map_err(|e| ErrorInternalServerError(format!("{:#?}", e)))?;

            Ok(HttpResponse::Ok().content_type("text/html").body(body))
        }
        Err(_) => {
            // uuid and username is not a valid trip match
            p404(tmpl).await
        }
    }
}
