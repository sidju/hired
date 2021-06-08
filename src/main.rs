// Import the highlighting theme
const THEME: &[u8] = include_bytes!("../assets/theme.xml");

// All UI abtractions
mod hui;

pub fn main() {
  // We start by handling command line input
  // TODO: Handle command line input
  let path = "".to_string();

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
