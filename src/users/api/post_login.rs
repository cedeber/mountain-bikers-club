use super::LoginForm;
use crate::models::{Pool, User};
use crate::users::utils::verify_password;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;
use diesel::RunQueryDsl;

/// => /api/user/login
pub async fn login(
    form: web::Form<LoginForm>,
    pool: web::Data<Pool>,
    user_id: Identity,
    session: Session,
) -> Result<HttpResponse> {
    use crate::schema::users;

    if form.i_am_not_a_robot.is_some() {
        // The form was probably filled by spam robot, so simply fake a success and redirect to home
        return Ok(HttpResponse::Unauthorized().finish());
    }

    if form.username.is_empty() || form.password.is_empty() {
        session.set(
            "message",
            SessionMessage {
                message: "Merci de remplir les champs nom d'utilisateur⋅trice et mot de passe."
                    .to_string(),
            },
        )?;
        Ok(redirect_to("/-/login"))
    } else {
        let connection: &PgConnection = &pool.get().unwrap();
        let lowercase_username = form.username.to_lowercase();
        let user: QueryResult<User> = users::table
            .filter(users::username.eq(&lowercase_username))
            .first::<User>(connection);

        // Verify login only
        let user = match user {
            Ok(user) => user,
            Err(_) => {
                session.set(
                    "message",
                    SessionMessage {
                        message: "Mauvais nom d'utilisateur⋅trice ou mot de passe.".to_string(),
                    },
                )?;
                return Ok(redirect_to("/-/login"));
            }
        };

        // Verify user is active
        // No need to check password or write the token is the account is not active yet
        if !user.active {
            session.set(
                "message",
                SessionMessage {
                    message: "Votre compte n'a pas encore été activé. Merci de vérifier votre boite email.".to_string(),
                },
            )?;
            return Ok(redirect_to("/-/login"));
        }

        // Verify password is ok
        match verify_password(&lowercase_username, &form.password, &user.password) {
            Ok(_) => {
                user_id.remember(lowercase_username);
                Ok(redirect_to("/"))
            }
            Err(_) => {
                session.set(
                    "message",
                    SessionMessage {
                        message: "Mauvais nom d'utilisateur⋅trice ou mot de passe.".to_string(),
                    },
                )?;
                Ok(redirect_to("/-/login"))
            }
        }
    }
}
