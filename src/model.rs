use crate::{streamcounter, streamcounter::LongestDitch, sullygnome};
use anyhow::{anyhow, Result};
use chrono::{Datelike, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new("^([^|]+)\\|(?:[^|]+)\\|(.+)$").unwrap();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Year {
    Current,
    Last,
}

impl Year {
    pub fn days_till_today(&self) -> usize {
        match self {
            Year::Current => streamcounter::days_in_current_year(),
            Year::Last => streamcounter::days_in_last_year(),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamerModel {
    pub games: Vec<GameModel>,
    pub total_time_min: u64,
    pub at_least_one_stream: bool,
    pub variety_percent: f64,
    pub ow_percent: f64,
    pub are_we_variety: bool,

    pub days_ditched: usize,
    pub days_until_now: usize,
    pub percent_ditched: f64,

    pub year: i32,

    pub longest_ditch: LongestDitch,
}

impl StreamerModel {
    pub fn create(
        year: Year,
        games: sullygnome::GamesResponse,
        streams: sullygnome::StreamsResponse,
    ) -> Result<Self> {
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
        let mut ow_percent = ow_time as f64 / total_time_sec as f64;
        let mut variety_percent = 1.0 - ow_percent;
        if ow_percent.is_nan() {
            ow_percent = 0.0;
            variety_percent = 0.0;
        }

        let days_until_now = year.days_till_today();
        let days_streamed = streamcounter::count(&streams.data);
        let days_ditched = days_until_now - days_streamed;
        let mut percent_ditched = days_ditched as f64 / days_until_now as f64;
        if percent_ditched.is_nan() {
            percent_ditched = 1.0;
        }

        Ok(Self {
            games: games
                .data
                .into_iter()
                .map(GameModel::try_from)
                .collect::<Result<Vec<_>>>()?,
            total_time_min: total_time_sec,
            at_least_one_stream: total_time_sec > 0,
            ow_percent,
            variety_percent,
            are_we_variety: variety_percent >= 0.3,

            days_ditched,
            days_until_now,
            percent_ditched,

            year: match year {
                Year::Current => Utc::now().year(),
                Year::Last => Utc::now().year() - 1,
            },

            longest_ditch: LongestDitch::calculate(year, &streams.data),
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
