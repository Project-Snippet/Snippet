use crate::state::State;
use crate::{Request, Response, Time};

/// Everything the device remembers. The firmware owns one of these and
/// feeds it every request it receives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device {
    pub state: State,
    pub time: Time,
}

impl Device {
    pub fn new() -> Self {
        Device {
            state: State::Default,
            time: Time { hours: 0, minutes: 0, seconds: 0 },
        }
    }

    /// Apply one request and return the reply to send back.
    pub fn handle(&mut self, request: Request) -> Response {
        match request {
            Request::Ping => Response::Pong,
            Request::NextState => {
                self.state = self.state.next();
                Response::Ack
            }
            Request::SetState(state) => {
                self.state = state;
                Response::Ack
            }
            Request::GetState => Response::State(self.state),
            Request::SetTime(time) if time.is_valid() => {
                self.time = time;
                Response::Ack
            }
            Request::SetTime(_) => Response::Rejected,
            Request::GetTime => Response::Time(self.time),
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_gets_pong() {
        assert_eq!(Device::new().handle(Request::Ping), Response::Pong);
    }

    #[test]
    fn starts_in_default_state() {
        assert_eq!(Device::new().handle(Request::GetState), Response::State(State::Default));
    }

    #[test]
    fn next_state_advances_and_acks() {
        let mut device = Device::new();
        assert_eq!(device.handle(Request::NextState), Response::Ack);
        assert_eq!(device.handle(Request::GetState), Response::State(State::Clock));
    }

    #[test]
    fn set_state_jumps_straight_to_the_state() {
        let mut device = Device::new();
        device.handle(Request::SetState(State::System));
        assert_eq!(device.handle(Request::GetState), Response::State(State::System));
    }

    #[test]
    fn stored_time_is_returned_exactly() {
        let mut device = Device::new();
        let time = Time { hours: 14, minutes: 32, seconds: 5 };
        assert_eq!(device.handle(Request::SetTime(time)), Response::Ack);
        assert_eq!(device.handle(Request::GetTime), Response::Time(time));
    }

    #[test]
    fn invalid_time_is_rejected_and_old_time_kept() {
        let mut device = Device::new();
        let good = Time { hours: 10, minutes: 0, seconds: 0 };
        device.handle(Request::SetTime(good));

        for bad in [
            Time { hours: 24, minutes: 0, seconds: 0 },
            Time { hours: 0, minutes: 60, seconds: 0 },
            Time { hours: 0, minutes: 0, seconds: 60 },
            Time { hours: 255, minutes: 255, seconds: 255 },
        ] {
            assert_eq!(device.handle(Request::SetTime(bad)), Response::Rejected);
        }
        assert_eq!(device.handle(Request::GetTime), Response::Time(good));
    }
}
