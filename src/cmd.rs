use crate::io;
use crate::buffer::Buffer;

const SELECTION_PARSE_ERR: &str = "Could not parse given selection.";
const SELECTION_OUT_OF_BOUNDS: &str = "Selection out of bounds.";
const SELECTION_INVERTED: &str = "Selection is empty or inverted.";

const NO_COMMAND_ERR: &str = "No valid command given.";
const VALID_COMMANDS: &str =
    "abcdefthijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

// Utility function needed a few times while parsing
pub fn find_any_of(options: &str, input: &str) -> Option<usize> {
    for (index, ch) in input.char_indices() {
        if options.chars().any(|cmd| cmd == ch) {
            return Some(index);
        }
    }
    None
}

// A parsing state machine
// Each state is the task completed when the state is active
pub struct Parse<S> {
    stage: S
}
struct Init<'a> {
    input: &'a str,
    state: &'a crate::State,
}
struct Command<'a> {
    input: &'a str,
    selection: (usize, usize),
    command: char,
    state: &'a crate::State,
}
struct Expression<'a> {
    selection: (usize, usize),
    command: char,
    expression: Vec<&'a str>,
}

// A command abstraction
pub enum Cmd<'a> {
    A(Append<'a>),
}
struct Append<'a> {
    state: &'a dyn Buffer,
    selection: (usize, usize),
    input: Vec<String>,
    // Flags
    p: bool,
}

// impl Parse<Cmd<'a>> {
//     pub fn parse<'a>(input: &'a str, state: &'a crate::State)
//                      -> Self {
//         let start = Parse{ stage: Init{
//             input: input,
//             state: state,
//         }};
//         // Perform the parsing
//         start
//             .parse_selection()
//             .parse_command()
//             .parse_expression()
//             .parse_flags()
//     }
// }
impl <'a> Parse<Command<'a>> {
    // Get the selection and command, the basic data
    fn parse_command(from: Parse<Init<'a>>) -> Result<Self, &str> {
        let bufferlen = from.stage.state.buffer.len();
        let selection = from.stage.state.selection.unwrap_or_else(||(0 as usize, bufferlen));
        let parse_index = | index: &str, selection: usize, default: usize | -> Result<usize, &str> {
            match index {
                "." => Ok(selection),
                "$" => Ok(bufferlen),
                "+" => Ok(selection + 1),
                "-" => Ok(selection - 1),
                _ => { match index.chars().next() {
                    Some('-') => index[1..].parse::<usize>().map(|x| x - 1),
                    Some('+') => index[1..].parse::<usize>().map(|x| x + 1),
                    _ => index.parse::<usize>()
                }.map_err(|_| SELECTION_PARSE_ERR)},
            }
        };
        // Separate out the index of the first command, an the selection str
        let (command_index, string) =
            match find_any_of(VALID_COMMANDS, from.stage.input) {
                Some(x) => Ok((x, &from.stage.input[..x])),
                None => Err(NO_COMMAND_ERR),
            }?;

        // If no string was given we use the default
        let parsed = if string.len() == 0 {
            Ok(selection)
        }
        // Else we parse what is given, if possible
        else {
            // split the parsing based on where there is a ','
            match find_any_of(",;", string) {
                Some(x) => {
                    // If the found one is ';' the default shift
                    // from 1,bufferlen to selection.0,bufferlen
                    let (start, end) = if string.chars().next() == Some(',') {
                        (parse_index(&string[..x], selection.0, 1)?,
                         parse_index(&string[x+1..], selection.1, bufferlen)?)
                    }
                    else {
                        (parse_index(&string[..x], selection.0, 1)?,
                         parse_index(&string[x+1..], selection.1, bufferlen)?)
                    };
                    // Handle assorted edge cases
                    if end > bufferlen {
                        Err(SELECTION_OUT_OF_BOUNDS)
                    }
                    else if start > end {
                        Err(SELECTION_INVERTED)
                    }
                    else {
                        if start == 0 {
                            Ok((start, end))
                        }
                        else {
                            Ok((start - 1, end ))
                        }
                    }
                },
                None => {
                    // If no ',' exists we check if one index was given
                    // It is treated as end
                    let lone = parse_index(string, selection.1, bufferlen)?;
                    // Avoid panics from overshooting the buffer length
                    if lone > bufferlen {
                        Err(SELECTION_OUT_OF_BOUNDS)
                    }
                    else if lone == 0 {
                        Ok((lone, lone))
                    }
                    else {
                        Ok((lone - 1, lone))
                    }
                },
            }
        }?;
        // Build the state for the next stage
        Ok(Self{stage: Command{
            input: &from.stage.input[command_index + 1 ..],
            selection: parsed,
            command:
            from.stage.input[command_index..].chars().next().unwrap(),
            state: from.stage.state,
        }})
    }
}
    //fn parse_expression(self: Self::Expression) -> Self::Flags {

    //}
    // fn parse_flags(self: Self::Flags) -> Self::Done {
    // }

