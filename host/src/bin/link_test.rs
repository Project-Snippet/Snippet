use std::error::Error;

use snippet_host::link::Link;
use snippet_protocol::state::State;
use snippet_protocol::{Request, Response, Time};

/// Send a request, print the exchange, and fail the run if the reply is wrong.
fn check(link: &mut Link<impl std::io::Read + std::io::Write>, request: Request, expected: Response)
    -> Result<(), Box<dyn Error>>
{
    let reply = link.send(request)?;
    println!("{request:?} -> {reply:?}");
    assert_eq!(reply, expected, "unexpected reply to {request:?}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/dev/ttyACM0".to_string());
    let mut link = Link::open(&path)?;

    let time = Time { hours: 14, minutes: 32, seconds: 5 };
    let bad_time = Time { hours: 99, minutes: 0, seconds: 0 };

    check(&mut link, Request::Ping, Response::Pong)?;

    // Time is stored, read back exactly, and bad times are refused.
    check(&mut link, Request::SetTime(time), Response::Ack)?;
    check(&mut link, Request::GetTime, Response::Time(time))?;
    check(&mut link, Request::SetTime(bad_time), Response::Rejected)?;
    check(&mut link, Request::GetTime, Response::Time(time))?;

    // States: jump to a known one, then step. Watch the LED bar as it runs.
    check(&mut link, Request::SetState(State::Default), Response::Ack)?;
    check(&mut link, Request::NextState, Response::Ack)?;
    check(&mut link, Request::GetState, Response::State(State::Clock))?;
    check(&mut link, Request::SetState(State::Cycle), Response::Ack)?;
    check(&mut link, Request::NextState, Response::Ack)?;
    check(&mut link, Request::GetState, Response::State(State::Default))?;

    println!("All checks passed.");
    Ok(())
}
