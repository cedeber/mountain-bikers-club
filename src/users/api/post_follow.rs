use crate::models::{Follow, Pool};
use crate::schema::{followers, users};
use crate::users::api::FollowForm;
use crate::users::utils::get_user;
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;
use diesel::RunQueryDsl;

/// => /api/user/follow
pub async fn follow(
    form: web::Form<FollowForm>,
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

    if form.username.is_empty() {
        session.set(
            "message",
            SessionMessage {
                message: "Impossible de suivre personne.".to_string(),
            },
        )?;
        Ok(redirect_to(&*format!("/{}", user.username)))
    } else {
        let following_id: QueryResult<i32> = users::table
            .filter(users::username.eq(&form.username))
            .select(users::id)
            .first::<i32>(connection);

        // following username exists
        if following_id.is_ok() {
            let following_id = following_id.unwrap();

            // save in db
            let follow = Follow {
                user_id: user.id,
                following_id,
            };

            let query_follow: QueryResult<Follow> = diesel::insert_into(followers::table)
                .values(follow)
                .get_result(connection);

            if query_follow.is_err() {
                session.set(
                    "message",
                    SessionMessage {
                        message: "Impossible de suivre ce membre.".to_string(),
                    },
                )?;
            }
            Ok(redirect_to(&*format!("/{}", form.username)))
        } else {
            session.set(
                "message",
                SessionMessage {
                    message: "Impossible de suivre ce membre.".to_string(),
                },
            )?;
            Ok(redirect_to(&*format!("/{}", user.username)))
        }
    }
}

/// => /api/user/unfollow
pub async fn unfollow(
    form: web::Form<FollowForm>,
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

    if form.username.is_empty() {
        session.set(
            "message",
            SessionMessage {
                message: "Impossible de se désabonner de personne.".to_string(),
            },
        )?;
        Ok(redirect_to(&*format!("/{}", user.username)))
    } else {
        let following_id: QueryResult<i32> = users::table
            .filter(users::username.eq(&form.username))
            .select(users::id)
            .first::<i32>(connection);

        // following username exists
        if following_id.is_ok() {
            let following_id = following_id.unwrap();

            let deletion = diesel::delete(
                followers::table
                    .filter(followers::user_id.eq(user.id))
                    .filter(followers::following_id.eq(following_id)),
            )
            .execute(connection);

            if deletion.is_err() {
                session.set(
                    "message",
                    SessionMessage {
                        message: "Impossible de se désabonner de ce membre.".to_string(),
                    },
                )?;
            }
            Ok(redirect_to(&*format!("/{}", form.username)))
        } else {
            session.set(
                "message",
                SessionMessage {
                    message: "Impossible de se désabonner de ce membre.".to_string(),
                },
            )?;
            Ok(redirect_to(&*format!("/{}", user.username)))
        }
    }
}
