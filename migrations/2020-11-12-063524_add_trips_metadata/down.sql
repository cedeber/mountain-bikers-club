-- This file should undo anything in `up.sql`

ALTER TABLE trips
    DROP COLUMN meeting_point;

ALTER TABLE trips
    DROP COLUMN time;

ALTER TABLE trips
    DROP COLUMN distance;

ALTER TABLE trips
    DROP COLUMN elevation;