/// Red, a minimalistic ed-like text editor written in rust.
/// The goal is to replicate all relevant functions in ed and add some additional features to make it more usable.

mod io;
mod cmd;
mod buffer;

use buffer::{VecBuffer, Buffer};

// Runtime variables
pub struct State{
    // Configurations
    selection: Option<(usize, usize)>, // The start and end of selected lines
    prompt: Option<String>, // The string printed out when accepting commands
    file: Option<String>,// The one to write to by default
    // state variables
    done: bool, // Marks that it is time to exit
    error: Option<&'static str>, // Tracks the latest error
    stdin: std::io::Stdin, // The stdin is shared, to avoid conflicting opens
    buffer: VecBuffer, // The editing buffer
}
impl State {
    pub fn new() -> Self {
        Self {
            selection: None,
            prompt: None,
            file: None,
            done: false,
            error: None,
            stdin: std::io::stdin(),
            buffer: VecBuffer::new(),
        }
    }
}

fn main() {
    println!("Welcome to red, the rust-ed.");
    //println!("For assistance, enter '?'.");

    // Init command string
    let mut command = String::new();

    // Init state
    let mut state = State::new();

    // // Parse command line args
    // let mut i = 0;
    // for arg in std::env::args() {
    //     if i != 0 {
    //         match arg {
    //             filename => {
    //                 use std::io::{BufRead, BufReader};
    //                 // Open the file TODO, better error handling
    //                 let file =
    //                     std::fs::OpenOptions::new()
    //                     .read(true)
    //                     .write(true)
    //                     .create(true) // If the file is not found it is created
    //                     .open(filename)
    //                     .unwrap();
    //                 // A buffered reader is required to read line by line
    //                 let mut reader = BufReader::new(file);
    //                 // Loop reading the lines of the file into the buffer.
    //                 loop {
    //                     let mut line = String::new();
    //                     match reader.read_line(&mut line).unwrap() {
    //                         0 => break, // Means we have reached EOF
    //                         _ => state.buffer.push(line),
    //                     }
    //                 }
    //             },
    //         }
    //     }
    //     i +=1;
    // }

    // Loop until done. Take, identify and execute commands
    while !state.done {

        // Read command
        io::read_command(&mut state, &mut command);

        // Handle command
        match cmd::parse_and_run(&mut state, &mut command) {
            Ok(()) => {},
            Err(e) => {
                state.error = Some(e);
                println!("?");
            },
        }
    }
}
