-- Your SQL goes here

CREATE TABLE IF NOT EXISTS comments
(
    id      INT          NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    date    TIMESTAMP    NOT NULL DEFAULT NOW(), -- auto
    trip_id INT          NOT NULL REFERENCES trips (id) ON DELETE CASCADE,
    user_id INT          NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    message VARCHAR(500) NOT NULL DEFAULT ''     -- optional
)