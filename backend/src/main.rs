use std::str::FromStr;

use actix_web::{
    dev::Service,
    get,
    http::header::{HeaderName, HeaderValue},
    web, App, HttpServer, Responder,
};
use chrono::Utc;
use futures_util::future::FutureExt;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, PartialEq)]
struct Id {
    #[serde(rename = "videoId")]
    id: String,
}

#[derive(Deserialize)]
struct Items {
    id: Id,
}

#[derive(Deserialize)]
struct YouTubeResponse {
    items: Vec<Items>,
}
#[derive(Deserialize, Serialize)]
struct Record {
    channel: String,
    id: String,
    last_viewed: Option<chrono::Datetime<Utc>>,
}
//("channelId", "UC0YvoAYGgdOfySQSLcxtu1w"),

async fn request_videos(client: &reqwest::Client, record: &Record) -> YouTubeResponse {
    let response = client
        .get("https://www.googleapis.com/youtube/v3/search")
        .query(&[
            ("part", "id,snippet"),
            ("order", "date"),
            ("maxResults", "20"),
            ("key", &std::fs::read_to_string("secrets/key.txt").unwrap()),
            ("channelId", ""),
            ("publishedAfter", "2024-08-29T02:00:08Z"),
        ])
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    serde_json::from_str(&response).unwrap()
}

#[get("/videos")]
async fn videos() -> impl Responder {
    let records: Vec<Record> =
        serde_json::from_str(&std::fs::read_to_string("records.json").unwrap()).unwrap();

    let client = reqwest::Client::new();

    //let responce = std::fs::read_to_string("response.json").unwrap();
    let response: YouTubeResponse = serde_json::from_str(&response).unwrap();
    let ids = response.items.into_iter().map(|x| x.id.id);
    web::Json(ids.collect::<Vec<_>>())
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
