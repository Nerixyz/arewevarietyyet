use anyhow::Result as AnyResult;
use chrono::{DateTime, Utc};
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
            HeaderValue::from_static("arewevarietyyet/0.1 - github.com/nerixyz/arewevarietyyet"),
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
}

const GAMES_REQUEST_URL: &str =
    "https://sullygnome.com/api/tables/channeltables/games/2022/3505649/%20/1/2/desc/0/1000";

const STREAMS_REQUEST_URL: &str =
    "https://sullygnome.com/api/tables/channeltables/streams/2022/3505649/%20/1/1/desc/0/1000";

pub async fn get_games() -> AnyResult<GamesResponse> {
    Ok(SULLYGNOME_CLIENT
        .get(GAMES_REQUEST_URL)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn get_streams() -> AnyResult<StreamsResponse> {
    Ok(SULLYGNOME_CLIENT
        .get(STREAMS_REQUEST_URL)
        .send()
        .await?
        .json()
        .await?)
}
