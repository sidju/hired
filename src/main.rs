// Import the highlighting theme
const THEME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/compressed_theme"));
const SYNTAXES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/compressed_syntaxes"));

// All UI abtractions
mod hui;

use add_ed::ui::UI;
use add_ed::error_consts::{
  DISABLE_RAWMODE,
  TERMINAL_WRITE,
};
use clap::Parser;

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
  
  // Construct editor
  let mut buffer = add_ed::buffer::Buffer::new();
  let mut ui = hui::HighlightingUI::new();
  let mut io = add_ed::io::LocalIO::new();
  let mut ed = add_ed::Ed::new(&mut buffer, &mut io, args.path);
  ed.n = args.n;
  ed.l = args.l;

  // Start raw mode before using HighlightingUI
  // Avoid using .unwrap(), .expect() or panic!() when in raw mode, as it leaves
  // the terminal in an unusable state for bash.
  crossterm::terminal::enable_raw_mode().expect("Failed to configure terminal.");

  // Handle if hired is started not on column 0 (for example git may do this)
  if crossterm::cursor::position().expect(TERMINAL_WRITE).0 != 0 {
    print!("\n\r");
  }

  // Before normal execution, run a command to open the given path
  let res = ed.run_command(&mut ui, "e");
  // If we failed to open file for some reason, print that error
  if let Err(e) = res {
    let res2 = ui.print_message( if ed.print_errors { e } else { "?\n" } );
    // If we cannot print, clear raw mode and panic
    if let Err(_) = res2 {
      crossterm::terminal::disable_raw_mode()
        .expect(DISABLE_RAWMODE);
      res2.unwrap();
    }
  }

  // Run the editor, saving result
  let res = ed.run(&mut ui);

  // Clear out raw mode before reacting to result
  crossterm::terminal::disable_raw_mode()
    .expect(DISABLE_RAWMODE);

  res.unwrap();
}
