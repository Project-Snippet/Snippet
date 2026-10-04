use chrono::{Datelike, Local, NaiveDateTime, Timelike};
use snippet_protocol::{Date, Time};

/// Split a calendar date-time into the protocol's `Date` and `Time`.
pub fn split(dt: NaiveDateTime) -> (Date, Time) {
    let date = Date {
        // A year outside u16 becomes 0, which the device rejects as invalid.
        year: u16::try_from(dt.year()).unwrap_or(0),
        month: dt.month() as u8,
        day: dt.day() as u8,
    };
    let time = Time {
        hours: dt.hour() as u8,
        minutes: dt.minute() as u8,
        seconds: dt.second() as u8,
    };
    (date, time)
}

/// The PC's current local date and time.
pub fn now() -> (Date, Time) {
    split(Local::now().naive_local())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn splits_a_date_time_into_protocol_types() {
        let dt = NaiveDate::from_ymd_opt(2026, 10, 4)
            .unwrap()
            .and_hms_opt(14, 32, 5)
            .unwrap();
        let (date, time) = split(dt);
        assert_eq!(date, Date { year: 2026, month: 10, day: 4 });
        assert_eq!(time, Time { hours: 14, minutes: 32, seconds: 5 });
    }

    #[test]
    fn what_the_clock_produces_is_always_valid() {
        let (date, time) = now();
        assert!(date.is_valid());
        assert!(time.is_valid());
    }

    #[test]
    fn last_second_of_a_leap_day_is_valid() {
        let dt = NaiveDate::from_ymd_opt(2028, 2, 29)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let (date, time) = split(dt);
        assert!(date.is_valid() && time.is_valid());
    }
}
