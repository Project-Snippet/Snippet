use super::State;
use crate::states::state1::State1;

pub struct State5;

impl State for State5 {
    fn next(self: Box<Self>) -> Box<dyn State> {
        Box::new(State1)
    }

    fn print_state(&self) {
        println!("Current state: State5");
    }
}
