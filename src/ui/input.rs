// This module takes events and handles them as text input or commands

// Since unicode is weird and this method is missing from str
// Finds the nearest char boundary preceeding given index and returns its index
fn rfind_boundary(s: &str, i: usize) -> usize {
  for b in (0 .. i).rev() {
    if s.is_char_boundary(b) { return b; }
  }
  0
}

// This input getter runs get_event and buffers the input until a lone . appears on a line.
// Then it returns that buffer.
pub fn get_input(state: &mut crate::State)
  -> Result<Vec<String>, crossterm::ErrorKind>
{
  // To be placed in State
  let mut out = std::io::stdout();

  // The Write trait, to flush after each print
  use std::io::Write;
  // The trait for queueing commands
  use crossterm::QueueableCommand;
  // And print command
  use crossterm::style::Print;
  // Move cursor commands
  use crossterm::cursor::{MoveLeft, MoveRight};
  // All the event classes
  use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, Event};

  // Create the buffer we store the input in
  let mut buffer = Vec::new();
  buffer.push(String::new());
  // Add some index variables we'll need
  let mut lindex = 0; // Line index, lin-dex
  let mut chindex = 0; // Char index, ch-index
  let mut char_size = 0; // The nr of bytes the current char is
  let mut ret = false; // Flag when ready to return

  // loop getting input events, ignoring those we can't handle.
  while !ret {
    match crossterm::event::read()? {
      Event::Mouse(_) => (),
      Event::Resize(x, y) => {
        state.term_size = (x as usize, y as usize);
      },
      // Match the code. Then check for modifiers separately for each key?
      Event::Key(key) => match key.code {
        KeyCode::Char(ch) => {
          buffer[lindex].insert(chindex, ch);
          chindex += ch.len_utf8();
          char_size += ch.len_utf8();
          // Verify that we are at a valid char-boundary before printing, to support modifiers
          if buffer[lindex].is_char_boundary(chindex) {
            out.queue(Print(&buffer[lindex][chindex - char_size .. ]))?;
            char_size = 0;
            out.flush()?;
          }
        },
        KeyCode::Backspace => {
          // First we need to find the nearest preceeding character boundary
          let b = rfind_boundary(&buffer[lindex], chindex);
          chindex = b;
          buffer[lindex].remove(chindex);
          // Only move left if we printed the last char
          if char_size == 0 {
            out.queue(MoveLeft(1))?;
            // Print out all the string behind, to shift it leftwards
            out.queue(Print(&buffer[lindex][chindex .. ]))?;
            // And overwrite the last character, since we will be one char short
            out.queue(Print(' '))?;
            // TODO: handle moving up if we wrapped from the print above
            out.queue(MoveLeft((buffer[lindex].len() - chindex) as u16))?;
            out.flush()?;
          }
        },
        KeyCode::Enter => {
          buffer[lindex].insert(chindex, '\n');
          chindex += 1;
          out.queue(Print("\n\r"))?; // Newline and carriage return, since in raw mode
          out.flush()?;
          if buffer[lindex].trim() == "." {
            // We got the end line. Delete this line and return
            buffer.remove(lindex);
            ret = true;
          }
          else {
            // Else we add a new line after current and go to it
            lindex += 1;
            chindex = 0;
            buffer.insert(lindex, String::new());
          }
        }
        _ => (), // Ignore unknown codes
      }
    }
  }
  Ok(buffer)
}

// The core UI loop, handles events and runs the resulting commands on newlines
pub fn core_loop(state: &mut crate::State) -> Result<(), &'static str> {
  // All relevant data is stored in the State struct, so just get looping
  loop {

  }
}
