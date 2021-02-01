-- Your SQL goes here

CREATE TABLE invitations
(
    id              UUID        NOT NULL PRIMARY KEY,
    email           VARCHAR(80) NOT NULL UNIQUE REFERENCES users (email) ON DELETE CASCADE,
    expiration_date TIMESTAMP   NOT NULL
);