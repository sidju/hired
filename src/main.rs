
/// Red, a minimalistic ed-like text editor written in rust.
/// The goal is to replicate all relevant functions in ed and add some additional features to make it more usable.

use syntect::parsing::SyntaxSet;
use syntect::highlighting::Theme;

pub mod error_consts;
use error_consts::*;

mod cmd;
mod buffer;
mod file;
mod ui;

use buffer::VecBuffer;
use buffer::Buffer;

// Store theme as a constant
const THEME: &[u8] = include_bytes!("../assets/theme.xml");

// Runtime variables
pub struct State {
  // Configurations
  prompt: String, // The string printed out when accepting commands
  print_errors: bool,
  // Connection to and data about the terminal
  stdout: std::io::Stdout, // the terminal interaction handle for the UI
  term_size: (usize,usize),
  // Runtime data, errors and done
  done: bool, // Marks that it is time to exit
  error: Option<&'static str>, // Tracks the latest error
  // Buffer and our position in it
  file: String,// The one to write to by default
  selection: Option<(usize, usize)>, // The start and end of selected lines
  buffer: VecBuffer, // The editing buffer
  // Data for syntax highlighting
  syntax_lib: SyntaxSet,
  theme: Theme,
}
impl State {
  pub fn new() -> Self {
    let mut theme_reader = std::io::Cursor::new(&THEME[..]);
    let theme = syntect::highlighting::ThemeSet::load_from_reader(&mut theme_reader).unwrap();
    Self {
      prompt: ":".to_string(),
      print_errors: true,

      stdout: std::io::stdout(),
      term_size: crossterm::terminal::size().map(|(a,b)| (a as usize, b as usize)).unwrap_or((80,24)),

      done: false,
      error: None,

      file: String::new(),
      selection: None,
      buffer: VecBuffer::new(),

      syntax_lib: SyntaxSet::load_defaults_newlines(),
      theme: theme,
    }
  }
}

fn main() {
  println!("Welcome to hired. Ed with highlighting written in rust.");
  println!("For assistance, enter '?'.");

  // Init state
  let mut state = State::new();

  // TODO: Add handling of custom config and custom themes!!!

  // Use the terminal in raw mode during the core loop
  crossterm::terminal::enable_raw_mode().expect("Failed to open terminal in raw mode");

  // Read in and handle command line arguments
  let mut first = true;
  for arg in std::env::args() {
    if !first {
      match arg.chars().next() {
        // Eventually handle command line flags
        //Some('-') => 
        // TODO: Make something less horrifying to handle errors here
        _ => match (||->Result<(),&str> {
          let mut data = file::read_file(&arg, false)?;
          let datalen = data.len();
          state.buffer.insert(&mut data, 0)?;
          state.buffer.set_saved();
          state.file = arg;
          state.selection = Some((0, datalen));
          Ok(())
        })() {
          Ok(_) => {},
          Err(e) => { println!("{}\n\r", e); },
        },
      }
    }
    first = false;
  }

  // Loop until done. Take, identify and execute commands
  while !state.done {

    // Read command
    let res = ui::get_command(&mut state)
      // Run command
      .and_then(|mut cmd| cmd::run(&mut state, &mut cmd))
    ;

    // Handle result
    match res {
      Ok(()) => {},
      Err(e) => {
        // Include for printing
        use std::io::Write;
        use crossterm::{QueueableCommand, style::Print};

        state.error = Some(e);
        if state.print_errors {
          state.stdout.queue(Print(e)).expect(TERMINAL_WRITE);
        }
        else {
          state.stdout.queue(Print("?\n\r")).expect(TERMINAL_WRITE);
        }
        state.stdout.flush().expect(TERMINAL_WRITE);
      },
    }
  }

  crossterm::terminal::disable_raw_mode().expect(DISABLE_RAWMODE);
}
