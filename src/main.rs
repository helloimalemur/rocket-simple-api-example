#[macro_use]
extern crate rocket;
use rocket::serde::json::Json;
use std::collections::HashMap;
use std::net::SocketAddr;
mod fairings;
use crate::fairings::apikey_fairing::ApiKey;
use config::Config;
use log::info;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config as LogConfig;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::tokio::time::{interval_at, Instant};
use rocket::Response;
use rocket::{custom, tokio};
use serde_json::Value;
use sqlx::MySqlPool;

// // // // // // // // // // // // // // // // // // // // // // // //
// // // // // // // // // // // // // // // // // // // // // // // //

#[get("/")]
async fn index(socket_addr: SocketAddr) -> &'static str {
    info!(target:"app::requests", "ROOT PATH - From: {}", socket_addr.ip().to_string());
    "Hello, Astronauts!"
}

#[get("/api/<data>")]
async fn getdata(
    socket_addr: SocketAddr,
    data: String,
    key: ApiKey<'_>,
    settings: &rocket::State<HashMap<String, String>>,
) {
    let secret = settings.get("api_key").unwrap();
    if key.to_string().eq(secret) {
        println!("{}\n{}", key.to_string(), data);
    }

    info!(target:"app::requests", "GET - From: {}", socket_addr.ip().to_string());
}

#[post("/api/senddata", data = "<data>")]
async fn senddata(
    socket_addr: SocketAddr,
    data: Json<Value>,
    key: ApiKey<'_>,
    settings: &rocket::State<HashMap<String, String>>,
) -> Result<(), ErrorResponder> {
    let secret = settings.get("api_key").unwrap();
    if key.to_string().eq(secret) {
        println!("{}\n{:?}", key.to_string(), data);
    }

    info!(target:"app::requests", "From: {}, SUCCESS: {}, USER: {}", socket_addr.ip().to_string(), res, data.clone().username);
    Ok(())
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
        .logger(
            Logger::builder()
                .appender("requests")
                .additive(true)
                .build("app::requests", LevelFilter::Info),
        )
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    // logging to info
    info!(target: "app::requests","Starting");

    // start re-occuring task
    tokio::spawn(async {
        let start = Instant::now();
        let mut interval = interval_at(start, tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
        }
    });

    // launch Rocket
    custom(&config)
        .manage(settings_map.clone())
        .mount("/", routes![index, getdata, senddata,])
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
