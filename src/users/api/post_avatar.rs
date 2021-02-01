use crate::models::Pool;
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::{AvatarActor, AvatarMessage, SessionMessage};
use actix::Addr;
use actix_identity::Identity;
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};

// => /api/trip/new
pub async fn avatar(
    mut payload: Multipart,
    addr: web::Data<Addr<AvatarActor>>,
    pool: web::Data<Pool>,
    user_id: Identity,
    session: Session,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let addr = addr.get_ref();
    let user = get_user(connection, &user_id.identity());

    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    // while let Ok(Some(mut field)) = payload.try_next().await {
    if let Ok(Some(mut field)) = payload.try_next().await {
        let mut full = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            full.extend_from_slice(&data);
        }

        addr.do_send(AvatarMessage {
            data: full,
            username: user.username.clone(),
        });
    }

    session.set(
        "message",
        SessionMessage {
            message: "Votre avatar sera mis à jour dans quelques instants.".to_string(),
        },
    )?;
    Ok(redirect_to(&*format!("/{}", user.username)))
}
