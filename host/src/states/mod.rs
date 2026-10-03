pub trait State {
    fn next(self: Box<Self>) -> Box<dyn State>;
    fn print_state(&self);
}

pub mod state1;
pub mod state2;
pub mod state3;
pub mod state4;
pub mod state5;
