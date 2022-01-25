use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use crate::sullygnome;
use crate::sullygnome::{GameData, GamesResponse};

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new("^([^|]+)\\|(:?[^|]+)\\|(.+)$").unwrap();
}

#[derive(Serialize, Debug)]
pub struct StreamerModel {
    pub games: Vec<GameModel>
}

impl TryFrom<sullygnome::GamesResponse> for StreamerModel {
    type Error = anyhow::Error;

    fn try_from(value: GamesResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            games: value.data.try_into()?,
        })
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameModel {
    pub time_streamed_sec: u64,
    pub category: String,
    pub category_image: String,
}

impl TryFrom<sullygnome::GameData> for GameModel {
    type Error = anyhow::Error;

    fn try_from(value: GameData) -> Result<Self, Self::Error> {
        let (category, category_image) = extract_category_and_url(&value.gamesplayed).ok_or_else(|| anyhow!("bad games"))?;
        Ok(Self {
            category,
            category_image,
            time_streamed_sec: value.streamtime,
        })
    }
}

fn extract_category_and_url(gamesplayed: &str) -> Option<(String, String)> {
    let captures = GAME_REGEX.captures_iter(gamesplayed).next()?;
    let mut matches = captures.iter().skip(1);
    let game = matches.next()??;
    let game_url = matches.next()??;
    Some((game.as_str().to_string(), game_url.as_str().to_string()))
}