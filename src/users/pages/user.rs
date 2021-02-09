use crate::models::{Follow, Pool, Trip, TripUser, User};
use crate::p404;
use crate::schema::{followers, trips, users};
use crate::trips::{get_trip_context, TripContext};
use crate::users::utils::get_user;
use crate::utils::{consume_message, redirect_to};
use actix_identity::Identity;
use actix_session::Session;
use actix_web::error::ErrorInternalServerError;
use actix_web::{web, HttpResponse, Result};
use chrono::DateTime;
use diesel::expression::sql_literal::sql;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::cmp::Ordering;
use std::time::SystemTime;

#[derive(Deserialize)]
pub struct UserRequest {
    tab: Option<String>,
    offset: Option<i64>,
    show: Option<String>,
    search: Option<String>,
}

// => /{username}?tab
pub async fn user(
    pool: web::Data<Pool>,
    user_id: Identity,
    tmpl: web::Data<tera::Tera>,
    session: Session,
    web::Path(username): web::Path<String>,
    web::Query(info): web::Query<UserRequest>,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());
    let member = get_user(connection, &Some(username));

    if member.is_none() {
        // user doesn't exist
        return p404(tmpl).await;
    }

    let member = member.unwrap();

    match info.tab {
        None => render_user_overview(&connection, &tmpl, &session, &user, &member),
        Some(ref tab) => match tab.as_ref() {
            "activity" => render_member_activity(&connection, &tmpl, &session, &user, &member),
            "trips" => render_member_trips(&connection, &tmpl, &session, &user, &member, &info),
            "following" => render_member_following(&connection, &tmpl, &session, &user, &member),
            "followers" => render_member_followers(&connection, &tmpl, &session, &user, &member),
            _ => p404(tmpl).await, // TODO render_user_*
        },
    }
}

// --- User Overview ---
fn render_user_overview(
    conn: &PgConnection,
    tmpl: &web::Data<tera::Tera>,
    session: &Session,
    user: &Option<User>,
    member: &User,
) -> Result<HttpResponse> {
    let is_followed = if user.is_some() {
        let user = user.as_ref().unwrap();
        let follow_query: QueryResult<Follow> = followers::table
            .filter(followers::user_id.eq(user.id))
            .filter(followers::following_id.eq(member.id))
            .first::<Follow>(conn);
        follow_query.is_ok()
    } else {
        false
    };

    let following_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::user_id.eq(member.id))
        .load::<Follow>(conn);
    let following = match following_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let followers_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::following_id.eq(member.id))
        .load::<Follow>(conn);
    let followers = match followers_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let mut ctx = tera::Context::new();
    ctx.insert("user", user);
    ctx.insert("member", member);
    ctx.insert("is_followed", &is_followed);
    ctx.insert("following", &following);
    ctx.insert("followers", &followers);
    ctx.insert("message", &consume_message(&session));
    ctx.insert("trips_count", &get_member_trips_count(conn, &member.id));

    let body = tmpl
        .render("user/overview.html", &ctx)
        .map_err(|e| ErrorInternalServerError(format!("{:#?}", e)))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

