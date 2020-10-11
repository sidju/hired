// This module takes events and handles them as text input or commands

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
  use crossterm::event::{KeyEvent, KeyCode, Event};

  // Create the buffer we store the input in
  let mut buffer = Vec::new();
  buffer.push(String::new());
  // Add some index variables we'll need
  let mut lindex = 0; // Line index, lin-dex
  let mut chindex = 0; // Char index, ch-index
  let mut ret = false; // Flag when ready to return

  // loop getting input events, ignoring those we can't handle.
  while !ret {
    match crossterm::event::read()? {
      Event::Mouse(_) => (),
      Event::Resize(x, y) => {
        state.term_size = (x as usize, y as usize);
      },
      // currently only handle non-modified keys
      Event::Key(key) => if key.modifiers.is_empty() {
        match key.code {
          KeyCode::Char(ch) => {
            buffer[lindex].insert(chindex, ch);
            chindex += 1;
            out.queue(Print(ch))?;
            out.flush()?;
          },
          KeyCode::Backspace => {
            chindex -= 1;
            buffer[lindex].remove(chindex);
            out.queue(MoveLeft(1))?;
            out.queue(Print(' '))?;
            out.queue(MoveLeft(1))?;
            out.flush()?;
          },
          KeyCode::Enter => {
            buffer[lindex].insert(chindex, '\n');
            chindex += 1;
            out.queue(Print("\n\r"))?; // Newline and carriage return, since in raw mode
            out.flush()?;
            if buffer[lindex] == ".\n" {
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
  }
  Ok(buffer)
}

// The core UI loop, handles events and runs the resulting commands on newlines
pub fn core_loop(state: &mut crate::State) -> Result<(), &'static str> {
  // All relevant data is stored in the State struct, so just get looping
  loop {

  }
}
