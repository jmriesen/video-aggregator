use std::collections::HashMap;

use crate::{Channel, Video};

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

#[derive(Deserialize, PartialEq)]
struct ContentDetails {
    //I decided not to actually parse the stirng since simple string comparisons
    //are sufficient for my filtering.
    pub duration: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Items {
    pub id: String,
    pub content_details: ContentDetails,
}

#[derive(Deserialize)]
struct Response {
    pub items: Vec<Items>,
}

pub async fn call(client: &reqwest::Client, videos: &[Video]) -> impl Iterator<Item = Video> {
    let request = client
        .get("https://www.googleapis.com/youtube/v3/videos")
        .query(&[
            ("part", "id,contentDetails"),
            (
                "id",
                &videos
                    .iter()
                    .map(|x| x.0.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            ("key", &std::fs::read_to_string("secrets/key.txt").unwrap()),
        ]);

    //Unwraps are fine sine this is a toy project
    let response = request.send().await.unwrap().text().await.unwrap();
    let response: Response = serde_json::from_str(&response).unwrap();
    response
        .items
        .into_iter()
        .map(|x| (x.id, x.content_details.duration))
        // Include if 1 minute < duration <= 1 hour.
        // See google API for format documentation
        // [[https://developers.google.com/youtube/v3/docs/videos]]
        .filter(|(_, duration)| {
            //No hour component && minute component but not exactly 1 minute
            !duration.contains("H") && duration.contains("M") && duration != "PT1M"
        })
        .map(|(id, _)| Video(id))
}
