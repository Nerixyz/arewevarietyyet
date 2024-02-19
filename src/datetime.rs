use std::ops::Add;

use chrono::{DateTime, Datelike, Days, Duration, NaiveDate, TimeZone, Utc};

pub fn first_day_in_year(year: i32) -> DateTime<Utc> {
    Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(year, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    )
}

pub fn first_time_in_day(time_in_day: DateTime<Utc>) -> DateTime<Utc> {
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

pub fn days_in_year(year: i32) -> usize {
    let start = first_day_in_year(year);
    let end = first_day_in_year(year + 1);
    (end - start).num_days() as usize
}

pub fn end_of_day(time_in_day: DateTime<Utc>) -> DateTime<Utc> {
    Utc.from_utc_datetime(
        &time_in_day
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    )
}
