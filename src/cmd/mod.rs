use crate::error_consts::*;
use crate::io;

use crate::buffer::Buffer;

mod parse_selection;
use parse_selection::*;
mod parse_expressions;
use parse_expressions::*;
mod parse_path;
use parse_path::*;
mod parse_flags;
use parse_flags::*;


pub fn run<'a>(state: &'a mut crate::State, command: &'a mut str)
  -> Result<(), &'static str>
{
  // Declare flags for printing after the command has been executed.
  let mut p = false;
  let mut n = false;
  let mut l = false;

  // Parse out the command index and the selection
  let (cmd_i, selection) = parse_selection(command)?;

  // Use the cmd_i to get a clean selection
  let clean = &command[cmd_i + 1..].trim();

  // Match the command and act upon it
  match command[cmd_i..].trim().chars().next() {
    // No command is valid. It updates selection and thus works as a print when viewer is on
    None => {
      // Get and update the selection.
      let sel = interpret_selection(selection, state.selection, state.buffer.len(), false);
      state.selection = Some(sel);
      Ok(())
    },
    Some(ch) => match ch {
      // Quit commands
      'q' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        if state.buffer.saved() {
          state.done = true;
          return Ok(());
        }
        else {
          Err(UNSAVED_CHANGES)
        }
      }
      'Q' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        state.done = true;
        return Ok(());
      }
      // Help commands
      '?' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        println!("{}", HELP_TEXT);
        Ok(())
      },
      'h' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        println!("{:?}", state.error);
        Ok(())
      },
      'H' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        state.print_errors = !state.print_errors; // Toggle the setting
        Ok(())
      }
      // File commands
      'f' => { // Set or print filename
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        match parse_path(clean) {
          None => { // Print current filename
            if state.file.len() == 0 {
              println!("No file set.");
            }
            else {
              println!("Current file is: {}", state.file);
            }
          },
          Some(x) => { // Set new filename
            state.file = x.to_string();
          }
        }
        Ok(())
      }
      'e' | 'E' | 'r' => {
        // Read the selection if 'r', else error on any selection and return 0 on none (Lone default == no input)
        let index = 
          if ch == 'r' {
            Ok(interpret_selection(selection, state.selection, state.buffer.len(), true).1)
          }
          else {
            if selection == Sel::Lone(Ind::Default) {
              Ok(0)
            }
            else {
              Err(SELECTION_FORBIDDEN)
            }
          }?;
        // Only 'e' cares if the buffer is saved
        if !state.buffer.saved() & (ch == 'e') {
          Err(UNSAVED_CHANGES)
        }
        else {
          // Get the path (cutting of the command char and the trailing newline)
          let path = parse_path(clean).unwrap_or(&state.file);
          // Read the data from the file
          let mut data = crate::file::read_file(path)?;
          let datalen = data.len();
          // Empty the buffer if not 'r'
          if ch != 'r' {
            state.buffer = crate::buffer::VecBuffer::new();
          }
          state.buffer.insert(&mut data, index)?;
          state.buffer.set_saved();
          state.file = path.to_string();
          state.selection = Some((0,datalen));
          Ok(())
        }
      },
      'w' | 'W' => {
        // Get the selection to write
        let sel = interpret_selection(selection, state.selection, state.buffer.len(), true);
        // Get the path (cutting of the command char and the trailing newline)
        let path = parse_path(clean).unwrap_or(&state.file);
        // Get the data
        let data = state.buffer.get_selection(sel)?;
        let append = ch == 'W';
        // Write it into the file (append if 'W')
        crate::file::write_file(path, data, append)?;
        // If all was written, update state.file and set saved
        if sel == (0, state.buffer.len()) {
          state.buffer.set_saved();
          state.file = path.to_string();
        }
        state.selection = Some(sel);
        Ok(())
      }
      // Print commands
      // TODO: change this to an update of selection and always check for print flags
      'p' | 'n' | 'l' => {
        // Get and update the selection.
        let sel = interpret_selection(selection, state.selection, state.buffer.len(), false);
        state.selection = Some(sel);
        // Get the flags
        let mut flags = parse_flags(&command[cmd_i..].trim(), [('p', false), ('n', false), ('l', false)].iter().cloned().collect())?;
        // Set the global print flags (safe to unwrap since parse_flags never removes a key)
        p = flags.remove(&'p').unwrap();
        n = flags.remove(&'n').unwrap();
        l = flags.remove(&'l').unwrap();
        Ok(())
      }
      // Basic editing commands
      'a' => {
        // Get the input
        let mut input = io::read_insert(&state);
        // Calculate the selection
        let index = interpret_selection(selection, state.selection, state.buffer.len(), false).1;
        let end = index + input.len();
        // Insert the data
        state.buffer.insert(&mut input, index)?;
        // Update the selection
        state.selection = Some((index, end));
        Ok(())
      }
      'i' => {
        // Get the input
        let mut input = io::read_insert(&state);
        // Calculate the selection
        let index = interpret_selection(selection, state.selection, state.buffer.len(), false).0;
        let end = index + input.len();
        // Insert the data
        state.buffer.insert(&mut input, index)?;
        // Update the selection
        state.selection = Some((index, end));
        Ok(())
      }
      'c' => {
        // Get the input
        let mut input = io::read_insert(&state);
        // Calculate the selection
        let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
        let end = selection.0 + input.len();
        // Perform the replace
        state.buffer.change(&mut input, selection)?;
        // Update the selection
        state.selection = Some((selection.0, end));
        Ok(())
      }
      'd' => {
        // Calculate the selection
        let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
        // Perform the deletion
        state.buffer.delete(selection)?;
        // Try to have a selection afterwards
        if state.buffer.len() != 0 {
          // If we didn't just delete the whole head we can sub one from selection.0 for a valid index
          let start = if 0 != selection.0 { selection.0 - 1 } else { selection.0 };
          // If we didn't just delete the whole tail we can add one to selection.0 for a valid index
          let end = if state.buffer.len() != selection.0 { selection.0 + 1 } else { selection.0 };
          // Since bufferlen != 0 either start or end have been modified to get a non-empty selection
          state.selection = Some((start, end));
        }
        else {
          state.selection = None;
        }
        Ok(())
      }
      // Advanced editing commands
      'm' | 't' => {
        // Parse the index to move to
        let index = match parse_index(clean
          .trim_end_matches(|c: char| c.is_ascii_alphabetic() )
        )? {
          Ind::Default => state.selection.unwrap_or((0,state.buffer.len())).1,
          Ind::BufferLen => state.buffer.len(),
          Ind::Relative(x) => u_i_add(
            state.selection.map(|s| s.1).unwrap_or(state.buffer.len()),
            x
          ),
          Ind::Literal(x) => x,
        };
        // Calculate the selection
        let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
        let end = index + (selection.1 - selection.0);
        if ch == 'm' {
          // Perform the move
          state.buffer.mov(selection, index)?;
        }
        else {
          // Copy instead of moving
          state.buffer.copy(selection, index)?;
        }
        // Update the selection
        state.selection = Some((index, end));
        Ok(())
      }
      'j' => {
        // Calculate the selection
        let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
        // Perform the deletion
        state.buffer.join(selection)?;
        // Update the selection
        state.selection = Some((selection.0, selection.0 + 1)); // Guaranteed to exist, but may be wrong.
        Ok(())
      }    
      // Regex commands
      // s and g, is essence
      's' /* | 'g' */ => {
        // Calculate the selection
        let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
        // Read in the expressions
        let expressions = parse_expressions(clean);
        // Split based on command
        if ch == 's' {
          if expressions.len() == 3 { // A proper new expression was given
            let global = expressions[2].contains('g');
            // Perform the command, which returns the resulting selection
            state.selection = Some(
              state.buffer.search_replace((expressions[0], expressions[1]), selection, global)?
            );         
          }
          else { return Err(EXPRESSION_TOO_SHORT); }
        }
        else { // implies 'g'
        }
        Ok(())
      }
      _cmd => {
        Err(UNDEFINED_COMMAND)
      }
    }
  }?;
  
  // If print flags are set, print
  if p | n | l {
    if let Some(sel) = state.selection {
      // Get selection
      let output = state.buffer.get_selection(sel)?;
      // Print it (static false is to not limit by terminal height)
      crate::ui::format_print(state, output, sel.0, false, n, l)?;
    }
    else {
      Err(SELECTION_INVERTED)?
    }
  }
  // Othewise, print the height of the terminal -2 lines from start of the selection - 5 or so
  else {
    if let Some(sel) = state.selection {
      // Handle the cases where we would go out of index bounds
      let start = sel.0.saturating_sub(5);
      let end = state.buffer.len();
      let output = state.buffer.get_selection((start,end))?;
      crate::ui::format_print(state, output, start, true, true, false)?; // TODO: Handle flags
    }
  }

  Ok(())
}
