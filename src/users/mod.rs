pub use api::get_logout::logout as get_logout;
pub use api::get_validate::validate as get_validate;
pub use api::post_avatar::avatar as post_avatar;
pub use api::post_follow::follow as post_follow;
pub use api::post_follow::unfollow as post_unfollow;
pub use api::post_login::login as post_login;
pub use api::post_new::new as post_new;
pub use api::post_search::search as post_search;
pub use api::post_update::update as post_update;
pub use pages::user::user as user_page;

mod api;
mod pages;
pub mod utils;
