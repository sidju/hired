// Import the highlighting theme
const THEME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/compressed_theme"));
const SYNTAXES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/compressed_syntaxes"));

// All UI abtractions
mod hui;
use hui::error::HighlightingUIError as HUIError;

use add_ed::ui::UI;
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
  let mut ui = hui::HighlightingUI::new();
  let mut io = add_ed::io::LocalIO::new();
  // Create a temporary macro store, a proper one TBD
  let macro_store = std::collections::HashMap::new();
  let mut ed = add_ed::Ed::new(&mut io, &macro_store);
  ed.n = args.n;
  ed.l = args.l;

  // Start raw mode before using HighlightingUI
  // Avoid using .unwrap(), .expect() or panic!() when in raw mode, as it leaves
  // the terminal in an unusable state for bash.
  crossterm::terminal::enable_raw_mode()
    .map_err(HUIError::RawmodeSwitchFailed)
    .unwrap()
  ;

  // Run the editor, saving result
  let res = (|| -> Result<(), add_ed::error::EdError>{
    // Handle if hired is started not on column 0 (for example git may do this)
    // (Doesn't require raw mode to run, but enters and leaves rawmode if not.)
    let pos = crossterm::cursor::position()
      .map_err(HUIError::TerminalIOFailed)
      .unwrap()
    ;
    if pos.0 != 0 { print!("\n\r"); }

    let res = ed.run_command(&mut ui, &format!("e{}", args.path));
    if let Err(e) = res {
      ui.print_message(&format!("{}", e))?;
    }
    ed.run(&mut ui)?;
    Ok(())
  })();
  // Clear out raw mode before reacting to result
  crossterm::terminal::disable_raw_mode()
    .map_err(HUIError::RawmodeSwitchFailed)
    .unwrap();
  // Panic if we exit because of a fatal error
  res.unwrap();
}
