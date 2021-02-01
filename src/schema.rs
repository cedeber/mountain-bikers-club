table! {
    comments (id) {
        id -> Int4,
        date -> Timestamp,
        trip_id -> Int4,
        user_id -> Int4,
        message -> Varchar,
    }
}

table! {
    followers (user_id, following_id) {
        user_id -> Int4,
        following_id -> Int4,
    }
}

table! {
    invitations (id) {
        id -> Uuid,
        email -> Varchar,
        expiration_date -> Timestamp,
    }
}

table! {
    trips (id) {
        id -> Int4,
        uuid -> Bpchar,
        name -> Text,
        date -> Timestamp,
        description -> Text,
        author -> Int4,
        meeting_point -> Varchar,
        time -> Int4,
        distance -> Int4,
        elevation -> Int4,
    }
}

table! {
    trips_users (user_id, trip_id) {
        trip_id -> Int4,
        user_id -> Int4,
        will_join -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        creation_date -> Timestamp,
        active -> Bool,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        name -> Varchar,
        location -> Varchar,
        bio -> Varchar,
    }
}

joinable!(comments -> trips (trip_id));
joinable!(comments -> users (user_id));
joinable!(trips -> users (author));
joinable!(trips_users -> trips (trip_id));
joinable!(trips_users -> users (user_id));

allow_tables_to_appear_in_same_query!(
    comments,
    followers,
    invitations,
    trips,
    trips_users,
    users,
);
