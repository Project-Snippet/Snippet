use serde::{Deserialize, Serialize};

/// The display modes of Snippet. The device owns the current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Default,
    Clock,
    Media,
    Upcoming,
    Dayview,
    System,
    ToDo,
    Cycle,
}

impl State {
    /// The state that follows this one. After `Cycle` it wraps to `Default`.
    pub fn next(self) -> State {
        match self {
            State::Default => State::Clock,
            State::Clock => State::Media,
            State::Media => State::Upcoming,
            State::Upcoming => State::Dayview,
            State::Dayview => State::System,
            State::System => State::ToDo,
            State::ToDo => State::Cycle,
            State::Cycle => State::Default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_moves_through_states_in_order() {
        assert_eq!(State::Default.next(), State::Clock);
        assert_eq!(State::ToDo.next(), State::Cycle);
    }

    #[test]
    fn eight_nexts_return_to_start() {
        let mut state = State::Default;
        for _ in 0..8 {
            state = state.next();
        }
        assert_eq!(state, State::Default);
    }
}
