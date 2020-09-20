
/// Red, a minimalistic ed-like text editor written in rust.
/// The goal is to replicate all relevant functions in ed and add some additional features to make it more usable.

#[macro_use]
extern crate derivative;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;


pub mod error_consts;

mod io;
mod cmd;
mod buffer;
mod file;

mod ui;

use buffer::VecBuffer;
use buffer::Buffer;

// Runtime variables
#[derive(Derivative)]
#[derivative(Debug)]
pub struct State {
    // Configurations
    prompt: String, // The string printed out when accepting commands
    print_errors: bool,
    // state variables
    selection: Option<(usize, usize)>, // The start and end of selected lines
    file: String,// The one to write to by default
    done: bool, // Marks that it is time to exit
    error: Option<&'static str>, // Tracks the latest error
    stdin: std::io::Stdin, // The stdin is shared, to avoid conflicting opens
    term_size: (usize,usize),
    #[derivative(Debug="ignore")]
    buffer: VecBuffer, // The editing buffer
    #[derivative(Debug="ignore")]
    syntax_lib: SyntaxSet,
    #[derivative(Debug="ignore")]
    theme_lib: ThemeSet,
}
impl State {
    pub fn new() -> Self {
        Self {
            prompt: String::new(),
            print_errors: true,

            selection: None,
            file: String::new(),
            done: false,
            error: None,
            term_size: crossterm::terminal::size().map(|(a,b)| (a as usize, b as usize)).unwrap_or((80,24)),

            stdin: std::io::stdin(),
            buffer: VecBuffer::new(),
            syntax_lib: SyntaxSet::load_defaults_newlines(),
            theme_lib: ThemeSet::load_defaults(),
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

    // TODO: Add handling of custom config and custom themes!!!

    // Read in and handle command line arguments
    let mut first = true;
    for arg in std::env::args() {
        if !first {
            match arg.chars().next() {
                // Eventually handle command line flags
                //Some('-') => 
                // TODO: Make something less horrifying to handle errors here
                _ => match (||->Result<(),&str> {
                  let mut data = file::read_file(&arg)?;
                  let datalen = data.len();
                  state.buffer.insert(&mut data, 0)?;
                  state.buffer.set_saved();
                  state.file = arg;
                  state.selection = Some((0, datalen));
                  Ok(())
                })() {
                  Ok(_) => {},
                  Err(e) => { println!("{}", e); }
                },
            }
        }
        first = false;
    }

    // Loop until done. Take, identify and execute commands
    while !state.done {

        // Read command
        io::read_command(&mut state, &mut command);

        // Update the terminal size before running commands
        state.term_size = crossterm::terminal::size().map(|(a,b)| (a as usize, b as usize)).unwrap_or((80,24));

        // Handle command
        match cmd::run(&mut state, &mut command) {
            Ok(()) => {},
            Err(e) => {
                state.error = Some(e);
                if state.print_errors {
                  println!("{}", e);
                }
                else {
                  println!("?");
                }
            },
        }
    }
}
