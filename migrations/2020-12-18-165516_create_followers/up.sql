-- Your SQL goes here

CREATE TABLE followers
(
    user_id      INT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    following_id INT NOT NULL REFERENCES users (id) ON DELETE CASCADE,

    PRIMARY KEY (user_id, following_id)
);