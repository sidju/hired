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
    // Print commands
    // TODO, change this to an update of selection and always check for print flags
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
    
    Some(cmd) => {
      Err(UNDEFINED_COMMAND)
    }
  }
}
