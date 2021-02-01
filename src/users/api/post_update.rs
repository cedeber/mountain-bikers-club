use super::UpdateForm;
use crate::models::{Pool, User};
use crate::schema::users;
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;

// => /api/user/update
pub async fn update(
    form: web::Form<UpdateForm>,
    pool: web::Data<Pool>,
    user_id: Identity,
    session: Session,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();

    if form.username != user.username {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let modified_user = diesel::update(users::table.filter(users::username.eq(&form.username)))
        .set((
            users::name.eq(&form.name),
            users::location.eq(&form.location),
            users::bio.eq(&form.bio),
        ))
        .get_result::<User>(connection);

    match modified_user {
        Ok(user) => Ok(redirect_to(&*format!("/{}", user.username,))),
        Err(_) => {
            // Message "message-sorry-unexpected-error"
            session.set(
                "message",
                SessionMessage {
                    message: "Désolé, une erreur inattendue s'est produite.".to_string(),
                },
            )?;
            Ok(redirect_to("/"))
        }
    }
}
