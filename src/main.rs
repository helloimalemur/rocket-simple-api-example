#[macro_use]
extern crate rocket;
use rocket::serde::{json::Json};
use std::collections::HashMap;
use std::net::SocketAddr;
mod entities;
mod manage_sessions;
use manage_sessions::*;
mod manage_users;
mod fairings;
use sqlx::{MySqlPool};
use config::Config;
use rocket::tokio::time::{interval_at, Instant};
use rocket::{custom, tokio};
use rocket::http::{Header, Status};
use rocket::request::{Request};
use rocket::{Response};
use rocket::fairing::{Fairing, Info, Kind};
use crate::entities::login::Login;
use crate::entities::users::User;
use crate::fairings::apikey_fairing::ApiKey;
use crate::manage_sessions::session_funcs::{get_username_by_session, verify_session_by_session_id};
use crate::manage_users::user_funcs::{create_new_user, login_user};
use log::{info};
use log::{LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::Config as LogConfig;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Logger, Root};


// // // // // // // // // // // // // // // // // // // // // // // //
// // // // // // // // // // // // // // // // // // // // // // // //

#[get("/")]
async fn index(
    socket_addr: SocketAddr,
    pool: &rocket::State<MySqlPool>
) -> &'static str {
    let is_pool_closed= pool.is_closed();
    info!(target:"app::requests", "ROOT PATH - From: {}", socket_addr.ip().to_string());
    if is_pool_closed {
        "No Swimming"
    } else {
        "Hello, Astronauts!"
    }
}


#[get("/api/<session_id>")]
async fn get_user(
    socket_addr: SocketAddr,
    session_id: String,
    pool: &rocket::State<MySqlPool>,
    _key: ApiKey<'_>
) -> String {
    let result = get_username_by_session(session_id, pool).await;
    info!(target:"app::requests", "GET USER via SESSION - From: {}, USER_RESULT: {}", socket_addr.ip().to_string(), result);
    result
}


#[post("/api/adduser", data = "<data>")]
async fn adduser(
    socket_addr: SocketAddr,
    pool: &rocket::State<MySqlPool>,
    data: Json<User>,
    // _key: ApiKey<'_>,
) -> Result<(), ErrorResponder> {
    let res = create_new_user(data.clone(), pool).await;
    info!(target:"app::requests", "ADD USER - From: {}, SUCCESS: {}, USER: {}", socket_addr.ip().to_string(), res, data.clone().username);
    Ok(())
}


#[post("/api/login", data = "<data>")]
async fn login(
    pool: &rocket::State<MySqlPool>,
    data: Json<Login>,
) -> Result<String, ErrorResponder> {
    let key = login_user(pool, data.clone()).await;
    let mut success = false;
    if key.len() >30 {
        success = true;
    }
    info!(target:"app::requests", "LOGIN REQUEST - From: {}, SUCCESS: {}, USER: {}", data.clone().ipaddress, success, data.clone().username);
    Ok(key)
}


#[get("/api/logout/<session_id>")]
async fn logout(
    socket_addr: SocketAddr,
    pool: &rocket::State<MySqlPool>,
    session_id: String
) -> Result<(), Status> {
    let mut success = false;
    let user = String::from("");
    println!("{user}");
    if verify_session_by_session_id( session_id.clone(), pool).await {
        session_funcs::delete_session_from_db_by_session_id(session_id.clone(), pool).await;
        success = true;
        println!("deleting sessions for {}", session_id.clone());
    }
    info!(target:"app::requests", "LOGOUT USER - From: {}, SUCCESS: {}, USER: {}", socket_addr.ip().to_string(), success, user);
    Ok(())
}


#[get("/api/verify/<session_id>")]
async fn verify_by_session(
    socket_addr: SocketAddr,
    session_id: String,
    pool: &rocket::State<MySqlPool>
) -> &'static str {
    let mut verified = "false";
    let mut success = false;
    let mut user = String::from("");
    if verify_session_by_session_id(session_id.clone(), pool).await {
        verified = "true";
        success = true;
        user = get_username_by_session(session_id.clone(),pool).await
    }
    info!(target:"app::requests", "VERIFY USER SESSION - From: {}, SUCCESS: {}, USER: {}, SESSION: {}", socket_addr.ip().to_string(), success, user, session_id.clone());
    verified
}


// // // // // // // // // // // // // // // // // // // // // // // //
// // // // // // // // // // // // // // // // // // // // // // // //


pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        // response.set_header(Header::new("Access-Control-Allow-Origin", "https://yourlinuxadmin.com/"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_status(Status::new(200));
    }
}


// // // // // // // // // // // // // // // // // // // // // // // //
// // // // // // // // // // // // // // // // // // // // // // // //


#[rocket::main]
pub async fn main() {
    // load configuration file
    let settings = Config::builder()
        .add_source(config::File::with_name("config/Settings"))
        .build()
        .unwrap();
    let settings_map = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let config = rocket::Config {
        port: 8030,
        address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
        ..rocket::Config::debug_default()
    };


    // setup logging request logging to file
    let stdout = ConsoleAppender::builder().build();
    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(settings_map.get("log_path").unwrap().as_str())
        .unwrap();
    #[allow(unused_variables)]
    let log_config = LogConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("requests", Box::new(requests)))
        // .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(Logger::builder().appender("requests").additive(true).build("app::requests", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    // logging to info
    info!(target: "app::requests","Starting");


    // set database_url string
    let database_url: &str = settings_map.get("database_url").unwrap().as_str();
    println!("{}", database_url.clone());


    // start re-occuring task
    tokio::spawn(async {
        let start = Instant::now();
        let mut interval = interval_at(start, tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
        }
    });

    // initialize database connection
    let pool = MySqlPool::connect(&database_url).await.expect("database connection");

    // launch Rocket
    custom(&config)
        .manage::<MySqlPool>(pool)
        .mount(
            "/",
            routes![
                index,
                adduser,
                login,
                logout,
                verify_by_session,
                get_user,
            ],
        )
        .attach(CORS)
        .launch()
        .await
        .unwrap();

}

// The following impl's are for easy conversion of error types.
#[derive(Responder)]
#[response(status = 500, content_type = "json")]
struct ErrorResponder {
    message: String,
}


impl From<anyhow::Error> for ErrorResponder {
    fn from(err: anyhow::Error) -> ErrorResponder {
        ErrorResponder {
            message: err.to_string(),
        }
    }
}


impl From<String> for ErrorResponder {
    fn from(string: String) -> ErrorResponder {
        ErrorResponder { message: string }
    }
}

impl From<&str> for ErrorResponder {
    fn from(str: &str) -> ErrorResponder {
        str.to_owned().into()
    }
}
