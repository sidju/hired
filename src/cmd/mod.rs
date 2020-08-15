use crate::error_consts::*;
use crate::io;

use crate::buffer::Buffer;

mod parse_selection;
use parse_selection::*;

pub fn run<'a>(state: &'a mut crate::State, command: &'a mut str)
  -> Result<(), &'static str>
{
  // Parse out the command index and the selection
  let (cmd_i, selection) = parse_selection(command)?;

  // Match the command and act upon it
  match command[cmd_i..].chars().next() {
    None => Err(NO_COMMAND_ERR),
    // Quit commands
    Some('q') => {
      if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
      if state.buffer.saved() {
        state.done = true;
        Ok(())
      }
      else {
        Err(UNSAVED_CHANGES)
      }
    }
    Some('Q') => {
      if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
      state.done = true;
      Ok(())
    }
    // Help commands
    Some('h') => {
      if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
      println!("{:?}", state.error);
      Ok(())
    },
    Some('H') => {
      if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
      state.print_errors = !state.print_errors; // Toggle the setting
      Ok(())
    }
    // File commands
    // TODO: Unify filename handling
    // TODO: Unify 'r' with 'e' and 'E'
    Some('f') => { // Set or print filename
      if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
      match &command[cmd_i + 1 .. command.len()-1] {
        "" => { // Print current filename
          println!("{:?}", state.file);
        },
        x => { // Set new filename
          state.file = Some(x.to_string());
        }
      }
      Ok(())
    }
    Some('e') | Some('E') => {
      if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
      if !state.buffer.saved() & (command[cmd_i..].chars().next() == Some('e')) {
        Err(UNSAVED_CHANGES)
      }
      else {
        // Get the path (cutting of the command char and the trailing newline)
        let path = match &command[cmd_i + 1 .. command.len()-1] {
          "" => match state.file.as_ref() {
            Some(x) => x,
            None => "",
          },
          x => x,
        };
        // Read the data from the file
        let mut data = crate::file::read_file(path)?;
        let datalen = data.len();
        // Insert into a clean buffer
        state.buffer = crate::buffer::VecBuffer::new();
        state.buffer.insert(&mut data, 0)?;
        state.buffer.set_saved();
        state.file = Some(path.to_string());
        state.selection = Some((0, datalen));
        Ok(())
      }
    },
    Some('r') => {
      // Get the index to append at
      let index = interpret_selection(selection, state.selection, state.buffer.len(), true).1;
      // Get the path (cutting of the command char and the trailing newline)
      let path = match &command[cmd_i + 1 .. command.len()-1] {
        "" => match state.file.as_ref() {
          Some(x) => x,
          None => "",
        },
        x => x,
      };
      // Read the data from the file
      let mut data = crate::file::read_file(path)?;
      let datalen = data.len();
      // Append to the buffer at given index
      state.buffer.insert(&mut data, index)?;
      // Update file only if not set before (TODO: Maybe not do this? It is quite weird.)
      if state.file == None { state.file = Some(path.to_string()); }
      state.selection = Some((index, index + datalen));
      Ok(())
    },
    Some('w') | Some('W') => {
      // Get the selection to write
      let sel = interpret_selection(selection, state.selection, state.buffer.len(), true);
      // Get the path (cutting of the command char and the trailing newline)
      let path = match &command[cmd_i + 1 .. command.len()-1] {
        "" => match state.file.as_ref() {
          Some(x) => x,
          None => "",
        },
        x => x,
      };
      // Get the data
      let data = state.buffer.get_selection(sel)?;
      let append = command[cmd_i..].chars().next() == Some('W');
      // Write it into the file (append if 'W')
      crate::file::write_file(path, data, append);
      // If all was written, update state.file and set saved
      if sel == (0, state.buffer.len()) {
        state.buffer.set_saved();
        state.file = Some(path.to_string());
      }
      state.selection = Some(sel);
      Ok(())
    }
    // Print commands
    // TODO: change this to an update of selection and always check for print flags
    Some('p') | Some('n') | Some('l') => {
      // Identify which flags are set
      let mut n = false;
      let mut l = false;
      for char in command[cmd_i..command.len()-1].chars() {
        match char {
          'n' => { n = true; },
          'l' => { l = true; },
          'p' => { },
          _ => return Err(UNDEFINED_FLAG),
        }
      }
      // Normalise the selection and get its lines
      let sel = interpret_selection(selection, state.selection, state.buffer.len(), false);
      let output = state.buffer.get_selection(sel)?;
      // Print the output
      crate::io::format_print( state, output, sel.0, n, l );
      // And save the selection
      state.selection = Some(sel);
      Ok(())
    }
    // Basic editing commands
    Some('a') => {
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
    Some('i') => {
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
    Some('c') => {
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
    Some('d') => {
      // Calculate the selection
      let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
      // Perform the deletion
      state.buffer.delete(selection)?;
      // Update the selection
      state.selection = None;
      Ok(())
    }
    // Advanced editing commands
    // move, transfer, join etc
    Some('m') | Some('t') => {
      // Parse the index to move to
      let index = match parse_index(&(command[cmd_i + 1..command.len()-1])
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
      if command[cmd_i..].chars().next() == Some('m') {
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
    Some('j') => {
      // Calculate the selection
      let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
      // Perform the deletion
      state.buffer.join(selection)?;
      // Update the selection
      state.selection = None; // Could be calculated, but I won't bother now
      Ok(())
    }
    
    // Regex commands
    // s and g, is essence
    
    Some(cmd) => {
      Err(UNDEFINED_COMMAND)
    }
  }
}
