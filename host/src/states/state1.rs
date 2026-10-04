use super::State;
use crate::states::state2::State2;

pub struct State1;

impl State for State1 {
    fn next(self: Box<Self>) -> Box<dyn State> {
        Box::new(State2)
    }

    fn print_state(&self) {
        println!("Current state: State1");
    }
}
