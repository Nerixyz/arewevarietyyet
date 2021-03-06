use crate::{streamcounter, streamcounter::LongestDitch, sullygnome};
use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new("^([^|]+)\\|(?:[^|]+)\\|(.+)$").unwrap();
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamerModel {
    pub games: Vec<GameModel>,
    pub total_time_min: u64,
    pub variety_percent: f64,
    pub ow_percent: f64,
    pub are_we_variety: bool,

    pub days_ditched: usize,
    pub days_until_now: usize,
    pub percent_ditched: f64,

    pub longest_ditch: streamcounter::LongestDitch,
}

impl TryFrom<(sullygnome::GamesResponse, sullygnome::StreamsResponse)> for StreamerModel {
    type Error = anyhow::Error;

    fn try_from(
        (games, streams): (sullygnome::GamesResponse, sullygnome::StreamsResponse),
    ) -> Result<Self, Self::Error> {
        let (total_time_sec, ow_time) = games.data.iter().fold((0, 0), |(total, ow), item| {
            (
                total + item.streamtime,
                ow + if item.gamesplayed.starts_with("Overwatch") {
                    item.streamtime
                } else {
                    0
                },
            )
        });
        let ow_percent = ow_time as f64 / total_time_sec as f64;
        let variety_percent = 1.0 - ow_percent;

        let days_until_now = streamcounter::days_in_year();
        let days_streamed = streamcounter::count(&streams.data);
        let days_ditched = days_until_now - days_streamed;
        Ok(Self {
            games: games
                .data
                .into_iter()
                .map(GameModel::try_from)
                .collect::<Result<Vec<_>, Self::Error>>()?,
            total_time_min: total_time_sec,
            ow_percent,
            variety_percent,
            are_we_variety: variety_percent >= 0.3,

            days_ditched,
            days_until_now,
            percent_ditched: (days_ditched as f64) / (days_until_now as f64),
            longest_ditch: LongestDitch::calculate(&streams.data),
        })
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameModel {
    pub time_streamed_min: u64,
    pub category: String,
    pub category_image: String,
}

impl TryFrom<sullygnome::GameData> for GameModel {
    type Error = anyhow::Error;

    fn try_from(value: sullygnome::GameData) -> Result<Self, Self::Error> {
        let (category, category_image) =
            extract_category_and_url(&value.gamesplayed).ok_or_else(|| anyhow!("bad games"))?;
        Ok(Self {
            category,
            category_image,
            time_streamed_min: value.streamtime,
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
