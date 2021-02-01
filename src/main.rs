#[macro_use]
extern crate diesel;

#[macro_use]
extern crate lazy_static;

use crate::models::{Pool, User};
use crate::users::user_page;
use crate::users::utils::{get_user, get_user_display_name};
use crate::utils::{consume_message, redirect_to};
use actix::prelude::*;
use actix_files as fs;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::{CookieSession, Session};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::ContentEncoding;
use actix_web::rt::Runtime;
use actix_web::{guard, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use chrono::{DateTime, Utc};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::ImageOutputFormat;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::time::SystemTime;
use std::{env, io};
use tera::{to_value, try_get_value, Context, ErrorKind, Tera, Value};
use time::Duration;

mod comments;
mod models;
mod schema;
mod trips;
mod users;
mod utils;

lazy_static! {
    // Postgres DB
    pub static ref POOL: models::Pool = {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder()
            // .max_size(45)
            .build(manager)
            .expect("Failed to create pool.")
    };
    // Avatar Actor
    pub static ref AVATAR_ACTOR: Addr<AvatarActor> = SyncArbiter::start(1, || AvatarActor);
    // Rusoto S3 Client
    pub static ref S3_CLIENT: S3Client = {
        let region = Region::Custom {
            name: "".to_owned(),
            endpoint: env::var("AWS_S3_ENDPOINT").expect("AWS_S3_ENDPOINT must be set"),
        };
        S3Client::new(region)
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMessage {
    message: String,
}

async fn index(
    pool: web::Data<Pool>,
    user_id: Identity,
    tmpl: web::Data<tera::Tera>,
    session: Session,
) -> Result<HttpResponse> {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    let body = match user {
        Some(user) => {
            return Ok(redirect_to(&*format!("/{}?tab=activity", user.username)));
        }
        None => {
            let mut ctx = tera::Context::new();
            ctx.insert("message", &consume_message(&session));

            tmpl.render("index.html", &ctx)
                .map_err(|e| ErrorInternalServerError(format!("{:#?}", e)))?
        }
    };

    // Other Cookie. Not from the Cookie Session middleware.
    let cookie = Cookie::build("foo", "bar")
        .max_age(Duration::days(7))
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .content_type("text/html")
        .body(body))
}

/// Define message
struct AvatarMessage {
    data: Vec<u8>,
    username: String,
}

impl Message for AvatarMessage {
    type Result = Result<bool, std::io::Error>;
}

// Define actor
pub struct AvatarActor;

// Provide Actor implementation for our actor
impl Actor for AvatarActor {
    type Context = actix::SyncContext<Self>;
}

/// Define handler for `AvatarMessage` message
impl Handler<AvatarMessage> for AvatarActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: AvatarMessage, _ctx: &mut actix::SyncContext<Self>) -> Self::Result {
        // let rt = runtime::Builder::new_multi_thread()
        //     .enable_all()
        //     .build()
        //     .unwrap();
        let mut rt = Runtime::new().unwrap();

        let connection = &POOL.get().unwrap();
        let data = msg.data;
        if let Ok(img) = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .unwrap()
            .decode()
        {
            let user = get_user(connection, &Some(msg.username));

            if let Some(user) = user {
                let mut image_data: Vec<u8> = vec![];
                let img = img.resize_to_fill(500, 500, FilterType::Lanczos3);
                let _ = img.write_to(&mut image_data, ImageOutputFormat::Jpeg(75));
                let put_cmd = S3_CLIENT.put_object(PutObjectRequest {
                    body: Some(image_data.into()),
                    bucket: env::var("AWS_S3_BUCKET_NAME").expect("AWS_S3_BUCKET_NAME must be set"),
                    key: format!("avatars/{}.jpg", user.id),
                    ..Default::default()
                });
                let _ = rt.block_on(put_cmd);
            }
        }
        Ok(true)
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Load .env variables as ENV variables
    dotenv::dotenv().ok();

    // Setup the logger used by actix
    env_logger::init();

    // Set ACTIX_* in the production server to configure
    let server_url = env::var("ACTIX_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("ACTIX_PORT").unwrap_or_else(|_| "8080".to_string());

    // Launch the server
    HttpServer::new(|| {
        let is_dev = env::var("RUST_ENV").is_ok();
        let domain = if is_dev {
            "localhost"
        } else {
            "www.mountainbikers.club"
        };

        // Tera templates
        // TODO lazy_static
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera.register_filter("display_name", display_name);
        tera.register_filter("avatar_url", avatar_url);
        tera.register_filter("date_rfc", date_rfc);

        App::new()
            .data(tera)
            .data(POOL.clone())
            .data(AVATAR_ACTOR.clone())
            // cookie identity middleware
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32]) // <- create cookie identity policy
                    .name("auth-cookie")
                    .max_age(Duration::days(7).as_seconds_f64() as i64)
                    .same_site(SameSite::Lax)
                    .secure(!is_dev),
            ))
            // cookie session middleware
            .wrap(
                CookieSession::signed(&[0; 32])
                    .same_site(SameSite::Lax)
                    .secure(!is_dev),
            )
            .wrap(middleware::Compress::new(ContentEncoding::Br))
            // enable logger
            .wrap(middleware::Logger::default())
            // router
            .service(web::resource("/").to(index))
            .service(
                web::scope("/-")
                    .service(utils::mailto) // => /mail/{to}
                    .service(web::resource("/{page}").route(web::get().to(pages))),
            )
            // api
            .service(
                web::scope("/api")
                    .guard(guard::Host(domain))
                    .route("/user/new", web::post().to(users::post_new))
                    .route("/user/login", web::post().to(users::post_login))
                    .route("/user/update", web::post().to(users::post_update))
                    .route("/user/avatar", web::post().to(users::post_avatar))
                    .route("/user/follow", web::post().to(users::post_follow))
                    .route("/user/unfollow", web::post().to(users::post_unfollow))
                    .route("/user/logout", web::get().to(users::get_logout))
                    .route("/user/search", web::post().to(users::post_search))
                    .route("/trip/new", web::post().to(trips::post_new))
                    .route("/trip/gpx", web::post().to(trips::post_gpx))
                    .route("/trip/update", web::post().to(trips::post_update))
                    .route("/trip/delete", web::post().to(trips::post_delete))
                    .route("/trip/join", web::post().to(trips::post_join))
                    .route("/trip/reconsider", web::post().to(trips::post_reconsider))
                    .route("/comment/new", web::post().to(comments::post_new))
                    .route(
                        "/validate/{username}/{token}",
                        web::get().to(users::get_validate),
                    ),
            )
            // static files
            .service(
                fs::Files::new("/web", "./web")
                    .use_etag(true)
                    .use_last_modified(true),
            )
            // user pages
            .service(web::resource("/cdn/{tail:.*}").route(web::get().to(cdn)))
            .service(web::resource("/{username}/trip/{uid}").route(web::get().to(trips::trip_page)))
            .service(web::resource("/{username}").route(web::get().to(user_page)))
            // default
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind(format!("{}:{}", server_url, server_port))?
    .run()
    .await
}

