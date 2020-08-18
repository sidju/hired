use std::collections::HashMap;
use crate::error_consts::*;

/// Takes an input string and a map of flags to check for presence of.
/// If a char in the input doesn't already exist in the map it errors.
/// A typical call will look like
/// let flags = parse_flags(&input, [('p', false), ('n', false), ('l', false)]).iter().cloned().collect()?;
pub fn parse_flags(input: &str, mut flag_map: HashMap<char, bool>)
  -> Result<HashMap<char, bool>, &'static str>
{
  for char in input.trim().chars() {
    match flag_map.get_mut(&char) {
      Some(b) => {
        if !(*b) { *b = true; Ok(()) }
        else { Err(DUPLICATE_FLAG) }
      },
      None => Err(UNDEFINED_FLAG),
    }?
  }
  Ok(flag_map)
}
