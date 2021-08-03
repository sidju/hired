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

// This input getter runs get_event and buffers the input with expected editing features
// Initial contents of the buffer is given as a vector of newline terminated strings
// A prefix can be given, which is then printed at start of every line and not included in input
// A terminator can be given.
// If given: input is returned after terminator has been entered alone on a line. Else on newline.
pub fn event_input(
  state: &mut super::HighlightingUI,
  initial_buffer: Vec<String>,
  prefix: Option<char>,
  terminator: Option<char>, // If none take only one line
) -> Result<Vec<String>, crossterm::ErrorKind> {
  let mut stdout = std::io::stdout();

  // Set the cursor to be visible, so our moves are visible
  stdout.queue(crossterm::cursor::Show)?;

  // Set up buffer and variables for moving in it
  let mut buffer = initial_buffer;
  if buffer.len() == 0 { buffer.push("\n".to_string()); } // The buffer mustn't be empty
  let mut lindex = 0; // Line index, lin-dex
  let mut chindex = 0; // Char index, ch-index

  // Variable for tracking how many steps back in history
  // we are when moving back in history
  let mut hoffset = state.command_history.len();
  // And one for keeping current input while moving about in history
  let mut semi_history = "\n".to_string();

  // Then the distances we need to remember between printing
  let mut dists = super::print::PrintData{ height: 0, cursor_y: 0, cursor_x: 0 };

  // And if we are to return
  let mut ret = false; // Flag when ready to return

  // Then input specific variables
  let mut partial = String::with_capacity(4); // To store partial chars

  // and finally movement specific
   // If we move via up/down to a line shorter than our current chindex that prior chindex is saved
   // here, so we can go to that prior chindex if next operation is up/down. Else it's cleared.
  let mut goal_chindex = None;

  // loop getting input events, ignoring those we can't handle.
  while !ret {
    // Print before blocking waiting for input

    // Move up the cursor to overwrite prior input with this input
    if (dists.height - dists.cursor_y) > 0 {
      stdout.queue(crossterm::cursor::MoveUp(dists.height - dists.cursor_y))?;
    }
    stdout.queue(crossterm::cursor::MoveToColumn(0))?;
    // Clear away old print
    stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown))?;
    // Then print
    let syntax = state.syntax_lib.find_syntax_plain_text();
    dists = super::print::internal_print(
      state,
      &syntax,
      &mut buffer.iter().map(|line| &line[..]),
      prefix,
      Some((lindex, chindex)),
      0,
      false,
      false,
    )?;
    // And move to the positions returned
    if dists.cursor_y > 0 {
      stdout.queue(crossterm::cursor::MoveUp(dists.cursor_y))?;
    }
    // We add one, since we wish the cursor to be one step beyond the last character
    stdout.queue(crossterm::cursor::MoveToColumn(dists.cursor_x))?;
    // Then make sure to flush this, or the cursor won't move
    stdout.flush()?;

    match crossterm::event::read()? {
      // If resize event, just update usize
      Event::Resize(x, y) => { state.term_size = (x as usize, y as usize); },
  
      // Ignore mouse events
      Event::Mouse(_) => (),
  
      // If key event, match code and modifiers and handle thereafter
      Event::Key(key) => {
        // Check if any of the state variables should be cleared
        // Done here instead of in all but 2 key-handlers
  
        // If doing anything but continued input of partial character, clear it
        if let KeyCode::Char(_) = key.code {} else {
          partial.clear();
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
            if terminator.is_none() {
              ret = true;
            }
            // Else, add a line
            else {
              // Insert a newline to properly terminate current line
              buffer[lindex].insert(chindex, '\n');
              chindex += 1;
              // Split of the string at current index, inserting the resulting strings into buffer
              let tmp = buffer[lindex].split_off(chindex);
              buffer.insert(lindex + 1, tmp);
              // If the line left behind is now a lone dot on a line, delete it and return
              // Check if we just created the terminating line
              let mut iter = buffer[lindex].chars();
              if iter.next() == terminator && iter.next() == Some('\n') {
                // Remove the terminating line
                buffer.remove(lindex);
                // Check to clear unexpected line created after terminating
                if buffer[lindex] == "\n" {
                  buffer.remove(lindex);
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
            // Go back/forth in history if in one-line mode
            if terminator.is_none() {
              match key.code {
                KeyCode::Up => {
                  // If leaving present
                  if hoffset == state.command_history.len() {
                    // Save current input line as semi history, unwrap or shouldn't ever be needed
                    semi_history = buffer.pop().unwrap_or("\n".to_string());
                  }
                  else {
                    buffer.pop();
                  }
                  // Then move into history
                  hoffset = hoffset.saturating_sub(1);
                },
                KeyCode::Down => {
                  // If not in the present, move forward in history
                  if hoffset < state.command_history.len() { hoffset += 1; }
                  buffer.pop();
                },
                _ => (),
              }
              // Read that history entry into the buffer
              buffer.push(
                state.command_history
                  .get(hoffset) // Get history at offset
                  .map(|line| line.clone()) // Convert from &str to String
                  .unwrap_or(semi_history.clone()) // If none we have arrived in the present
              )
            }
            else {
              // First move to the indicated line, if possible
              match key.code {
                KeyCode::Up => { lindex = lindex.saturating_sub(1); },
                KeyCode::Down => { if lindex < buffer.len() - 1 { lindex += 1; } },
                _ => (),
              }
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
      } // End of Key input event matching
    } // End of event match
  } // End of while

  // Before returning print a clean print to leave in the buffer

  // Move up the cursor to overwrite prior input with this input
  if (dists.height - dists.cursor_y) > 0 {
    stdout.queue(crossterm::cursor::MoveUp(dists.height - dists.cursor_y))?;
  }
  stdout.queue(crossterm::cursor::MoveToColumn(0))?;
  // Clear away old print
  stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown))?;
  // Then print
  let syntax = state.syntax_lib.find_syntax_plain_text();
  super::print::internal_print(
    state,
    &syntax,
    &mut buffer.iter().map(|line| &line[..]),
    prefix,
    None,
    0,
    false,
    false,
  )?;
  // Then flush and return
  stdout.flush()?;
  Ok(buffer)
}
