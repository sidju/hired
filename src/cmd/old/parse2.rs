/// A rewritten parser taking greater liberties with redefining the syntax
/// It has a strict separation, only parsing the input without state data

use enum_from_str_derive::FromStr;
use enum_from_str::ParseEnumVariantError;

// Consts for selection parsing
const INDEX_PARSE_ERR: &str = "Could not parse given index.";

// Consts for parsing commands
const NO_COMMAND_ERR: &str = "No valid command given.";
const POSSIBLE_COMMANDS: &str =
    "abcdefthijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

// Utility function needed a few times while parsing
fn find_any_of(options: &str, input: &str) -> Option<usize> {
  for (index, ch) in input.char_indices() {
    if options.chars().any(|cmd| cmd == ch) {
      return Some(index);
    }
  }
  None
}

pub enum Ind {
  Default,
  BufferLen,
  SelectionRelative(i32),
  Literal(usize),
}
// Parse out an index
fn parse_index(index: &str)
  -> Result<Ind, &'static str>
{
  if index.len() == 0 {
    Ok(Ind::Default)
  }
  else {
    match index {
      "." => Ok(Ind::SelectionRelative(0)),
      "$" => Ok(Ind::Bufferlen),
      "+" => Ok(Ind::SelectionRelative(1)),
      "-" => Ok(Ind::SelectionRelative(-1)),
      _ => { match index.chars().next() {
        Some('-') => index[..].parse::<i32>().map(|x| Ind::SelectionRelative(x) ),
        Some('+') => index[..].parse::<i32>().map(|x| Ind::SelectionRelative(x) ),
        _ => index.parse::<usize>().map(|x| Ind::Literal(x) )
      }.map_err(|_| INDEX_PARSE_ERR)},
    }
  }
}
pub enum Sel {
  FromStart(start: Ind, end: Ind),
  FromSelection(start: Ind, end: Ind),
  Lone(Ind)
}
fn parse_selection(selection: &str)
  -> Result<Option<Sel>, &'static str>
{
  if selection.len() == 0 {
    Ok(None)
  }
  else {
    match find_any_of(",;", selection) {
      Some(i) => {
        // If we find a separator we parse the separated indices
        let (mut start, end) = (
          parse_index(&selection[..i])?,
          parse_index(&selection[i+1..])?
        );
        if &selection[i..i+1] == "," {
          Ok(Sel::FromStart(start: start, end: end))
        }
        else {
          Ok(Sel::FromSelection(start: start, end: end))
        }
      }
      None => {
        // If no separator, parse as single line
        Ok(Sel::Lone(parse_index(selection)?))
      }
    }
  }
}
fn parse_arguments(arguments: &str)
  -> Result<Vec<String>, &'static str>
{
  // Track where in the input string we are parsing
  let mut index = 0;
  let mut retbuf = Vec::new();
  if arguments.len() > 0 {
    // Keep track of the separator
    // We can unwrap since we check len above
    let separator = arguments.chars().next().unwrap();
    // While we can find separators we use them to split the input
    while let Some(sepi) = arguments.find(separator) {
      retbuf.push(arguments[index+1..sepi].to_string());
      index = sepi + 1;
    }
  }
  // Push the last (potentially only) argument
  retbuf.push(arguments[index..].to_string());
  Ok(retbuf)
}
#[derive(FromStr)]
#[allow(non_camel_case_types)]
pub enum Command {
  q, // Quit
  Q, // Quit ignoring unsaved changes
  h, // Print last error
//  H, // Toggle printing errors as they occur
//  u, // Revert buffer one command back

  p, // Print the defined lines in defined mode

  i, // Insert given text at given index
  d, // Delete the selection
  c, // Replace selection with given text

  m, // Moves selection to given index
  t, // Copy the selection to given index
  j, // Join the selection into one line

//  y, // Cut selection from buffer
//  x, // Append what was last cut to selection

//  g, // Apply the given commands to lines matching expr.
  s, // Regex search and replace over selection

  F, // Open given file into new buffer ignoring unsaved
  f, // Open given file into new buffer
  r, // Append contents of given file to selection
  w, // Write the selection to given file
  W, // Append the selection to given file
}
fn parse_command(command: &str) -> Result<Command, &'static str> {
  command[..1].parse()
    .map_err(|_|UNDEFINED_COMMAND)
}

pub struct Cmd <'a>{
  state: &'a mut crate::State,
  selection: (usize, usize),
  command: Command,
  arguments: Vec<String>,
}


pub fn parse<'a>(
  state: &'a mut crate::State,
  input: &'a mut str,
) -> Result<Cmd<'a>, &'static str> {
  // Get the current length of the buffer, since multiple steps use it
  let bufferlen = state.buffer.len();

  // Find the command index by finding a letter
  let cindex = find_any_of(POSSIBLE_COMMANDS, input).ok_or(NO_COMMAND_ERR)?;

  // When we know the index of the command we can parse the selection
  let selection = parse_selection(&input[..cindex], bufferlen)?;
  #[cfg(feature = "debug")]
  { println!("Selection parsed from \"{}\" to {:?}", &input[0..cindex], selection); }

  // Get the enum for the command
  let command = parse_command(&input[cindex..])?;

  // We split the remaining input into arguments using the first character as separator
  // This gives a vector of strings where the last is the flags if any
  let mut arguments = parse_arguments(&input[cindex+1..])?;

  Ok(Cmd{
    state: state,
    selection: selection,
    command: command,
    arguments: arguments,
  })
}