// --- User Activity ---
fn render_member_activity(
    conn: &PgConnection,
    tmpl: &web::Data<tera::Tera>,
    session: &Session,
    user: &Option<User>,
    member: &User,
) -> Result<HttpResponse> {
    use crate::schema::trips_users;

    // list all trips which you participate at
    let participant_trips: Vec<(TripUser, Trip, User)> = trips_users::table
        .filter(trips_users::user_id.eq(member.id))
        .inner_join(trips::table)
        .inner_join(users::table.on(users::id.eq(trips::author)))
        // .union(trips::table.filter(trips::author.eq(member.id))) // not in diesel 1.4
        .order(trips::date.desc())
        .load::<(TripUser, Trip, User)>(conn)
        .unwrap_or_else(|_| vec![]);

    // We don't need to expose TripUser
    let mut participant_trips: Vec<(TripContext, User)> = participant_trips
        .iter()
        .map(|entry| {
            (
                get_trip_context(conn, &Some(member.clone()), &entry.1),
                entry.2.clone(),
            )
        })
        .collect();

    let organized_trips: Vec<Trip> = trips::table
        .filter(trips::author.eq(member.id))
        .order(trips::date.desc())
        .load::<Trip>(conn)
        .unwrap_or_else(|_| vec![]);

    let mut organized_trips: Vec<(TripContext, User)> = organized_trips
        .iter()
        .map(|entry| {
            (
                get_trip_context(conn, &Some(member.clone()), &entry),
                member.clone(),
            )
        })
        .collect();

    // merge both trips and sort by date
    participant_trips.append(&mut organized_trips);
    participant_trips.sort_by(|a, b| {
        if let Ok(date_a) = DateTime::parse_from_rfc3339(&*a.0.date) {
            if let Ok(date_b) = DateTime::parse_from_rfc3339(&*b.0.date) {
                return date_a.cmp(&date_b).reverse();
            }
        }

        Ordering::Equal
    });

    let is_followed = if user.is_some() {
        let user = user.as_ref().unwrap();
        let follow_query: QueryResult<Follow> = followers::table
            .filter(followers::user_id.eq(user.id))
            .filter(followers::following_id.eq(member.id))
            .first::<Follow>(conn);
        follow_query.is_ok()
    } else {
        false
    };

    let following_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::user_id.eq(member.id))
        .load::<Follow>(conn);
    let following = match following_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let followers_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::following_id.eq(member.id))
        .load::<Follow>(conn);
    let followers = match followers_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let mut ctx = tera::Context::new();
    ctx.insert("user", user);
    ctx.insert("member", member);
    ctx.insert("participant_trips", &participant_trips);
    ctx.insert("is_followed", &is_followed);
    ctx.insert("following", &following);
    ctx.insert("followers", &followers);
    ctx.insert("message", &consume_message(&session));
    ctx.insert("trips_count", &get_member_trips_count(conn, &member.id));

    let body = tmpl
        .render("user/activity.html", &ctx)
        .map_err(|e| ErrorInternalServerError(format!("{:#?}", e)))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

// --- User Trips ---
fn render_member_trips(
    conn: &PgConnection,
    tmpl: &web::Data<tera::Tera>,
    session: &Session,
    user: &Option<User>,
    member: &User,
    info: &UserRequest,
) -> Result<HttpResponse> {
    let offset_step = 25;
    let offset = info.offset.unwrap_or(0);
    let mut len: i64 = 0;

    // list all user's trips
    let member_trips: Vec<Trip> = if let Some(search) = &info.search {
        len = get_member_trips_search_count(conn, &member.id, &search);
        trips::table
            .filter(trips::author.eq(member.id))
            .filter(sql(&*format!(
                "METAPHONE(name, 50) LIKE '%' || METAPHONE('{}', 50) || '%'",
                search
            )))
            .or_filter(sql(&*format!("name LIKE '%{}%'", search)))
            .order(trips::date.desc())
            .limit(offset_step)
            .offset(offset)
            .load::<Trip>(conn)
            .unwrap_or_else(|_| vec![])
    } else {
        match &info.show {
            Some(value) => match value.as_ref() {
                "finished" => {
                    len = get_member_trips_finished_count(conn, &member.id);
                    trips::table
                        .filter(trips::date.lt(SystemTime::now()))
                        .filter(trips::author.eq(member.id))
                        .order(trips::date.desc())
                        .limit(offset_step)
                        .offset(offset)
                        .load::<Trip>(conn)
                        .unwrap_or_else(|_| vec![])
                }
                _ => vec![],
            },
            None => {
                len = get_member_trips_count(conn, &member.id);
                trips::table
                    .filter(trips::author.eq(member.id))
                    .order(trips::date.desc())
                    .limit(offset_step)
                    .offset(offset)
                    .load::<Trip>(conn)
                    .unwrap_or_else(|_| vec![])
            }
        }
    };

    let member_trips: Vec<TripContext> = member_trips
        .iter()
        .map(|trip| get_trip_context(conn, &None, &trip))
        .collect();

    let is_followed = if user.is_some() {
        let user = user.as_ref().unwrap();
        let follow_query: QueryResult<Follow> = followers::table
            .filter(followers::user_id.eq(user.id))
            .filter(followers::following_id.eq(member.id))
            .first::<Follow>(conn);
        follow_query.is_ok()
    } else {
        false
    };

    let following_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::user_id.eq(member.id))
        .load::<Follow>(conn);
    let following = match following_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let followers_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::following_id.eq(member.id))
        .load::<Follow>(conn);
    let followers = match followers_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let mut ctx = tera::Context::new();
    ctx.insert("user", user);
    ctx.insert("member", member);
    ctx.insert("member_trips", &member_trips);
    ctx.insert("is_followed", &is_followed);
    ctx.insert("following", &following);
    ctx.insert("followers", &followers);
    ctx.insert("message", &consume_message(&session));
    ctx.insert("trips_count", &len);
    ctx.insert("offset", &offset);
    ctx.insert("offset_step", &offset_step);
    ctx.insert("show", &info.show);
    ctx.insert("search", &info.search);

    let body = tmpl
        .render("user/trips.html", &ctx)
        .map_err(|e| ErrorInternalServerError(format!("{:#?}", e)))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

