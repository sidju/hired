mod config;
use config::construct_config;
mod macro_store;

// All UI abstractions
mod hui;
use hui::error::HighlightingUIError as HUIError;

use add_ed::ui::UI;

pub fn main() {
  // Parse CLI arguments, env and config file into a run configuration
  // (This will abort execution in a lot of cases, so it must be ran before
  // enabling raw mode)
  let config = construct_config();
  
  // Construct editor
  let mut ui = hui::HighlightingUI::new();
  let mut io = add_ed::io::LocalIO::new();
  // Create our macro store
  let macro_store = macro_store::MacroStore{
    config_macros: &config.macros,
  };
  let mut ed = add_ed::Ed::new(&mut io, &macro_store);
  ed.n = config.n;
  ed.l = config.l;

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

    let res = ed.run_command(&mut ui, &format!("e{}", config.path));
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