//impl Cmd {
//  /// Execute the command with stored arguments
//  fn execute() -> Result<(), &str> ;
//  /// A simple debug printout seems prudent as well
//  fn debug_print();
//}


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
                if lone > bufferlen {
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
    // marks the index of the previous found separator
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
            #[cfg(feature = "debug")]
            {
                println!("Found separator at index {}.", index + 1);
                println!("Concluding span {:?}",(last_index + 1, index + 1));
            }
            // since we are iterating of a slice shifted one step from
            // the expression slice we add one to all indices
            segments.push(&expression[last_index + 1 .. index + 1]);
            last_index = index + 1;
            i += 1;
            // If we have found enough segments we break
            if i >= nr {
                break;
            }
        }
    }
    // Error if insufficient expression segments were given
    if i < nr {
        Err("Expression too short or incorrectly closed.".to_string())
    }
    else {
        Ok((last_index, segments))
    }
}
fn insert(buffer: &mut Vec<String>, data: &mut Vec<String>,mut index: usize)
          -> Result<(), String> {
    // 0 is valid and should place on first line, others should be shifted
    if index != 0 {
        index -= 1;
    }
    #[cfg(feature = "debug")]
    {
        println!("inserting at index {}", index);
    }
    // To minimise the processing we split the vector
    let mut tail = buffer.split_off(index);
    // And then append both the insert and the split off part
    buffer.append(data);
    buffer.append(&mut tail);
    Ok(())
}
fn delete(buffer: &mut Vec<String>, selection: (usize, usize))
          -> Result<(), String> {
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= buffer.len() {
        let mut tail = buffer.split_off(selection.1);
        let _deleted = buffer.split_off(selection.0);
        buffer.append(&mut tail);
        Ok(())
    }
    else {
        #[cfg(feature = "debug")]
        {
            println!("The selection was {:?}", selection);
            if selection.0 >= selection.1 {
                println!("The selection is empty or inverted");
            }
            if selection.1 > buffer.len() {
                println!("The selection overshoots the buffer.");
            }
        }
        Err("Invalid selection.".to_string())
    }
}
fn change(buffer: &mut Vec<String>,
          data: &mut Vec<String>,
          selection: (usize, usize)
) -> Result<(), String> {
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= buffer.len() {
        let mut tail = buffer.split_off(selection.1);
        let _deleted = buffer.split_off(selection.0);
        buffer.append(data);
        buffer.append(&mut tail);
        Ok(())
    }
    else {
        #[cfg(feature = "debug")]
        {
            println!("The selection was {:?}", selection);
            if selection.0 >= selection.1 {
                println!("The selection is empty or inverted");
            }
            if selection.1 > buffer.len() {
                println!("The selection overshoots the buffer.");
            }
        }
        Err("Invalid selection.".to_string())
    }
}
fn search_replace(buffer: &mut Vec<String>,
                  pattern: (&str, &str),
                  selection: (usize, usize),
                  global: bool,
) -> Result<(), String> {
    use regex::Regex;
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= buffer.len() {
        // Compile the regex used to match/extract data
        let regex = Regex::new(pattern.0).expect("Failed to create pattern regex.");
        if global {
            for index in selection.0 .. selection.1 {
                let after = regex.replace_all(&buffer[index], pattern.1);
                #[cfg(feature = "debug")]
                {
                    print!("Replacing:\n{}\nwith:\n{}",
                           &buffer[index],after
                    );
                }
                buffer[index] = after.to_string();
            }
            Ok(())
        }
        else {
            // Check each line for a match. If found, replace and break
            for index in selection.0 .. selection.1 {
                if regex.is_match(&buffer[index]) {
                    let after = regex.replace(&buffer[index], pattern.1);
                    #[cfg(feature = "debug")]
                    {
                        print!("Replacing:\n{}\nwith:\n{}",
                               &buffer[index],after
                        );
                    }
                    buffer[index] = after.to_string();
                    break;
                }
            }
            Ok(())
        }
    }
    else {
        #[cfg(feature = "debug")]
        {
            println!("The selection was {:?}", selection);
            if selection.0 >= selection.1 {
                println!("The selection is empty or inverted");
            }
            if selection.1 > buffer.len() {
                println!("The selection overshoots the buffer.");
            }
        }
        Err("Invalid selection.".to_string())
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
            'c' => {
                let lines = parse_selection(&command[0..index],
                                            &state.selection,
                                            state.buffer.len()
                )?;
                // Read in the text to change selection into
                let mut data = io::read_insert(&state);
                // Adjust the selection, if affected by delete
                let new_selection = Some((lines.0, lines.0 + data.len()));
                // Insert it into the buffer at instructed line
                change(&mut state.buffer, &mut data, lines)?;
                state.selection = new_selection;
                return Ok(())
            },
            'd' => {
                let lines = parse_selection(&command[0..index],
                                           &state.selection,
                                           state.buffer.len()
                )?;
                // Adjust the selection
                let new_selection = Some((lines.0 + 1, lines.0));
                // Delete the given lines
                delete(&mut state.buffer, lines)?;
                state.selection = new_selection;
                return Ok(())
            },
            's' => {
                let lines = parse_selection(&command[0..index],
                                            &state.selection,
                                            state.buffer.len()
                )?;
                // Adjust the selection
                let new_selection = Some((lines.0, lines.1));
                // Get the regex itself
                let (_expr_end, expr) =
                    parse_expression(&command[index+1..command.len()],
                                     2 as usize,
                )?;
                // Perform the replacement
                search_replace(&mut state.buffer,
                               (expr[0], expr[1]),
                               lines,
                               true
                )?;
                // If no error, set selection and return ok
                state.selection = new_selection;
                return Ok(())
            }
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
