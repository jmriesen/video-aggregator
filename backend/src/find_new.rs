use crate::{Channel, Video};

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, PartialEq)]
struct Id {
    #[serde(rename = "videoId")]
    pub id: String,
}

#[derive(Deserialize, PartialEq)]
struct Snippet {
    #[serde(rename = "publishedAt")]
    pub published_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct Items {
    pub id: Id,
    pub snippet: Snippet,
}

#[derive(Deserialize)]
struct Response {
    pub items: Vec<Items>,
}

//Returns new (video, publication times) in reverse chronological order
pub async fn call(
    client: &reqwest::Client,
    channel: &Channel,
    published_after: Option<DateTime<Utc>>,
) -> impl Iterator<Item = (Video, DateTime<Utc>)> {
    let mut request = client
        .get("https://www.googleapis.com/youtube/v3/search")
        .query(&[
            ("part", "id,snippet"),
            ("order", "date"),
            ("maxResults", "20"),
            ("channelId", &channel.id),
            ("key", &std::fs::read_to_string("secrets/key.txt").unwrap()),
        ]);
    let published_after = published_after.map(|x| x + std::time::Duration::from_mins(1));
    if let Some(time_stamp) = published_after {
        request = request.query(&[("publishedAfter", time_stamp)]);
    }

    //Unwraps are fine sine this is a toy project
    let response = request.send().await.unwrap().text().await.unwrap();
    let response: Response = serde_json::from_str(&response).unwrap();
    response
        .items
        .into_iter()
        .map(|x| (Video(x.id.id), x.snippet.published_at))
}
