
/// Red, a minimalistic ed-like text editor written in rust.
/// The goal is to replicate all relevant functions in ed and add some additional features to make it more usable.

#[macro_use]
extern crate derivative;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::Theme;

pub mod error_consts;

mod io;
mod cmd;
mod buffer;
mod file;

mod ui;

use buffer::VecBuffer;
use buffer::Buffer;

// Store theme as a constant
const THEME: &[u8] = include_bytes!("../assets/theme.xml");

// Runtime variables
#[derive(Derivative)]
#[derivative(Debug)]
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
  #[derivative(Debug="ignore")]
  buffer: VecBuffer, // The editing buffer
  // Data for syntax highlighting
  #[derivative(Debug="ignore")]
  syntax_lib: SyntaxSet,
  #[derivative(Debug="ignore")]
  theme: Theme,
}
impl State {
  pub fn new() -> Self {
    let mut theme_reader = std::io::Cursor::new(&THEME[..]);
    let theme = syntect::highlighting::ThemeSet::load_from_reader(&mut theme_reader).unwrap();
    Self {
      prompt: String::new(),
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
  //println!("For assistance, enter '?'.");

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

  // Use the terminal in raw mode during the core loop
  crossterm::terminal::enable_raw_mode().expect("Failed to open terminal in raw mode");

  // Loop until done. Take, identify and execute commands
  while !state.done {

    // Read command
    let command = ui::get_command(&mut state);

    // Handle command
    match command.and_then(|mut cmd| cmd::run(&mut state, &mut cmd)) {
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

  crossterm::terminal::disable_raw_mode().expect("Failed to clean raw mode before closing. Either restart terminal or run 'reset'. Good luck!");
}
