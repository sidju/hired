// Import the highlighting theme
const THEME: &[u8] = include_bytes!("../assets/theme.xml");

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
  
  let path = args.path;

  // Create buffer and use command line input to init it
  let mut buffer = add_ed::buffer::VecBuffer::new();

  // Then we construct our UI
  let mut ui = hui::HighlightingUI::new();
  crossterm::terminal::enable_raw_mode().expect("Failed to configure terminal.");

  // Then start up the editor
  let mut ed = add_ed::Ed::new(&mut buffer, path).expect("Failed to open file.");
  ed.run(&mut ui).unwrap();

  // Clear out raw mode before closing
  crossterm::terminal::disable_raw_mode()
    .expect("Failed to clear raw mode. Run 'reset' to fix terminal.");
}
