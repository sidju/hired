/// Red, a minimalistic ed-like text editor written in rust.
/// The goal is to replicate all relevant functions in ed and add some additional features to make it more usable.

mod io;
mod cmd;

// Runtime variables
pub struct State{
    // Configurations
    selection: Option<(usize, usize)>, // The start and end of selected lines
    prompt: Option<String>, // The string printed out when accepting commands
    // state variables
    done: bool, // Marks that it is time to exit
    error: Option<String>, // Tracks the latest error
    stdin: std::io::Stdin, // The stdin is shared, to avoid conflicting opens
    buffer: Vec<String>, // The editing buffer
}
impl State {
    pub fn new() -> Self {
        Self {
            selection: None,
            prompt: None,
            done: false,
            error: None,
            stdin: std::io::stdin(),
            buffer: Vec::new(),
        }
    }
}

fn main() {
    // TODO: take command line input, such as filename

    println!("Welcome to red, the rust-ed.");
    //println!("For assistance, enter '?'.");

    // Init state
    let mut state = State::new();

    // Store command strings separately
    let mut command = String::new();

    // Loop until done. Take, identify and execute commands
    while !state.done {

        // Read command
        io::read_command(&mut state, &mut command);

        // Handle command
        match cmd::handle_command(&mut state, &mut command) {
            Ok(()) => {},
            Err(e) => {
                state.error = Some(e);
                println!("?");
            },
        }
    }
}