/// Static pages, under /-/*
async fn pages(
    tmpl: web::Data<tera::Tera>,
    pool: web::Data<Pool>,
    user_id: Identity,
    web::Path(page): web::Path<String>,
    session: Session,
) -> HttpResponse {
    let connection: &PgConnection = &pool.get().unwrap();
    let user = get_user(connection, &user_id.identity());

    let mut ctx = Context::new();
    ctx.insert("user", &user);
    ctx.insert("message", &consume_message(&session));

    let body = tmpl.render(&*format!("pages/{}.html", page), &ctx);

    match body {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => match e.kind {
            ErrorKind::TemplateNotFound(_) => p404(tmpl)
                .await
                .unwrap_or_else(|_| HttpResponse::NotFound().finish()),
            _ => HttpResponse::InternalServerError().body(format!("{:#?}", e)),
        },
    }
}

/// 404 handler
async fn p404(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse> {
    let body = tmpl
        .render("errors/404.html", &Context::new())
        .map_err(|e| ErrorInternalServerError(format!("{:#?}", e)))?;

    Ok(HttpResponse::NotFound()
        .content_type("text/html")
        .body(body))
}

// Tera filters
pub fn display_name(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("display_name", "value", User, value);
    let r = get_user_display_name(&s);
    Ok(to_value(&r).unwrap())
}

pub fn avatar_url(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("avatar_url", "value", User, value);
    Ok(to_value(format!("/cdn/avatars/{}.jpg", s.id)).unwrap())
}

pub fn date_rfc(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("date", "value", SystemTime, value);
    let date: DateTime<Utc> = s.into();
    let date = date.to_rfc3339();
    Ok(to_value(date).unwrap())
}

// CDN S3
async fn cdn(req: HttpRequest) -> HttpResponse {
    let path: String = req.match_info().get("tail").unwrap().parse().unwrap();
    let file = S3_CLIENT
        .get_object(GetObjectRequest {
            bucket: env::var("AWS_S3_BUCKET_NAME").expect("AWS_S3_BUCKET_NAME must be set"),
            key: path,
            ..Default::default()
        })
        .await;

    if let Ok(file) = file {
        HttpResponse::Ok()
            .header("Cache-Control", "max-age=7200") // 5 days
            .streaming(file.body.unwrap())
    } else {
        redirect_to("/web/assets/logos/icon-44.svg")
        // HttpResponse::NotFound().finish()
    }
}
