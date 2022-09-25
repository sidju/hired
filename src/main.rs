// Import the highlighting theme
const THEME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/compressed_theme"));
const SYNTAXES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/compressed_syntaxes"));

// All UI abtractions
mod hui;

use clap::Parser;
use std::collections::HashMap;

/// hired, the highlighting EDitor
#[derive(Parser)]
#[clap(version, about)]
struct Args {
  /// path to the file to open
  #[clap(value_parser, default_value = "")]
  path: String,
  /// default to printing with line numbers
  #[clap(action, short)]
  n: bool,
  /// default to printing in literal mode
  #[clap(action, short)]
  l: bool,
}

pub fn main() {
  // We start by handling command line input
  let args = Args::parse();
  
  // Use the parsed input to configure UI and editor
  let path = args.path;

  // Construct editor
  let mut buffer = add_ed::buffer::VecBuffer::new();
  let mut ui = hui::HighlightingUI::new();
  // Empty hash map, since we have no support for reading in macro config yet
  let mut ed = add_ed::Ed::new(&mut buffer, path, HashMap::new(), args.n, args.l).expect("Failed to open file.");

  // Start raw mode after opening file, to not use .expect() when in raw mode
  crossterm::terminal::enable_raw_mode().expect("Failed to configure terminal.");

  // Run the editor, saving result
  let res = ed.run(&mut ui);

  // Clear out raw mode before reacting to result
  crossterm::terminal::disable_raw_mode()
    .expect("Failed to clear raw mode. Run 'reset' to fix terminal.");

  res.unwrap();
}
