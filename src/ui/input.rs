// This module takes events and handles them as text input or commands

// The trait for queueing commands
use crossterm::QueueableCommand;
// All the event classes
use crossterm::event::{KeyCode, KeyModifiers, Event};
// And the writeable trait, to be able to flush stdout
use std::io::Write;

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
fn event_input(state: &mut crate::State, command: bool)
  -> Result<Vec<String>, crossterm::ErrorKind>
{
  // Set the cursor to be visible, so our moves are visible
  state.stdout.queue(crossterm::cursor::Show)?;
  // Also print the prompt string, so that it exists before the first received event
  if command {
    state.stdout.queue(crossterm::style::Print(&state.prompt))?;
    state.stdout.flush()?;
  }

  // Create the buffer and the variables to move in it
  let mut buffer = Vec::new();
  buffer.push("\n".to_string());
  let mut lindex = 0; // Line index, lin-dex
  let mut chindex = 0; // Char index, ch-index

  // Then the distances we need to remember between printing
  let mut dists: (u16, u16) = (0,0); // For cursor to top and bottom of print, respectively

  // And if we are to return
  let mut ret = false; // Flag when ready to return

  // Then input specific variables
  let mut partial = String::with_capacity(4); // To store partial chars

  // and finally movement specific
   // If we move via up/down to a line shorter than our current chindex that prior chindex is saved
   // here, so we can go to that prior chindex if next operation is up/down. Else it's cleared.
  let mut goal_chindex = None;

  // loop getting input events, ignoring those we can't handle.
  while !ret { match crossterm::event::read()? {
    // If resize event, just update usize
    Event::Resize(x, y) => { state.term_size = (x as usize, y as usize); },

    // Ignore mouse events
    Event::Mouse(_) => (),

    // If key event, match code and modifiers and handle thereafter
    Event::Key(key) => {
      // Check if any of the state variables should be cleared
      // Done here instead of in all but 2 key-handlers

      // If doing anything but continued input of partial character, clear it
      match key.code {
        KeyCode::Char(_) => (),
        _ => { partial.clear(); },
      }

      // If doing anything but moving up/down, clear goal_chindex
      if (key.code != KeyCode::Up &&
         key.code != KeyCode::Down ) ||
         key.modifiers != KeyModifiers::NONE
      {
        goal_chindex = None;
      }

      match (key.code, key.modifiers) {

        // Start with true input; characters and deletions
        (KeyCode::Char(ch), KeyModifiers::SHIFT) | (KeyCode::Char(ch), KeyModifiers::NONE) => {
          partial.push(ch);
          // If the partial is now complete, put it in the buffer
          if partial.is_char_boundary(0) {
            let tmp = chindex;
            chindex += partial.len();
            buffer[lindex].insert(tmp, partial.remove(0));
          }
        },

        (KeyCode::Backspace, KeyModifiers::NONE) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
          if chindex == 0 {
            // Join this and preceeding line
            if lindex != 0 {
              // Go to end of previous line, remove its newline and append current line
              let tmp = buffer.remove(lindex);
              lindex -= 1;
              buffer[lindex].pop();
              chindex = buffer[lindex].len();
              buffer[lindex].push_str(&tmp);
            }
          }
          else {
            // Just delete preceeding character
            chindex = rfind_boundary(&buffer[lindex], chindex);
            buffer[lindex].remove(chindex);
          }
        },

        (KeyCode::Delete, KeyModifiers::NONE) => {
          if chindex >= buffer[lindex].len() - 1 {
            // Join this and following line
            // But only if there is a following line
            if lindex != buffer.len() - 1 {
              // Remove our newline and append next line
              buffer[lindex].pop();
              let tmp = buffer.remove(lindex + 1);
              buffer[lindex].push_str(&tmp);
            }
          }
          else {
            // Just delete following character
            buffer[lindex].remove(chindex);
          }
        },

        (KeyCode::Enter, KeyModifiers::NONE) => {
          // If only getting one line, return
          if command {
            ret = true;
          }
          // Else, add a line
          else {
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
        }

        // Then we have movement; right/left, up/down, home/end
        (KeyCode::Right, KeyModifiers::NONE) => {
          if chindex == buffer[lindex].len() - 1 {
            // Go to next line
            if buffer.len() - 1 > lindex {
              lindex += 1;
              chindex = 0;
            }
          }
          else {
            chindex = find_boundary(
              &buffer[lindex][.. buffer[lindex].len() - 1],
              chindex
            );
          }
        },

        (KeyCode::Left, KeyModifiers::NONE) => {
          if chindex == 0 {
            // Go to previous line
            if lindex > 0 {
              lindex -= 1;
              chindex = buffer[lindex].len() - 1;
            }
          }
          else {
            chindex = rfind_boundary(&buffer[lindex], chindex);
          }
        },

        (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
          // First move to the indicated line, if possible
          match key.code {
            KeyCode::Up => { lindex = lindex.saturating_sub(1); },
            KeyCode::Down => { if lindex < buffer.len() - 1 { lindex += 1; } },
            _ => (),
          }
          // Then try to go to goal_chindex and place chindex within the new line
          match goal_chindex {
            Some(tmp) => { chindex = tmp; },
            None => (),
          }
          // If current chindex is too big, save it as goal and go to nearest valid chindex
          if chindex >= buffer[lindex].len() {
            goal_chindex = Some(chindex);
            chindex = buffer[lindex].len() - 1;
          }
        },

        (KeyCode::Home, KeyModifiers::NONE) => {
          lindex = 0;
          chindex = 0;
        },

        (KeyCode::End, KeyModifiers::NONE) => {
          lindex = buffer.len() - 1;
          chindex = buffer[lindex].len() - 1;
        },

        _ => (), // Ignore unknown codes
      } // End of matching key-codes and modifiers

      // Then we print
      dists = crate::ui::print::print_input(
        state,
        &buffer,
        lindex,
        chindex,
        dists.0,
        command,
      )?;
    } // End of Key input event matching

  }} // End of while and event match

  // To not overwrite any of our buffer, go to bottom an move to next line
  if dists.1 > 0 {
    state.stdout.queue(crossterm::cursor::MoveDown(dists.1))?;
  }
  state.stdout.queue(crossterm::style::Print("\n\r"))?;
  // Then flush and return
  state.stdout.flush()?;
  Ok(buffer)
}

// The public re-exports of the event input function
pub fn get_input(state: &mut crate::State)
  -> Result<Vec<String>, &'static str>
{
  event_input(state, false)
    .map_err(|_| "Error occured in input")
}

pub fn get_command(state: &mut crate::State)
  -> Result<String, &'static str>
{
  event_input(state, true)
    .map_err(|_| "Error occured in input")
    .map(|mut x| x.pop().unwrap_or_else(|| String::with_capacity(0)))
}
