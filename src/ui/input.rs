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

// A print function that prints the current state of the buffer over the previous state
// Moves the cursor to the given position after printing
// Moves 'topdist' lines up before starting printing
// Returns cursor's distance to top and bottom, respectively
fn print_input(state: &mut crate::State, out: &mut impl Write, buffer: &Vec<String>, lindex: usize, chindex: usize, topdist: u16)
  -> Result<(u16, u16), crossterm::ErrorKind>
{
  // First go to the top of the previous input
  if topdist != 0 {
    out.queue(crossterm::cursor::MoveUp(topdist))?;
  }
  out.queue(crossterm::cursor::MoveToColumn(0))?;
  // And clear all of the previous print
  out.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown))?;

  // A bool to track if we have passed our current cursor index, to find its position
  let mut passed = false;

  // To track the position of the cursor, by when we passed it in the buffer
  let mut x: u16 = 0;
  let mut y: u16 = 0;
  // And the height of the print, for returning distance to top and bottom
  let mut height: u16 = 0;

  // Then start looping over the buffer
  for (linenr, line) in buffer.iter().enumerate() {

    // Create a character to track nr characters printed this line
    let mut chars_printed = 0;

    // If this isn't the first line, newline and carriage retun
    if linenr != 0 {
      out.queue(Print("\n\r"))?;
      // And incement height and maybe y
      height += 1;
      if passed { y += 1; }
    }

    for (i, ch) in line.char_indices() {
      // If we haven't reached our current cursor position before, check if we have now.
      // This by nesting if not found, if lindex == line_i, if chindex == i
      if ! passed {
        if lindex <= linenr && chindex <= i {
          // Set the x coordinate using chars_printed modulo terminal width
          x = (chars_printed % state.term_size.0) as u16;
          // And mark chindex as passed
          passed = true;
        }
      }
    
      // Ignore characters that are newlines (since they confuse our wrapping and are handled by the end of line)
      if ch != '\n' && ch != '\r' {
        // If our current x position is 0 in modulo of the terminal width
        // we are about to go out the side of the terminal
        if chars_printed + 1 % state.term_size.0 == 0 {
          // Print newline and carriage return
          out.queue(Print("\n\r"))?;
          // Increment the height of this print
          height += 1;
          // If the cursor is marked as found/passed, increment cursor height as well
          if passed { y += 1; }
        }
    
        // TODO: Handle printing weird characters with other widths than 1

        // Increment the number of characters printed
        chars_printed += 1;
        // Finally, print the character
        out.queue(Print(ch))?;
      }
    } // End of chars
  } // End of lines

  // When done with looping, move the cursor to the calculated coordinates
  out.queue(crossterm::cursor::MoveToColumn(x + 1))?;
  if y != 0 {
    out.queue(crossterm::cursor::MoveUp(y))?;
  }
  out.flush()?;

  // Finally we return the distances
  Ok((height - y, y))
}

// This input getter runs get_event and buffers the input until a lone . appears on a line
// Then it returns that buffer.
pub fn get_input(state: &mut crate::State)
  -> Result<Vec<String>, crossterm::ErrorKind>
{
  // To be placed in State
  let mut out = std::io::stdout();

  // Set the cursor to be visible, so our moves are visible
//  out.queue(crossterm::cursor::Show)?;
//  out.queue(crossterm::cursor::EnableBlinking)?;

  // Create the buffer we store the input in
  let mut buffer = Vec::new();
  buffer.push("\n".to_string());
  // Add some index variables we'll need
  let mut lindex = 0; // Line index, lin-dex
  let mut chindex = 0; // Char index, ch-index
  let mut char_size = 0; // The nr of bytes the current char is
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
          buffer[lindex].insert(chindex, ch);
          chindex += ch.len_utf8();
          char_size += ch.len_utf8();
          // Verify that we are at a valid char-boundary before printing, to support modifiers
          if buffer[lindex].is_char_boundary(chindex) {
            // Then clear char_size, to signify that the character is complete
            char_size = 0;
          }
        },
        (KeyCode::Left, KeyModifiers::NONE) => {
          let prev = rfind_boundary(&buffer[lindex], chindex);
          if prev != chindex {
            chindex = prev;
          }
        },
        (KeyCode::Right, KeyModifiers::NONE) => {
          let next = find_boundary(
            &buffer[lindex][.. buffer[lindex].len() - 1],
            chindex
          );
          if next != chindex {
            chindex = next;
          }
        },
        (KeyCode::Backspace, KeyModifiers::NONE) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
          // First we need to find the nearest preceeding character boundary
          let b = rfind_boundary(&buffer[lindex], chindex);
          chindex = b;
          buffer[lindex].remove(chindex);
        },
        (KeyCode::Enter, KeyModifiers::NONE) => {
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
    dists = print_input(state, &mut out, &buffer, lindex, chindex, dists.0)?;
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
