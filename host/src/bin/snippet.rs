use std::error::Error;
use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use snippet_host::{clock, link::Link};
use snippet_protocol::{Request, Response};

enum Command {
    NextState,
    Quit,
}

/// Read typed lines on a background thread, so waiting for a keypress never
/// delays the clock.
fn spawn_input_thread() -> mpsc::Receiver<Command> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for line in io::stdin().lock().lines().map_while(Result::ok) {
            let command = match line.trim() {
                "s" => Command::NextState,
                "q" => Command::Quit,
                other => {
                    println!("\nUnknown input {other:?}. Type 's' for next state, 'q' to quit.");
                    continue;
                }
            };
            let quit = matches!(command, Command::Quit);
            if tx.send(command).is_err() || quit {
                break;
            }
        }
    });
    rx
}

/// Send a request that should be answered with `Ack`.
fn send_expecting_ack(link: &mut Link<impl io::Read + io::Write>, request: Request) -> Result<(), Box<dyn Error>> {
    match link.send(request)? {
        Response::Ack => Ok(()),
        other => Err(format!("device answered {request:?} with {other:?}").into()),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let default_port = if cfg!(windows) { "COM3" } else { "/dev/ttyACM0" };
    let path = std::env::args().nth(1).unwrap_or_else(|| default_port.to_string());
    let mut link = Link::open(&path)?;
    let commands = spawn_input_thread();

    println!("Connected to {path}. Type 's' + Enter for next state, 'q' + Enter to quit.");
    loop {
        let (date, time) = clock::now();
        send_expecting_ack(&mut link, Request::SetDate(date))?;
        send_expecting_ack(&mut link, Request::SetTime(time))?;
        print!(
            "\r{:04}-{:02}-{:02} {:02}:{:02}:{:02}  ",
            date.year, date.month, date.day, time.hours, time.minutes, time.seconds
        );
        io::stdout().flush()?;

        match commands.recv_timeout(Duration::from_secs(1)) {
            Ok(Command::NextState) => {
                send_expecting_ack(&mut link, Request::NextState)?;
                if let Response::State(state) = link.send(Request::GetState)? {
                    println!("\nState is now {state:?}");
                }
            }
            Ok(Command::Quit) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
    println!("\nBye.");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\nerror: {e}");
        std::process::exit(1);
    }
}
