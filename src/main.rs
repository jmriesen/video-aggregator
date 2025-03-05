#![feature(duration_constructors)]
mod file_backed;
mod find_new;
mod length_filter;

use std::str::FromStr;

use actix_web::{
    dev::Service,
    get,
    http::header::{HeaderName, HeaderValue},
    web, App, HttpServer, Responder,
};
use chrono::{DateTime, Utc};
use futures_util::future::FutureExt;
use handlebars::{DirectorySourceOptions, Handlebars};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
async fn get_videos() -> Vec<Video> {
    const RECORD_FILE: &'static str = "records/records.json";
    let mut records =
        file_backed::FileBacked::<Vec<Record>>::new(&std::path::Path::new(RECORD_FILE));

    let client = reqwest::Client::new();

    let mut channel_videos = vec![];
    for record in records.as_mut().iter_mut() {
        let all_videos = record.get_new_videos(&client).await;
        let filtered_videos = length_filter::call(&client, &all_videos.collect::<Vec<_>>()).await;
        channel_videos.extend(filtered_videos)
    }
    channel_videos
}

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let video_list = get_videos().await;
    let links: Vec<_> = video_list
        .iter()
        .map(|x| format!("https://www.youtube.com/embed/{}?autoplay=0&mute=0", x.0))
        .collect();
    let links = json!({
    "videos":links
    });
    let body = hb.render("index", &links).unwrap();

    web::Html::new(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(
            "./templates",
            DirectorySourceOptions {
                tpl_extension: ".html".to_owned(),
                hidden: false,
                temporary: false,
            },
        )
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);
    HttpServer::new(move || {
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
            .app_data(handlebars_ref.clone())
            //res.headers_mut().append("Access-Control-Allow-Origin", "*");
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
