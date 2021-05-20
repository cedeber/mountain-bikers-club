use crate::models::User;
use data_encoding::HEXUPPER;
use diesel::prelude::*;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use ring::{digest, pbkdf2};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// --- Password ---
static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA512;
const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;

pub type Credential = [u8; CREDENTIAL_LEN];

pub enum PasswordError {
    WrongUsernameOrPassword,
}

pub fn get_salt(username: &str) -> Vec<u8> {
    let salt_component = HEXUPPER
        .decode(&dotenv::var("DB_PASSWORD_SALT").unwrap().as_bytes())
        .unwrap();
    let mut salt = Vec::with_capacity(salt_component.len() + username.as_bytes().len());
    salt.extend(&salt_component);
    salt.extend(username.as_bytes());
    salt
}

pub fn generate_password_hash(username: &str, password: &str) -> String {
    let pbkdf2_iterations = NonZeroU32::new(100_000).unwrap();
    let salt = get_salt(username);
    let mut to_store: Credential = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        PBKDF2_ALG,
        pbkdf2_iterations,
        &salt,
        password.as_bytes(),
        &mut to_store,
    );
    HEXUPPER.encode(&to_store)
}

pub fn verify_password(
    username: &str,
    attempted_password: &str,
    actual_password_hash: &str,
) -> Result<(), PasswordError> {
    let pbkdf2_iterations = NonZeroU32::new(100_000).unwrap();
    let salt = get_salt(username);
    let actual_password = HEXUPPER.decode(actual_password_hash.as_bytes()).unwrap();
    pbkdf2::verify(
        PBKDF2_ALG,
        pbkdf2_iterations,
        &salt,
        attempted_password.as_bytes(),
        &actual_password,
    )
    .map_err(|_| PasswordError::WrongUsernameOrPassword)
}

// --- Token ---
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,
    sub: String,
}

pub fn _generate_token(username: &str) -> String {
    let secret = dotenv::var("TOKEN_SECRET").unwrap();
    let exp = SystemTime::now()
        .checked_add(Duration::from_secs(604800)) // 7 days
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let my_claims = Claims {
        exp: exp as usize,
        sub: username.to_string(), // or email address
    };

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();

    token
}

pub fn _verify_token(encoded_token: &str) -> Option<String> {
    let secret = dotenv::var("TOKEN_SECRET").unwrap();
    // let validation = Validation {
    //     sub: Some(username.to_string()),
    //     ..Default::default()
    // };
    let token = decode::<Claims>(
        &encoded_token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    );

    match token {
        Ok(data) => Some(data.claims.sub),
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => None,
            // ErrorKind::InvalidIssuer => None,
            // ErrorKind::InvalidSignature => None,
            // ErrorKind::InvalidSubject => None,
            _ => None,
        },
    }
}

// --- User ---
/// Check in DB if user exists and has an active account
pub fn get_user(connection: &PgConnection, username: &Option<String>) -> Option<User> {
    use crate::schema::users;

    match username {
        Some(username) => {
            let user: QueryResult<User> = users::table
                .filter(users::username.eq(&username.to_lowercase()))
                .filter(users::active.eq(true))
                .first::<User>(connection);

            match user {
                Ok(user) => Some(user),
                Err(_) => None,
            }
        }
        None => None,
    }
}

/// Get real name or username instead
pub fn get_user_display_name(user: &User) -> String {
    if !user.name.is_empty() {
        user.name.clone()
    } else {
        user.username.clone()
    }
}
