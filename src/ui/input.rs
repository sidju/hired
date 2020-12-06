// This module takes events and handles them as text input or commands

// TODO: We need more checks against common errors, like moving past or deleting the newline on a line. Also, cleanup.

  // The Write trait, to flush after each print
  use std::io::Write;
  // The trait for queueing commands
  use crossterm::QueueableCommand;
  // And print command
  use crossterm::style::Print;
  // Move cursor commands
  use crossterm::cursor::{MoveLeft, MoveRight, SavePosition, RestorePosition};
  // All the event classes
  use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, Event};

// Since unicode is weird and this method is missing from str
// Finds the nearest char boundary preceeding given index and returns its index
fn rfind_boundary(s: &str, i: usize) -> usize {
  for b in (0 .. i).rev() {
    if s.is_char_boundary(b) { return b; }
  }
  0
}
fn find_boundary(s: &str, i: usize) -> usize {
  for b in i + 1 .. s.len() + 1 {
    if s.is_char_boundary(b) { return b; }
  }
  i
}

// This input getter runs get_event and buffers the input until a lone . appears on a line
// Then it returns that buffer.
pub fn get_input(state: &mut crate::State)
  -> Result<Vec<String>, crossterm::ErrorKind>
{
  // To be placed in State
  let mut out = std::io::stdout();

  // Set the cursor to be visible, so our moves are visible
  out.queue(crossterm::cursor::Show)?;

  // Create the buffer we store the input in
  let mut buffer = Vec::new();
  buffer.push("\n".to_string());
  // Add some index variables we'll need
  let mut lindex = 0; // Line index, lin-dex
  let mut chindex = 0; // Char index, ch-index
  let mut partial = String::with_capacity(4); // To store partial chars
  let mut ret = false; // Flag when ready to return

  // For saving the cursor height on screen between printing rounds
  let mut dists: (u16, u16) = (0,0); // For cursor to top and bottom of print, respectively

  // loop getting input events, ignoring those we can't handle.
  while !ret {
    match crossterm::event::read()? {
      Event::Mouse(_) => (),
      Event::Resize(x, y) => {
        state.term_size = (x as usize, y as usize);
      },
      // Match the code. Then check for modifiers separately for each key?
      Event::Key(key) => match (key.code, key.modifiers) {
        (KeyCode::Char(ch), KeyModifiers::SHIFT) | (KeyCode::Char(ch), KeyModifiers::NONE) => {
          partial.push(ch);
          // If the partial is now complete, put it in the buffer
          if partial.is_char_boundary(0) {
            let tmp = chindex;
            chindex += partial.len();
            buffer[lindex].insert(tmp, partial.remove(0));
          }
        },
        (KeyCode::Left, KeyModifiers::NONE) => {
          partial.clear();
          if chindex == 0 {
            // Go to previous line
          }
          else {
            chindex = rfind_boundary(&buffer[lindex], chindex);
          }
        },
        (KeyCode::Right, KeyModifiers::NONE) => {
          partial.clear();
          if chindex == buffer[lindex].len() - 1 {
            // Go to next line
          }
          else {
            chindex = find_boundary(
              &buffer[lindex][.. buffer[lindex].len() - 1],
              chindex
            );
          }
        },
        (KeyCode::Backspace, KeyModifiers::NONE) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
          partial.clear();
          if chindex == 0 {
            // Join this and preceeding line
          }
          else {
            // Just delete preceeding character
            chindex = rfind_boundary(&buffer[lindex], chindex);
            buffer[lindex].remove(chindex);
          }
        },
        (KeyCode::Enter, KeyModifiers::NONE) => {
          partial.clear();
          // Insert the newline
          buffer[lindex].insert(chindex, '\n');
          chindex += 1;
          // Split of the string at current index, inserting the resulting strings into buffer
          let tmp = buffer[lindex].split_off(chindex);
          buffer.insert(lindex + 1, tmp);
          // If the line left behind is now a lone dot on a line, delete it and return
          if buffer[lindex] == ".\n" {
            buffer.remove(lindex);
            // Check if we have a trailing newline. If so delete it.
            if lindex == buffer.len() - 1 && buffer[lindex].len() == 1 {
              // This means there exists a last line of one character, the trailing newline to delete
              buffer.pop();
            }
            ret = true;
          }
          // Else increment and reset chindex.
          else {
            lindex += 1;
            chindex = 0;
          }
        }
        _ => (), // Ignore unknown codes
      }
    }
    // Then we print
    dists = crate::ui::print::print_input(state, &mut out, &buffer, lindex, chindex, dists.0)?;
  }
  // Just to not overwrite the last entered line, move down and to column 0
  out.queue(crossterm::cursor::MoveToColumn(0))?;
  out.queue(crossterm::cursor::MoveDown(dists.1 + 1))?;
  // Then return
  Ok(buffer)
}

// The core UI loop, handles events and runs the resulting commands on newlines
pub fn core_loop(state: &mut crate::State) -> Result<(), &'static str> {
  // All relevant data is stored in the State struct, so just get looping
  loop {

  }
}
