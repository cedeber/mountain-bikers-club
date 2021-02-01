-- Your SQL goes here

CREATE TABLE trips_users
(
    trip_id   INT  NOT NULL REFERENCES trips (id) ON DELETE CASCADE,
    user_id   INT  NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    will_join BOOL NOT NULL DEFAULT FALSE,

    PRIMARY KEY (user_id, trip_id)
);