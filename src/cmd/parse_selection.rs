/// Since default selections vary between commands and some of them need to know
/// if anything was entered we parse to an intermediary struct that is then
/// interpreted using data from the command.

use crate::error_consts::*;

#[derive(PartialEq, Debug)]
pub enum Ind {
  Default,
  BufferLen,
  Relative(i32),
  Literal(usize),
}
#[derive(PartialEq, Debug)]
pub enum Sel {
  FromStart(Ind, Ind),
  FromSelection(Ind, Ind),
  Lone(Ind),
}

pub fn parse_index(index: &str)
  -> Result<Ind, &'static str>
{
  if index.len() == 0 {
    Ok(Ind::Default)
  }
  else {
    match index {
      "." => Ok(Ind::Relative(0)),
      "$" => Ok(Ind::BufferLen),
      "+" => Ok(Ind::Relative(1)),
      "-" => Ok(Ind::Relative(-1)),
      _ => { match index.chars().next() {
        Some('-')|Some('+') => index[..].parse::<i32>().map(|x| Ind::Relative(x) ),
        _ => index.parse::<usize>().map(|x| Ind::Literal(x) ),
      }.map_err(|_| INDEX_PARSE)},
    }
  }
}

// Returns the index of the command char and the parsed selection
pub fn parse_selection(input: &str)
  -> Result<(usize, Sel), &'static str>
{
  // Variables set in the loop
  let mut sep_i = None;

  for (i, char) in input.char_indices() {
    // Find the separator, if any
    if char == ',' || char == ';' {
      if sep_i != None { return Err(INDEX_PARSE); } // Multiple separators given
      // Save index and which separator it is
      sep_i = Some((i, char));
    }
    // When we reach the command, use the collected data to parse
    else if char.is_ascii_alphabetic() | (char == '\n') | (char == '?') {
      let sel = match sep_i {
        Some((si, sep)) => {
          // Means we parse the indices separately
          let sel_start = parse_index(&input[..si])?;
          let sel_end = parse_index(&input[si + 1..i])?;
          // Convert from inclusive 1 indexed to exclusive 0 indexed
          let sel_start = match sel_start {
            Ind::Literal(x) => { Ind::Literal(x - ( 1 * (x != 0) as usize ) ) }, // subtract 1 if not 0
            i => i,
          };
          if sep == ',' {
            Sel::FromStart(sel_start, sel_end)
          }
          else {
            Sel::FromSelection(sel_start, sel_end)
          }
        }
        None => {
          // Means we parse a lone index 
          Sel::Lone(parse_index(&input[..i])?)
        }
      };
      // Return selection and index of the command char
      return Ok((i, sel));
    }
  }
  Err(NO_COMMAND)
}

pub fn u_i_add(a: usize, b: i32)
 -> usize
{
  if b.is_negative() {
    a.saturating_sub((b * -1) as usize)
  }
  else {
    a.saturating_add(b as usize)
  }
}

pub fn interpret_index(
  index: Ind,
  old_selection: Option<usize>,
  bufferlen: usize,
  default: usize,
)
  -> usize
{
  let sel = old_selection.unwrap_or(default);
  match index {
    Ind::Default => default,
    Ind::BufferLen => bufferlen,
    Ind::Relative(x) => u_i_add(sel, x),
    Ind::Literal(x) => x,
  }
}

pub fn interpret_selection(
  selection: Sel,
  old_selection: Option<(usize, usize)>,
  bufferlen: usize,
  default_all: bool,
)
  -> (usize, usize)
{
  let old_sel = old_selection.unwrap_or((0, bufferlen));
  match selection {
    Sel::Lone(i) => {
      match i {
        Ind::Default => {
          if default_all { (0, bufferlen) }
          else { old_sel }
        }
        Ind::BufferLen => (bufferlen - 1, bufferlen),
        Ind::Relative(x) => (u_i_add(old_sel.0, x), u_i_add(old_sel.1, x)),
        Ind::Literal(x) => (x - (1 * (x != 0) as usize), x)
      }
    }
    Sel::FromSelection(i, j) => {
      let start = match i {
        Ind::Default => old_sel.0,
        Ind::BufferLen => bufferlen,
        Ind::Relative(x) => u_i_add(old_sel.0, x),
        Ind::Literal(x) => x,
      };
      let end = match j {
        Ind::Default => bufferlen,
        Ind::BufferLen => bufferlen,
        Ind::Relative(x) => u_i_add(old_sel.1, x),
        Ind::Literal(x) => x,
      };
      (start, end)
    }
    Sel::FromStart(i, j) => {
      let start = match i {
        Ind::Default => 0,
        Ind::BufferLen => bufferlen,
        Ind::Relative(x) => u_i_add(old_sel.0, x),
        Ind::Literal(x) => x,
      };
      let end = match j {
        Ind::Default => bufferlen,
        Ind::BufferLen => bufferlen,
        Ind::Relative(x) => u_i_add(old_sel.1, x),
        Ind::Literal(x) => x,
      };
      (start, end)
    }
  }
}
