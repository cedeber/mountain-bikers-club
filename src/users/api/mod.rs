use serde::Deserialize;

pub mod get_logout;
pub mod get_validate;
pub mod post_avatar;
pub mod post_follow;
pub mod post_login;
pub mod post_new;
pub mod post_search;
pub mod post_update;

#[derive(Debug, Deserialize)]
pub struct NewForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub i_am_not_a_robot: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub i_am_not_a_robot: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateForm {
    pub username: String,
    pub name: String,
    pub location: String,
    pub bio: String,
}

#[derive(Debug, Deserialize)]
pub struct FollowForm {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchForm {
    member: String,
    tab: String,
    query: String,
}
