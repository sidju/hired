use crate::io;

fn parse_index(index: &str) -> Result<usize, String> {
    match index.parse::<usize>() {
        Ok(int) => Ok(int),
        Err(e) => Err(format!("Error parsing index. {}", e)),
    }
}
fn parse_selection(string: &str,
                   selection: &Option<(usize,usize)>,
                   bufferlen: usize,
) -> Result<(usize, usize), String> {
    // If no string was given we use the default
    if string.len() == 0 {
        Ok(selection.unwrap_or_else(||(0 as usize, bufferlen)))
    }
    // Else we parse what is given, if possible
    else {
        // split the parsing based on where there is a ','
        match string.find(',') {
            Some(x) => {
                let start = if x != 0 {
                    parse_index(&string[..x])?
                }
                else {
                    1 // If there is nothing to parse, use default
                };
                let end = if x + 1 != string.len() {
                    parse_index(&string[x+1..])?
                }
                else {
                    bufferlen // If there is nothing to parse, use default
                };

                // Handle assorted edge cases
                if end > bufferlen {
                    Err("Selection overshoots the buffer length.".to_string())
                }
                else if start > end {
                    Err("Selection inverted. start of selection must be smaller than end.".to_string())
                }
                else if start == 0 {
                    Err("0 in an invalid address for a selection.".to_string())
                }
                else {
                    Ok((start - 1, end ))
                }
            },
            None => {
                // If no ',' exists we check if one index was given
                let lone = parse_index(string)?;
                // Avoid panics from overshooting the buffer length
                if lone + 1 > bufferlen {
                    Err("Selection overshoots the buffer length.".to_string())
                }
                else if lone == 0 {
                    Ok((lone, lone))
                }
                else {
                    Ok((lone - 1, lone))
                }
            },
        }
    }
}
// Parse out nr expression segments as a string vector
// Returns the index it finished at
// Errors if not enough segments could be found
fn parse_expression(expression: &str, nr: usize)
                    -> Result<(usize, Vec<&str>), String> {

    let mut segments = Vec::new();
    let mut last_index = 0;
    let mut i = 0;
    // Use the first char in the string to split the rest
    let separator = match expression.chars().next() {
        Some(ch) => ch,
        None => return Err("No expression provided.".to_string()),
    };

    // loop over the slice and add the slices to the res vector
    for (index, ch) in expression[1..].char_indices() {
        // If we are at a separator we save the slice into the vector
        if ch == separator {
            // since we are iterating of a slice shifted one step from
            // the expression slice we add one to all indices
            segments.push(&expression[last_index + 1 .. index + 1]);
            last_index = index;
            i += 1;
            // If we have found enough segments we break
            if i >= nr {
                break;
            }
        }
    }
    // Error if insufficient expression segments were given
    if i < nr {
        Err("Expression too short.".to_string())
    }
    else {
        Ok((last_index, segments))
    }
}
fn insert(buffer: &mut Vec<String>, data: &mut Vec<String>,mut index: usize)
          -> Result<(), String> {
    // If we are adding to the end, we use append (split doest't handle it)
    if index >= buffer.len() {
        #[cfg(feature = "debug")]
        {
            println!("appending since len was {} and index was {}",
                     buffer.len(), index);
        }
        buffer.append(data);
        Ok(())
    }
    else {
        // 0 is valid and should place on first line, others should be shifted
        if index != 0 {
            index -= 1;
        }
        #[cfg(feature = "debug")]
        {
            println!("inserting at index {}", index);
        }
        // To minimise the processing we split the vector
        //(moving all elements after the insertion to a separate vector)
        let mut tail = buffer.split_off(index);
        // And then append both the insert and the split off part
        buffer.append(data);
        buffer.append(&mut tail);
        Ok(())
    }
}
pub fn handle_command(state: &mut crate::State, command: &mut String)
                      -> Result<(), String> {
    for (index, ch) in command.trim().char_indices() {
        match ch {
            'q' => {
                if command.trim().len() > 1 {
                    return Err("q doesn't take any arguments".to_string());
                }
                else {
                    state.done = true;
                    return Ok(());
                }
            },
            'h' => {
                if command.trim().len() > 1 {
                    return Err("h doesn't take any arguments".to_string());
                }
                else {
                    return io::print_error(&state.error);
                }
            },
            'a' => {
                let line = parse_selection(&command[0..index],
                                           &state.selection,
                                           state.buffer.len()
                )?.1;
                // Read in the text to append
                let mut data = io::read_insert(&state);
                // Set the selection to be the inserted lines
                let new_selection = Some((line, line + data.len()));
                // Insert it into the buffer at instructed line
                insert(&mut state.buffer, &mut data, line + 1)?;
                state.selection = new_selection;
                return Ok(())
            },
            'i' => {
                let line = parse_selection(&command[0..index],
                                           &state.selection,
                                           state.buffer.len()
                )?.0;
                // Read in the text to append
                let mut data = io::read_insert(&state);
                // Set the selection to be the inserted lines
                let new_selection = Some((line, line + data.len()));
                // Insert it into the buffer at instructed line
                insert(&mut state.buffer, &mut data, line)?;
                state.selection = new_selection;
                return Ok(())
            },
            'p' => {
                let lines = parse_selection(&command[0..index],
                                           &state.selection,
                                           state.buffer.len()
                )?;
                // Set the selection to be the printed lines
                let new_selection = Some(lines);
                io::print_selection(lines, &state.buffer, true)?;
                state.selection = new_selection;
                return Ok(())
            },
            _ => {},
        }
    }
    // We should never leave the loop without returning, so error
    Err("Unknown command".to_string())
}
