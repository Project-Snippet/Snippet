use super::State;
use crate::states::state3::State3;

pub struct State2;

impl State for State2 {
    fn next(self: Box<Self>) -> Box<dyn State> {
        Box::new(State3)
    }

    fn print_state(&self) {
        println!("Current state: State2");
    }
}
