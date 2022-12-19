use crate::{model::Year, sullygnome::StreamData};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use serde::Serialize;
use std::ops::Add;

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LongestDitch {
    Current {
        from: DateTime<Utc>,
    },
    Past {
        from: DateTime<Utc>,
        duration: String,
    },
}

pub fn count(streams: &[StreamData]) -> usize {
    streams
        .iter()
        .fold(
            (0, Utc::now().date_naive().add(Duration::days(1))),
            |(count, last), item| {
                if last == item.start_date_time.date_naive() {
                    (count, last)
                } else {
                    (count + 1, item.start_date_time.date_naive())
                }
            },
        )
        .0
}

fn first_day_in_year(year: i32) -> DateTime<Utc> {
    Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(year, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    )
}

fn first_time_in_day(time_in_day: DateTime<Utc>) -> DateTime<Utc> {
    Utc.from_utc_datetime(&time_in_day.date_naive().and_hms_opt(0, 0, 0).unwrap())
}

pub fn days_in_current_year() -> usize {
    let today = first_time_in_day(Utc::now());
    let tomorrow = today.add(Duration::days(1));
    let start = first_day_in_year(today.year());
    if start < tomorrow {
        (tomorrow - start).num_days() as usize
    } else {
        0
    }
}

pub fn days_in_last_year() -> usize {
    let year = Utc::now().year();
    let start = first_day_in_year(year - 1);
    let end = first_day_in_year(year);
    (end - start).num_days() as usize
}

impl LongestDitch {
    fn current(from: DateTime<Utc>) -> Self {
        Self::Current { from }
    }
    fn past(ditch: &[StreamData]) -> Self {
        Self::Past {
            from: ditch[1].end_date_time(),
            duration: humantime::format_duration(
                ditch[1]
                    .duration_to(&ditch[0])
                    .to_std()
                    .unwrap_or(std::time::Duration::from_secs(0)),
            )
            .to_string(),
        }
    }

    pub fn calculate(year: Year, streams: &[StreamData]) -> Self {
        let old_ditch = streams.windows(2).reduce(|accum, item| {
            if accum[1].duration_to(&accum[0]) > item[1].duration_to(&item[0]) {
                accum
            } else {
                item
            }
        });
        match year {
            Year::Current => {
                // it's sorted from newest to oldest
                let last = match streams.first() {
                    Some(last) => last,
                    None => return Self::current(Utc::now()),
                };

                match old_ditch {
                    Some(old_ditch)
                        if last.duration_to_now() < old_ditch[1].duration_to(&old_ditch[0]) =>
                    {
                        Self::past(old_ditch)
                    }
                    _ => Self::current(last.end_date_time()),
                }
            }
            Year::Last => match old_ditch {
                Some(old_ditch) => Self::past(old_ditch),
                // This shouldn't really happen, as there should be at least one stream.
                None => Self::Past {
                    from: first_day_in_year(Utc::now().year() - 1),
                    duration: "0".to_owned(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{streamcounter::count, sullygnome::StreamData};

    #[test]
    fn it_works() {
        assert_eq!(
            count(&[
                StreamData {
                    start_date_time: "2022-01-04T14:08:05Z".parse().unwrap(),
                    length: 0
                },
                StreamData {
                    start_date_time: "2022-01-02T14:08:05Z".parse().unwrap(),
                    length: 0,
                },
                StreamData {
                    start_date_time: "2022-01-01T23:08:05Z".parse().unwrap(),
                    length: 0,
                },
                StreamData {
                    start_date_time: "2022-01-01T14:08:05Z".parse().unwrap(),
                    length: 0
                },
            ]),
            3
        );
    }
}
