use anyhow::Result as AnyResult;
use chrono::{DateTime, Datelike, Duration, Utc};
use futures::future;
use lazy_static::lazy_static;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;

use crate::datetime::end_of_day;

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
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GamesResponse {
    pub records_total: i32,
    pub data: Vec<GameData>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct GameData {
    pub streamtime: u64,
    pub gamesplayed: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StreamsResponse {
    pub records_total: i32,
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

    pub fn day_iter(&self) -> StreamDayIter {
        StreamDayIter {
            start_date_time: self.start_date_time,
            end_date_time: self.end_date_time(),
        }
    }
}

pub struct StreamDayIter {
    start_date_time: DateTime<Utc>,
    end_date_time: DateTime<Utc>,
}

impl Iterator for StreamDayIter {
    type Item = (u32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_date_time >= self.end_date_time {
            return None;
        }

        let (day, delta) = if self.start_date_time.date_naive() == self.end_date_time.date_naive() {
            let delta = self.end_date_time - self.start_date_time;
            self.start_date_time = self.end_date_time; // to return None next time
            (self.start_date_time.ordinal0(), delta)
        } else {
            let next_start = end_of_day(self.start_date_time);
            let delta = next_start - self.start_date_time;
            let start = self.start_date_time.ordinal0();
            self.start_date_time = next_start;
            (start, delta)
        };

        Some((day, (delta.num_minutes() as f32) / 60.0))
    }
}

pub trait SullyResource: Sized {
    type Item;

    fn get_it(year: i32, offset: i32) -> impl std::future::Future<Output = AnyResult<Self>>;

    fn records(&self) -> i32;
    fn extend(&mut self, it: impl Iterator<Item = Self::Item>);
    fn into_data(self) -> Vec<Self::Item>;
}

impl SullyResource for StreamsResponse {
    type Item = StreamData;

    fn get_it(year: i32, offset: i32) -> impl std::future::Future<Output = AnyResult<Self>> {
        get_streams(year, offset)
    }

    fn records(&self) -> i32 {
        self.records_total
    }

    fn extend(&mut self, it: impl Iterator<Item = Self::Item>) {
        self.data.extend(it);
    }

    fn into_data(self) -> Vec<Self::Item> {
        self.data
    }
}

impl SullyResource for GamesResponse {
    type Item = GameData;

    fn get_it(year: i32, offset: i32) -> impl std::future::Future<Output = AnyResult<Self>> {
        get_games(year, offset)
    }

    fn records(&self) -> i32 {
        self.records_total
    }

    fn extend(&mut self, it: impl Iterator<Item = Self::Item>) {
        self.data.extend(it);
    }

    fn into_data(self) -> Vec<Self::Item> {
        self.data
    }
}

pub async fn get_all_of<T: SullyResource>(year: i32) -> AnyResult<T> {
    let mut base = T::get_it(year, 0).await?;
    if base.records() > 100 {
        // (x + 99) / 100 is basically .div_ceil but that's unstable :(
        let f = (1..((base.records() + 99) / 100)).map(|n| T::get_it(year, n * 100));
        base.extend(
            future::join_all(f)
                .await
                .into_iter()
                .filter_map(Result::ok)
                .flat_map(T::into_data),
        );
    }

    Ok(base)
}

pub async fn get_games(year: i32, offset: i32) -> AnyResult<GamesResponse> {
    Ok(SULLYGNOME_CLIENT
        .get(format!("https://sullygnome.com/api/tables/channeltables/games/{year}/3505649/%20/1/2/desc/{offset}/100"))
        .send()
        .await?
        .json()
        .await?)
}

pub async fn get_streams(year: i32, offset: i32) -> AnyResult<StreamsResponse> {
    Ok(SULLYGNOME_CLIENT
        .get(format!("https://sullygnome.com/api/tables/channeltables/streams/{year}/3505649/%20/1/1/desc/{offset}/100"))
        .send()
        .await?
        .json()
        .await?)
}
