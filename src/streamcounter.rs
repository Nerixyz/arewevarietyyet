use crate::sullygnome::StreamData;
use chrono::{DateTime, Datelike, Duration, Utc};
use std::ops::Add;

pub fn count(streams: &[StreamData]) -> usize {
    let start = "2021-12-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap();
    streams
        .iter()
        .fold((0, &start), |(count, last), item| {
            if last.day() == item.start_date_time.day()
                && last.month() == item.start_date_time.month()
            {
                (count, last)
            } else {
                (count + 1, &item.start_date_time)
            }
        })
        .0
}

pub fn days_in_year() -> usize {
    let start = "2022-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
    // count this day as well
    (Utc::now().add(Duration::days(1)) - start).num_days() as usize
}

#[cfg(test)]
mod tests {
    use crate::{streamcounter::count, sullygnome::StreamData};

    #[test]
    fn it_works() {
        assert_eq!(
            count(&[
                StreamData {
                    start_date_time: "2022-01-04T14:08:05Z".parse().unwrap()
                },
                StreamData {
                    start_date_time: "2022-01-02T14:08:05Z".parse().unwrap()
                },
                StreamData {
                    start_date_time: "2022-01-01T23:08:05Z".parse().unwrap()
                },
                StreamData {
                    start_date_time: "2022-01-01T14:08:05Z".parse().unwrap()
                },
            ]),
            3
        );
    }
}
