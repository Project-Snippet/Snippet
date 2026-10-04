use super::State;
use crate::states::state5::State5;

pub struct State4;

impl State for State4 {
    fn next(self: Box<Self>) -> Box<dyn State> {
        Box::new(State5)
    }

    fn print_state(&self) {
        println!("Current state: State4");
    }
}
