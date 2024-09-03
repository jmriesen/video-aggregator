#![feature(duration_constructors)]
use std::str::FromStr;

use actix_web::{
    dev::Service,
    get,
    http::header::{HeaderName, HeaderValue},
    web, App, HttpServer, Responder,
};
use chrono::{DateTime, Utc};
use futures_util::future::FutureExt;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, PartialEq)]
struct Id {
    #[serde(rename = "videoId")]
    id: String,
}
#[derive(Deserialize, PartialEq)]
struct Snippet {
    #[serde(rename = "publishedAt")]
    published_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct Items {
    id: Id,
    snippet: Snippet,
}

#[derive(Deserialize)]
struct YouTubeResponse {
    items: Vec<Items>,
}
#[derive(Deserialize, Serialize)]
struct Record {
    channel: String,
    id: String,
    last_viewed: Option<chrono::DateTime<Utc>>,
}

async fn request_videos(client: &reqwest::Client, record: &Record) -> YouTubeResponse {
    let mut request = client
        .get("https://www.googleapis.com/youtube/v3/search")
        .query(&[
            ("part", "id,snippet"),
            ("order", "date"),
            ("maxResults", "20"),
            ("key", &std::fs::read_to_string("secrets/key.txt").unwrap()),
            ("channelId", &record.id),
        ]);
    if let Some(timestamp) = &record.last_viewed {
        request = request.query(&[(
            "publishedAfter",
            *timestamp + std::time::Duration::from_mins(1),
        )]);
    }

    let response = request.send().await.unwrap().text().await.unwrap();
    serde_json::from_str(&response).unwrap()
}

#[get("/videos")]
async fn videos() -> impl Responder {
    const RECORD_FILE: &'static str = "records/records.json";
    let mut records: Vec<Record> =
        serde_json::from_str(&std::fs::read_to_string(RECORD_FILE).unwrap()).unwrap();

    let client = reqwest::Client::new();

    //NOTE Async iterators are nightly only
    //Consider replacing once Async iters are stable.
    let mut responses = vec![];
    for record in &mut records {
        let response = request_videos(&client, record).await;
        //pull out the published time of the most recent video
        let most_resent_video_time = response.items.first().map(|item| item.snippet.published_at);
        if most_resent_video_time.is_some() {
            record.last_viewed = most_resent_video_time;
        }
        responses.push(response);
    }
    std::fs::write(RECORD_FILE, serde_json::to_string_pretty(&records).unwrap()).unwrap();

    let ids = responses
        .iter()
        .map(|x| x.items.iter())
        .flatten()
        .map(|x| x.id.id.clone())
        .collect::<Vec<String>>();

    web::Json(ids)
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
                println!("Hi from start. You requested: {}", req.path());
                srv.call(req).map(|res| {
                    println!("Hi from response");
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
