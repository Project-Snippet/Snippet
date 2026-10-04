use std::io::{self, Write};
mod states;

use states::state1::State1;
use states::State;

fn main() {
    let mut current_state: Box<dyn State> = Box::new(State1);

    loop {
        current_state.print_state();

        print!("Press 's' to switch to the next state or 'q' to quit: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "s" => {
                current_state = current_state.next();
            }
            "q" => {
                println!("Exiting the program");
                break;
            }
            _ => {
                println!("Invalid input. Please follow the instructions.");
            }
        }
    }
}