// --- User Following ---
fn render_member_following(
    conn: &PgConnection,
    tmpl: &web::Data<tera::Tera>,
    session: &Session,
    user: &Option<User>,
    member: &User,
) -> Result<HttpResponse> {
    let people: Vec<User> = followers::table
        .filter(followers::user_id.eq(member.id))
        .inner_join(users::table.on(users::id.eq(followers::following_id)))
        .load::<(Follow, User)>(conn)
        .unwrap_or_else(|_| vec![])
        .iter()
        .map(|resp| resp.1.clone())
        .collect();

    let is_followed = if user.is_some() {
        let user = user.as_ref().unwrap();
        let follow_query: QueryResult<Follow> = followers::table
            .filter(followers::user_id.eq(user.id))
            .filter(followers::following_id.eq(member.id))
            .first::<Follow>(conn);
        follow_query.is_ok()
    } else {
        false
    };

    let following_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::user_id.eq(member.id))
        .load::<Follow>(conn);
    let following = match following_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let followers_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::following_id.eq(member.id))
        .load::<Follow>(conn);
    let followers = match followers_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let mut ctx = tera::Context::new();
    ctx.insert("user", user);
    ctx.insert("member", member);
    ctx.insert("people", &people);
    ctx.insert("is_followed", &is_followed);
    ctx.insert("following", &following);
    ctx.insert("followers", &followers);
    ctx.insert("message", &consume_message(&session));
    ctx.insert("trips_count", &get_member_trips_count(conn, &member.id));

    let body = tmpl
        .render("user/following.html", &ctx)
        .map_err(|e| ErrorInternalServerError(format!("{:#?}", e)))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

// --- User Followers ---
fn render_member_followers(
    conn: &PgConnection,
    tmpl: &web::Data<tera::Tera>,
    session: &Session,
    user: &Option<User>,
    member: &User,
) -> Result<HttpResponse> {
    let people: Vec<User> = followers::table
        .filter(followers::following_id.eq(member.id))
        .inner_join(users::table.on(users::id.eq(followers::user_id)))
        .load::<(Follow, User)>(conn)
        .unwrap_or_else(|_| vec![])
        .iter()
        .map(|resp| resp.1.clone())
        .collect();

    let is_followed = if user.is_some() {
        let user = user.as_ref().unwrap();
        let follow_query: QueryResult<Follow> = followers::table
            .filter(followers::user_id.eq(user.id))
            .filter(followers::following_id.eq(member.id))
            .first::<Follow>(conn);
        follow_query.is_ok()
    } else {
        false
    };

    let following_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::user_id.eq(member.id))
        .load::<Follow>(conn);
    let following = match following_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let followers_query: QueryResult<Vec<Follow>> = followers::table
        .filter(followers::following_id.eq(member.id))
        .load::<Follow>(conn);
    let followers = match followers_query {
        Ok(q) => q.len(),
        Err(_) => 0,
    };

    let mut ctx = tera::Context::new();
    ctx.insert("user", user);
    ctx.insert("member", member);
    ctx.insert("people", &people);
    ctx.insert("is_followed", &is_followed);
    ctx.insert("following", &following);
    ctx.insert("followers", &followers);
    ctx.insert("message", &consume_message(&session));
    ctx.insert("trips_count", &get_member_trips_count(conn, &member.id));

    let body = tmpl
        .render("user/followers.html", &ctx)
        .map_err(|e| ErrorInternalServerError(format!("{:#?}", e)))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

// --- Utils ---
fn get_member_trips_count(conn: &PgConnection, member_id: &i32) -> i64 {
    trips::table
        .filter(trips::author.eq(member_id))
        .count()
        .get_result(conn)
        .unwrap()
}

fn get_member_trips_finished_count(conn: &PgConnection, member_id: &i32) -> i64 {
    trips::table
        .filter(trips::date.lt(SystemTime::now()))
        .filter(trips::author.eq(member_id))
        .count()
        .get_result(conn)
        .unwrap()
}

fn get_member_trips_search_count(conn: &PgConnection, member_id: &i32, search: &str) -> i64 {
    trips::table
        .filter(trips::author.eq(member_id))
        .filter(sql(&*format!(
            "METAPHONE(name, 50) LIKE '%' || METAPHONE('{}', 50) || '%'",
            search
        )))
        .or_filter(sql(&*format!("name LIKE '%{}%'", search)))
        .count()
        .get_result(conn)
        .unwrap()
}
