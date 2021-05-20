use crate::schema::{comments, followers, invitations, trips, trips_users, users};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use diesel::{r2d2::ConnectionManager, PgConnection};

/// Type alias to use in multiple places
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// --- Trips ---
#[derive(Associations, Clone, Debug, Identifiable, Queryable, PartialEq, Serialize)]
#[belongs_to(User, foreign_key = "author")]
#[table_name = "trips"]
pub struct Trip {
    pub id: i32,
    pub uuid: String, // Uuid
    pub name: String,
    pub date: SystemTime, // DateTime<Utc>
    pub description: String,
    pub author: i32, // => User(id)
    pub meeting_point: String,
    pub time: i32,
    pub distance: i32,
    pub elevation: i32,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "trips"]
pub struct NewTrip {
    pub uuid: String,
    pub name: String,
    pub date: SystemTime,
    pub description: String,
    pub author: i32,
    pub meeting_point: String,
    pub time: i32,
    pub distance: i32,
    pub elevation: i32,
}

// --- Users ---
#[derive(Clone, Debug, Identifiable, Queryable, PartialEq, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub creation_date: SystemTime,
    pub active: bool,
    pub username: String,
    pub email: String,
    pub password: String,
    pub name: String,
    pub location: String,
    pub bio: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

// --- Trips and Users ---
#[derive(Associations, Clone, Debug, Deserialize, Insertable, Queryable, PartialEq, Serialize)]
#[belongs_to(Trip, foreign_key = "trip_id")]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "trips_users"]
pub struct TripUser {
    pub trip_id: i32,
    pub user_id: i32,
    pub will_join: bool,
}

// --- Comments ---
#[derive(Associations, Clone, Debug, Deserialize, Insertable, Queryable, PartialEq, Serialize)]
#[belongs_to(Trip, foreign_key = "trip_id")]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "comments"]
pub struct Comment {
    pub id: i32,
    pub date: SystemTime,
    pub trip_id: i32,
    pub user_id: i32,
    pub message: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "comments"]
pub struct NewComment {
    pub trip_id: i32,
    pub user_id: i32,
    pub message: String,
}

// --- Invitations ---
#[derive(Associations, Clone, Debug, Deserialize, Insertable, Queryable, PartialEq, Serialize)]
#[belongs_to(User, foreign_key = "email")]
#[table_name = "invitations"]
pub struct Invitation {
    pub id: uuid::Uuid,
    pub email: String,
    pub expiration_date: chrono::NaiveDateTime,
}

// --- Follow ---
#[derive(Associations, Clone, Debug, Deserialize, Insertable, Queryable, PartialEq, Serialize)]
#[belongs_to(User, foreign_key = "user_id", foreign_key = "user_id")]
#[table_name = "followers"]
pub struct Follow {
    pub user_id: i32,
    pub following_id: i32,
}
