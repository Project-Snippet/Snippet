use super::State;
use crate::states::state4::State4;

pub struct State3;

impl State for State3 {
    fn next(self: Box<Self>) -> Box<dyn State> {
        Box::new(State4)
    }

    fn print_state(&self) {
        println!("Current state: State3");
    }
}
