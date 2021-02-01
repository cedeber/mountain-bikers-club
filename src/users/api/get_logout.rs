use crate::utils::redirect_to;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::HttpResponse;

pub async fn logout(user_id: Identity, session: Session) -> HttpResponse {
    // remove token auth cookie
    user_id.forget();

    // purge session client side (cookie) and server side.
    session.purge();

    // back to home
    redirect_to("/")
}
