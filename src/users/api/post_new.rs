use crate::models::{Invitation, NewUser, Pool, User};
use crate::users::api::NewForm;
use crate::users::utils::{generate_password_hash, get_user_display_name};
use crate::utils::redirect_to;
use crate::SessionMessage;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::{PgConnection, RunQueryDsl};
use onig::Regex;
use std::env;
use std::ops::Add;
use uuid::Uuid;

// => /api/user/new
pub async fn new(
    form: web::Form<NewForm>,
    pool: web::Data<Pool>,
    session: Session,
) -> Result<HttpResponse> {
    let mut errors: Vec<String> = Vec::new();

    if form.i_am_not_a_robot.is_some() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    // username should be between 4 and 32 letters, case insensitive, only underscores as special characters
    let username_regex = Regex::new(r"^[a-zA-Z0-9_]{4,32}$").unwrap();

    // email from RFC2822 standards
    let email_regex =
        Regex::new(r"^[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$").unwrap();

    // password are at least 8 characters including a number and an uppercase letter.
    let password_regex =
        Regex::new(r"^(?=.*\d)(?=.*[a-z])(?=.*[A-Z])(?=.*[a-zA-Z]).{8,}$").unwrap();

    if username_regex.find(&form.username).is_none() {
        // "signup-username-invalid"
        errors.push(String::from("Le nom d'utilisateur⋅trice n'est pas valide."));
    }

    if email_regex.find(&form.email).is_none() {
        // "signup-email-invalid"
        errors.push(String::from("L'adresse email n'est pas valide."));
    }

    if password_regex.find(&form.password).is_none() {
        // "signup-password-invalid"
        errors.push(String::from("Le mot de passe n'est pas valide."));
    }

    if !errors.is_empty() {
        let messages = errors.join(" ");
        session.set("message", SessionMessage { message: messages })?;
        Ok(redirect_to("/-/join"))
    } else {
        // FIXME, we must register the user first to check for uniqueness
        // and then register the invitation

        let connection: &PgConnection = &pool.get().unwrap();
        let (invitation, user) = register_user(&connection, &form);

        match user {
            Ok(user) => {
                let invitation_uuid: Uuid;

                match invitation {
                    Ok(invitation) => {
                        invitation_uuid = invitation.id;
                    }
                    Err(_) => {
                        let message = String::from(
                            "Désolé, une erreur inattendue s'est produite. Contactez-nous.",
                        );
                        session.set("message", SessionMessage { message })?;
                        return Ok(redirect_to("/-/join"));
                    }
                }

                let client = reqwest::Client::new();
                let response = client
                    .post("https://api.mailjet.com/v3.1/send")
                    .basic_auth(
                        env::var("EMAIL_SMTP_USERNAME").unwrap(), 
                        Some(env::var("EMAIL_SMTP_PASSWORD").unwrap())
                    )
                    .json(&serde_json::json!({
                      "Messages":[
                        {
                          "From": {
                            "Email": "hello@mountainbikers.club",
                            "Name": "Mountain Bikers Club"
                          },
                          "To": [
                            {
                              "Email": user.email,
                              "Name": get_user_display_name(&user)
                            }
                          ],
                          "TemplateID": 2097999,
                          "TemplateLanguage": true,
                          "Subject": "Confirmation d'inscription. Activez votre compte.",
                          "Variables": {
                            "name": get_user_display_name(&user),
                            "confirmation_link": format!("https://www.mountainbikers.club/api/validate/{}/{}", user.username, invitation_uuid)
                          }
                        }
                      ]
                    }))
                    .send()
                    .await;

                match response {
                    Ok(_) => {
                        // Message "signin-email-send" + args
                        let message = format!("Un email a été envoyé à \"{}\". Merci d'activer votre compte avant de vous connecter.", user.email);
                        session.set("message", SessionMessage { message })?;
                        Ok(redirect_to("/"))
                    }
                    Err(_) => {
                        // Message "signin-send-activation-email-fail"
                        let message = String::from("L'envoi du mail d'activation a échoué. Désolé du désagrément. Merci de nous contacter.");
                        session.set("message", SessionMessage { message })?;
                        Ok(redirect_to("/-/join"))
                    }
                }
            }
            Err(e) => {
                match e {
                    diesel::result::Error::DatabaseError(kind, _info) => match kind {
                        diesel::result::DatabaseErrorKind::UniqueViolation => {
                            // Message "signin-username-email-unique"
                            let message = String::from("Le nom d'utilisateur⋅trice et l'adresse email doivent être uniques.");
                            session.set("message", SessionMessage { message })?;
                            Ok(redirect_to("/-/join"))
                        }
                        _ => {
                            // message "message-sorry-unexpected-error"
                            let message =
                                String::from("Désolé, une erreur inattendue s'est produite.");
                            session.set("message", SessionMessage { message })?;
                            Ok(redirect_to("/-/join"))
                        }
                    },
                    _ => {
                        // Message "message-sorry-unexpected-error"
                        let message = String::from("Désolé, une erreur inattendue s'est produite.");
                        session.set("message", SessionMessage { message })?;
                        Ok(redirect_to("/-/join"))
                    }
                }
            }
        }
    }
}

fn register_user(
    conn: &PgConnection,
    user: &NewForm,
) -> (QueryResult<Invitation>, QueryResult<User>) {
    use crate::schema::{invitations, users};

    let lowercase_username = user.username.to_lowercase();

    // Register the user
    let password_hash = generate_password_hash(&lowercase_username, &user.password);
    let new_user = NewUser {
        username: lowercase_username,
        email: user.email.to_string(),
        password: password_hash,
    };

    let query_user: QueryResult<User> = diesel::insert_into(users::table)
        .values(new_user)
        .get_result(conn);

    let new_invitation = Invitation {
        id: uuid::Uuid::new_v4(),
        email: user.email.to_string(),
        expiration_date: Utc::now().naive_utc().add(Duration::days(3)),
    };

    let query_invitation: QueryResult<Invitation> = if query_user.is_ok() {
        // Register the invitation after because the email column is linked to the users table email column
        // If the registration of the user fails, the registration of the invitation should fails too
        diesel::insert_into(invitations::table)
            .values(new_invitation)
            .get_result(conn)
    } else {
        QueryResult::Err(diesel::NotFound)
    };

    (query_invitation, query_user)
}
