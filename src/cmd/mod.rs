use crate::error_consts::*;
use crate::ui;

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
  // And a bool to say if view has any reason to be reprinted
  let mut view_changed = false;

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
      state.buffer.verify_selection(sel)?;
      state.selection = Some(sel);
      view_changed = true; // Set to reprint
      Ok(())
    },
    Some(ch) => match ch {
      // Quit commands
      'q' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        parse_flags(clean, "")?;
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
        parse_flags(clean, "")?;
        state.done = true;
        return Ok(());
      }
      // Help commands
      '?' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        parse_flags(clean, "")?;
        ui::print(&mut state.stdout, HELP_TEXT);
        Ok(())
      },
      'h' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        parse_flags(clean, "")?;
        ui::print(&mut state.stdout, state.error.unwrap_or(NO_ERROR));
        Ok(())
      },
      'H' => {
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        parse_flags(clean, "")?;
        state.print_errors = !state.print_errors; // Toggle the setting
        Ok(())
      }
      // File commands
      'f' => { // Set or print filename
        if selection != Sel::Lone(Ind::Default) { return Err(SELECTION_FORBIDDEN); }
        parse_flags(clean, "")?;
        match parse_path(clean) {
          None => { // Print current filename
            if state.file.len() == 0 {
              ui::print(&mut state.stdout, NO_FILE);
            }
            else {
              ui::println(&mut state.stdout, &state.file);
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
        if !state.buffer.saved() && (ch == 'e') {
          Err(UNSAVED_CHANGES)
        }
        else {
          // Get the path (cutting of the command char and the trailing newline)
          let path = parse_path(clean);
          // If opening another file
          // or inserting current file again
          // or unconditionally resetting current changes
          // then view has changed
            if path.is_some() || ch == 'r' || ch == 'E' { view_changed = true; }
          let path = path.unwrap_or(&state.file);
          // Read the data from the file
          let mut data = crate::file::read_file(path, ch == 'r')?;
          let datalen = data.len();
          // Empty the buffer if not 'r'
          if ch != 'r' {
            state.buffer = crate::buffer::VecBuffer::new();
          }
          state.buffer.insert(&mut data, index)?;
          state.buffer.set_saved();
          state.file = path.to_string();
          state.selection = Some((index, index + datalen));
          Ok(())
        }
      },
      'w' | 'W' => {
        // Get the selection to write
        let sel = interpret_selection(selection, state.selection, state.buffer.len(), true);
        // Get the path (cutting of the command char and the trailing newline)
        let path = parse_path(clean).unwrap_or(&state.file);
        // Get flags
        let q = parse_flags(&command[cmd_i + 1 ..], "q")?.remove(&'q').unwrap();
        // If the 'q' flag is set the whole buffer must be selected
        if q && sel != (0, state.buffer.len()) { return Err(UNSAVED_CHANGES); }
        // Get the data
        let data = state.buffer.get_selection(sel)?;
        let append = ch == 'W';
        // Write it into the file (append if 'W')
        crate::file::write_file(path, data, append)?;
        // If all was written, update state.file and set saved
        if sel == (0, state.buffer.len()) {
          state.buffer.set_saved();
          state.file = path.to_string();
          state.done = q;
        }
        else {
          state.selection = Some(sel);
        }
        Ok(())
      }
      // Print commands
      'p' | 'n' | 'l' => {
        // Get and update the selection.
        let sel = interpret_selection(selection, state.selection, state.buffer.len(), false);
        state.buffer.verify_selection(sel)?;
        state.selection = Some(sel);
        // Get the flags
        let mut flags = parse_flags(&command[cmd_i..], "pnl")?;
        // Set the global print flags (safe to unwrap since parse_flags never removes a key)
        p = flags.remove(&'p').unwrap();
        n = flags.remove(&'n').unwrap();
        l = flags.remove(&'l').unwrap();
        Ok(())
      }
      // Basic editing commands
      'a' | 'i' | 'c' => {
        let sel = interpret_selection(selection, state.selection, state.buffer.len(), false);
        let mut flags = parse_flags(clean, "pnl")?;
        p = flags.remove(&'p').unwrap();
        n = flags.remove(&'n').unwrap();
        l = flags.remove(&'l').unwrap();
        // When all possible checks have been run, get input
        let mut input = ui::get_input(state)?;
        let new_sel = match ch {
          'a' | 'i' => {
            if input.len() != 0 {
              let start = if ch == 'a' { sel.1 } else { sel.0 };
              let end = start + input.len();
              state.buffer.insert(&mut input, start)?;
              Some((start, end))
            }
            else {
              // If no input the command was cancelled, keep the old selection
              state.selection
            }
          }
          'c' => {
            let end = sel.0 + input.len();
            state.buffer.change(&mut input, sel)?;
            if input.len() != 0 {
              Some((sel.0, end))
            }
            else {
              // Same as delete, use same post-selection logic
              if sel.0 != 0 { Some((sel.0 - 1, sel.0)) }
              else if sel.0 != state.buffer.len() { Some((sel.0, sel.0 + 1)) }
              else { None }
            }
          }
          _ => { panic!("Unreachable code reached"); }
        };
        // If resulting selection is empty, set original selection?
        state.selection = new_sel;
        view_changed = true;
        Ok(())
      }
      'd' => {
        let sel = interpret_selection(selection, state.selection, state.buffer.len(), false);
        // Since selection after execution can be 0 it isn't allowed to auto print after
        parse_flags(clean, "")?;
        state.buffer.delete(sel)?;
        // Try to figure out a selection after the deletion
        state.selection = 
          if sel.0 != 0 { Some((sel.0 - 1, sel.0)) }
          else if sel.0 != state.buffer.len() { Some((sel.0, sel.0 + 1)) }
          else { None }
        ;
        view_changed = true;
        Ok(())
      }
      // Advanced editing commands
      'm' | 't' => {
        // Split out the potential print flags from the index (nice extra feature)
        let ind_end = clean.find( char::is_alphabetic ).unwrap_or(clean.len());
        // Then parse first goal index, then flags
        let index = interpret_index(
          parse_index(&clean[..ind_end])?,
          state.selection.map(|s| s.1),
          state.buffer.len(),
          state.buffer.len(),
        );
        let mut flags = parse_flags(&clean[ind_end..], "pnl")?;
        p = flags.remove(&'p').unwrap();
        n = flags.remove(&'n').unwrap();
        l = flags.remove(&'l').unwrap();
        // Calculate the selection
        let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
        let end = index + (selection.1 - selection.0);
        // Make the change
        if ch == 'm' {
          state.buffer.mov(selection, index)?;
        }
        else {
          state.buffer.copy(selection, index)?;
        }
        // Update the selection
        state.selection = Some((index, end));
        view_changed = true;
        Ok(())
      }
      'j' => {
        // Calculate the selection
        let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
        let mut flags = parse_flags(clean, "pnl")?;
        p = flags.remove(&'p').unwrap();
        n = flags.remove(&'n').unwrap();
        l = flags.remove(&'l').unwrap();
        state.buffer.join(selection)?;
        state.selection = Some((selection.0, selection.0 + 1)); // Guaranteed to exist, but may be wrong.
        view_changed = true;
        Ok(())
      }    
      // Regex commands
      // s and g, in essence
      's' /* | 'g' */ => {
        // Calculate the selection
        let selection = interpret_selection(selection, state.selection, state.buffer.len(), false);
        // Read in the expressions
        let expressions = parse_expressions(clean);
        // Split based on command
        if ch == 's' {
          if expressions.len() == 3 { // A proper new expression was given
            let mut flags = parse_flags(&(expressions[2]), "gpnl")?;
            let g = flags.remove(&'g').unwrap();
            p = flags.remove(&'p').unwrap();
            n = flags.remove(&'n').unwrap();
            l = flags.remove(&'l').unwrap();
            // Perform the command, which returns the resulting selection
            state.selection = Some(
              state.buffer.search_replace((expressions[0], expressions[1]), selection, g)?
            );         
            view_changed = true;
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
      Err(SELECTION_EMPTY)?
    }
  }
  // Othewise, print the height of the terminal -2 lines from start of the selection - 5 or so
  else if view_changed {
    if let Some(sel) = state.selection {
      // Handle the cases where we would go out of index bounds
      let start = sel.0.saturating_sub(15);
      let end = state.buffer.len();
      let output = state.buffer.get_selection((start,end))?;
      crate::ui::format_print(state, output, start, true, true, false)?; // TODO: Handle flags
    }
  }

  Ok(())
}
