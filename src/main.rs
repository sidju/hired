// Import the highlighting theme
const THEME: &[u8] = include_bytes!("../assets/compressed_theme");
const SYNTAXES: &[u8] = include_bytes!("../assets/compressed_syntaxes");

// All UI abtractions
mod hui;

use argh::FromArgs;

#[derive(FromArgs)]
/// hired, the highlighting EDitor
struct Args {
  /// path to the file to open
  #[argh(positional, default = "String::new()")]
  path: String,
}

pub fn main() {
  // We start by handling command line input
  let args: Args = argh::from_env();
  
  // Use the parsed input to configure UI and editor
  let path = args.path;

  // Construct editor
  let mut buffer = add_ed::buffer::VecBuffer::new();
  let mut ui = hui::HighlightingUI::new();
  let mut ed = add_ed::Ed::new(&mut buffer, path).expect("Failed to open file.");

  // Start raw mode after opening file, to not use .expect() when in raw mode
  crossterm::terminal::enable_raw_mode().expect("Failed to configure terminal.");

  // Run the editor, saving result
  let res = ed.run(&mut ui);

  // Clear out raw mode before reacting to result
  crossterm::terminal::disable_raw_mode()
    .expect("Failed to clear raw mode. Run 'reset' to fix terminal.");

  res.unwrap();
}
