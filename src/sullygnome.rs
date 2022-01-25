use lazy_static::lazy_static;
use reqwest::Client;
use reqwest::header::{HeaderMap, self, HeaderValue};
use serde::Deserialize;
use anyhow::Result as AnyResult;

lazy_static! {
    static ref SULLYGNOME_CLIENT: Client = {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, HeaderValue::from_static("arewevarietyyet/0.1"));

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

const REQUEST_URL: &str = "https://sullygnome.com/api/tables/channeltables/games/2022/3505649/%20/1/2/desc/0/1000";

pub async fn request() -> AnyResult<GamesResponse> {
    Ok(SULLYGNOME_CLIENT.get(REQUEST_URL).send().await?.json()?)
}

pub fn