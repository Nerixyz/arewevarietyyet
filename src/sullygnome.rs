use anyhow::Result as AnyResult;
use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;

lazy_static! {
    static ref SULLYGNOME_CLIENT: Client = {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static(concat!(
                "arewevarietyyet/",
                env!("CARGO_PKG_VERSION"),
                " - github.com/nerixyz/arewevarietyyet"
            )),
        );

        Client::builder().default_headers(headers).build().unwrap()
    };
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct GamesResponse {
    pub data: Vec<GameData>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct GameData {
    pub streamtime: u64,
    pub gamesplayed: String,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct StreamsResponse {
    pub data: Vec<StreamData>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct StreamData {
    pub start_date_time: DateTime<Utc>,
    pub length: i64,
}

impl StreamData {
    pub fn end_date_time(&self) -> DateTime<Utc> {
        self.start_date_time + Duration::minutes(self.length)
    }

    pub fn duration_to(&self, other: &Self) -> Duration {
        if self.start_date_time > other.start_date_time {
            return other.duration_to(self);
        }
        // self <= other
        other.start_date_time - self.end_date_time()
    }

    pub fn duration_to_now(&self) -> Duration {
        Utc::now() - self.end_date_time()
    }
}

pub async fn get_games(year: i32) -> AnyResult<GamesResponse> {
    Ok(SULLYGNOME_CLIENT
        .get(format!("https://sullygnome.com/api/tables/channeltables/games/{year}/3505649/%20/1/2/desc/0/1000"))
        .send()
        .await?
        .json()
        .await?)
}

pub async fn get_streams(year: i32) -> AnyResult<StreamsResponse> {
    Ok(SULLYGNOME_CLIENT
        .get(format!("https://sullygnome.com/api/tables/channeltables/streams/{year}/3505649/%20/1/1/desc/0/1000"))
        .send()
        .await?
        .json()
        .await?)
}
