-- Your SQL goes here

CREATE TABLE users
(
    id            INT          NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    creation_date TIMESTAMP    NOT NULL DEFAULT NOW(), -- auto
    active        BOOLEAN      NOT NULL DEFAULT FALSE, -- need to validate account via email
    username      VARCHAR(32)  NOT NULL UNIQUE,        -- mandatory
    email         VARCHAR(80)  NOT NULL UNIQUE,        -- mandatory
    password      VARCHAR(128) NOT NULL,               -- mandatory
    name          VARCHAR(80)  NOT NULL DEFAULT '',    -- optional
    location      VARCHAR(120) NOT NULL DEFAULT '',    -- optional
    bio           VARCHAR(280) NOT NULL DEFAULT ''     -- optional
);

ALTER TABLE trips
    ADD COLUMN author INT NOT NULL DEFAULT -1 REFERENCES users (id) ON DELETE CASCADE;