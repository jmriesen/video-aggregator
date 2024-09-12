#![feature(duration_constructors)]
mod file_backed;
mod find_new;

use std::str::FromStr;

use actix_web::{
    dev::Service,
    get,
    http::header::{HeaderName, HeaderValue},
    web, App, HttpServer, Responder,
};
use chrono::{DateTime, Utc};
use futures_util::future::FutureExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Record {
    channel: Channel,
    last_viewed: Option<DateTime<Utc>>,
}

//video string internal Id.
#[derive(Deserialize, Serialize)]
struct Video(String);

#[derive(Deserialize, Serialize)]
struct Channel {
    id: String,
    name: String,
}

impl Record {
    async fn get_new_videos(&mut self, client: &Client) -> impl Iterator<Item = Video> {
        let mut channels_videos = find_new::call(&client, &self.channel, self.last_viewed).await;

        //pull out the published time of the most recent video
        let newest_video = channels_videos.next();
        if let Some((_, time_stamp)) = &newest_video {
            self.last_viewed = Some(*time_stamp);
        }
        newest_video.into_iter().chain(channels_videos).map(|x| x.0)
    }
}

#[get("/videos")]
async fn videos() -> impl Responder {
    const RECORD_FILE: &'static str = "records/records.json";
    let mut records =
        file_backed::FileBacked::<Vec<Record>>::new(&std::path::Path::new(RECORD_FILE));

    let client = reqwest::Client::new();

    let mut channel_videos = vec![];
    for record in records.as_mut().iter_mut() {
        channel_videos.extend(record.get_new_videos(&client).await)
    }

    web::Json(channel_videos)
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    res.map(|mut x| {
                        x.headers_mut().append(
                            HeaderName::from_str("Access-Control-Allow-Origin").unwrap(),
                            HeaderValue::from_static("*"),
                        );
                        x.headers_mut().append(
                            HeaderName::from_str("Access-Control-Allow-Methods").unwrap(),
                            HeaderValue::from_static("GET, OPTIONS"),
                        );
                        x.headers_mut().append(
                            HeaderName::from_str("Access-Control-Allow-Credentials").unwrap(),
                            HeaderValue::from_static("true"),
                        );
                        x
                    })
                })
            })
            //res.headers_mut().append("Access-Control-Allow-Origin", "*");
            .service(videos)
            .service(hello)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
