#![no_std]

#[cfg(test)]
extern crate std;

pub mod device;
pub mod framer;
pub mod state;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use state::State;

pub use postcard::Error;

/// Longest encoded message (including the COBS framing) we expect to send.
pub const MAX_FRAME: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

impl Time {
    /// A 24-hour clock time: hours 0-23, minutes and seconds 0-59.
    pub fn is_valid(&self) -> bool {
        self.hours < 24 && self.minutes < 60 && self.seconds < 60
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    /// A real calendar date between 2000 and 2099.
    pub fn is_valid(&self) -> bool {
        (2000..=2099).contains(&self.year)
            && (1..=12).contains(&self.month)
            && self.day >= 1
            && self.day <= days_in_month(self.year, self.month)
    }
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 31,
    }
}

/// Messages the host sends to the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    Ping,
    NextState,
    SetState(State),
    GetState,
    SetTime(Time),
    GetTime,
    SetDate(Date),
    GetDate,
}

/// Messages the device sends back to the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Response {
    Pong,
    Ack,
    /// The request was understood but its contents were not acceptable.
    Rejected,
    State(State),
    Time(Time),
    Date(Date),
}

/// Serialize `value` into `buf` as one COBS frame (it ends with a 0x00 byte).
/// Returns the slice of `buf` that holds the frame.
pub fn encode<'a, T: Serialize>(value: &T, buf: &'a mut [u8]) -> Result<&'a mut [u8], Error> {
    postcard::to_slice_cobs(value, buf)
}

/// Deserialize one frame. The frame is modified in place while decoding.
pub fn decode<T: DeserializeOwned>(frame: &mut [u8]) -> Result<T, Error> {
    postcard::from_bytes_cobs(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(value: T)
    where
        T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug,
    {
        let mut buf = [0u8; MAX_FRAME];
        let frame = encode(&value, &mut buf).unwrap();
        assert_eq!(frame.last(), Some(&0), "frame must end with 0x00");
        assert_eq!(decode::<T>(frame).unwrap(), value);
    }

    #[test]
    fn every_request_survives_a_round_trip() {
        let time = Time { hours: 23, minutes: 59, seconds: 7 };
        round_trip(Request::Ping);
        round_trip(Request::NextState);
        round_trip(Request::SetState(State::Media));
        round_trip(Request::GetState);
        round_trip(Request::SetTime(time));
        round_trip(Request::GetTime);
        round_trip(Request::SetDate(Date { year: 2026, month: 10, day: 4 }));
        round_trip(Request::GetDate);
    }

    #[test]
    fn every_response_survives_a_round_trip() {
        round_trip(Response::Pong);
        round_trip(Response::Ack);
        round_trip(Response::Rejected);
        round_trip(Response::Date(Date { year: 2099, month: 12, day: 31 }));
        round_trip(Response::State(State::Cycle));
        round_trip(Response::Time(Time { hours: 0, minutes: 0, seconds: 0 }));
    }

    #[test]
    fn no_zero_bytes_inside_a_frame() {
        let mut buf = [0u8; MAX_FRAME];
        let frame = encode(&Request::SetTime(Time { hours: 0, minutes: 0, seconds: 0 }), &mut buf).unwrap();
        let (last, body) = frame.split_last().unwrap();
        assert_eq!(*last, 0);
        assert!(body.iter().all(|&b| b != 0));
    }

    #[test]
    fn garbage_is_an_error_not_a_crash() {
        let mut garbage = [0xFF, 0xFF, 0xFF, 0x00];
        assert!(decode::<Request>(&mut garbage).is_err());
        let mut empty = [0x00];
        assert!(decode::<Request>(&mut empty).is_err());
    }

    #[test]
    fn dates_are_checked_against_the_calendar() {
        let date = |year, month, day| Date { year, month, day };
        assert!(date(2026, 10, 4).is_valid());
        assert!(date(2024, 2, 29).is_valid(), "2024 is a leap year");
        assert!(date(2000, 2, 29).is_valid(), "2000 is a leap year");
        assert!(!date(2100, 2, 29).is_valid(), "2100 is out of range");
        assert!(!date(2026, 2, 29).is_valid(), "2026 is not a leap year");
        assert!(!date(2026, 4, 31).is_valid());
        assert!(!date(2026, 13, 1).is_valid());
        assert!(!date(2026, 0, 1).is_valid());
        assert!(!date(2026, 1, 0).is_valid());
        assert!(!date(1999, 12, 31).is_valid());
    }
}
