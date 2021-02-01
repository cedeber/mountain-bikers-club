-- This file should undo anything in `up.sql`

ALTER TABLE trips
    DROP COLUMN author;

DROP TABLE users;
