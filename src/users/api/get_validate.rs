use crate::models::{Invitation, Pool, User};
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use chrono::Utc;
use diesel::prelude::*;

// /api/validate/{username}/{token}
pub async fn validate(
    web::Path((username, token)): web::Path<(String, String)>,
    pool: web::Data<Pool>,
    session: Session,
) -> Result<HttpResponse> {
    use crate::schema::{invitations, users};

    let connection: &PgConnection = &pool.get().unwrap();

    let now = Utc::now().naive_utc();
    let invitation: QueryResult<(User, Invitation)> = users::table
        .filter(users::username.eq(&username))
        .inner_join(invitations::table.on(users::email.eq(invitations::email)))
        .filter(invitations::expiration_date.gt(now))
        .first::<(User, Invitation)>(connection);

    if invitation.is_ok() {
        let (member, invitation) = invitation.unwrap();

        if invitation.id.to_string() == token {
            // activate account & empty bio column, use for token account validation
            // should works perfectly, so we can panic! via unwrap()
            let active = diesel::update(users::table.filter(users::id.eq(member.id)))
                .set(users::active.eq(true))
                .get_result::<User>(connection);

            if active.is_ok() {
                // delete invitation
                diesel::delete(invitations::table.filter(invitations::id.eq(invitation.id)))
                    .execute(connection)
                    .unwrap();

                // Message "signup-account-activated"
                session.set(
                    "message",
                    SessionMessage {
                        message: format!(
                            "Votre compte \"{}\" a bien été activé. Bienvenue!",
                            member.username
                        ),
                    },
                )?;
                return Ok(redirect_to("/-/login"));
            }
        }
    }

    // Message "signup-account-activation-error"
    session.set(
        "message",
        SessionMessage {
            message: "Désolé, nous n'avons pas réussi à activer votre compte.".to_string(),
        },
    )?;
    Ok(redirect_to("/"))
}
