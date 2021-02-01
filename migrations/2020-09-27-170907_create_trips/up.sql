-- Your SQL goes here

CREATE TABLE trips
(
    id          INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid        CHAR(36)  NOT NULL,            -- UUID, mandatory
    name        TEXT      NOT NULL DEFAULT '', -- mandatory
    date        TIMESTAMP NOT NULL,            -- mandatory
    description TEXT      NOT NULL DEFAULT ''  -- optional
);