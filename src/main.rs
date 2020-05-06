
/// Red, a minimalistic ed-like text editor written in rust.
/// The goal is to replicate all relevant functions in ed and add some additional features to make it more usable.

#[macro_use]
extern crate derivative;

mod io;
mod cmd;
mod buffer;
mod file;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

use buffer::{VecBuffer, Buffer};


// Runtime variables
#[derive(Derivative)]
#[derivative(Debug)]
pub struct State {
    // Configurations
    selection: Option<(usize, usize)>, // The start and end of selected lines
    prompt: Option<String>, // The string printed out when accepting commands
    file: Option<String>,// The one to write to by default
    #[derivative(Debug="ignore")]
    syntax_lib: SyntaxSet,
    #[derivative(Debug="ignore")]
    theme_lib: ThemeSet,
    print_errors: bool,
    // state variables
    done: bool, // Marks that it is time to exit
    error: Option<&'static str>, // Tracks the latest error
    stdin: std::io::Stdin, // The stdin is shared, to avoid conflicting opens
    #[derivative(Debug="ignore")]
    buffer: VecBuffer, // The editing buffer
}
impl State {
    pub fn new() -> Self {
        Self {
            selection: None,
            prompt: None,
            file: None,
            syntax_lib: SyntaxSet::load_defaults_newlines(),
            theme_lib: ThemeSet::load_defaults(),
            print_errors: false,
            done: false,
            error: None,
            stdin: std::io::stdin(),
            buffer: VecBuffer::new(),
        }
    }
}

fn main() {
    println!("Welcome to hired. Ed with highlighting written in rust.");
    println!("Use the h flag on your print commands to enable highlighting.");
    //println!("For assistance, enter '?'.");

    // Init command string
    let mut command = String::new();

    // Init state
    let mut state = State::new();

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
