// Get the prerequisite definitions for writing these functions
use super::HighlightingUI;

use crossterm::{
  QueueableCommand,
  ErrorKind,
  style::{
    Print,
    Color,
  }
};
use std::io::Write; // Needs to be used in for queue and flush

// Create some printing helpers
fn syntect_to_crossterm_color(
  c: syntect::highlighting::Color,
) -> Color {
  // If alpha value is zero the red value is which 16 color to use
  if c.a == 0 {
    match c.r {
      0 => Color::Black,
      1 => Color::DarkRed,
      2 => Color::DarkGreen,
      3 => Color::DarkYellow,
      4 => Color::DarkBlue,
      5 => Color::DarkMagenta,
      6 => Color::DarkCyan,
      7 => Color::Grey,

      8 => Color::DarkGrey,
      9 => Color::Red,
      10 => Color::Green,
      11 => Color::Yellow,
      12 => Color::Blue,
      13 => Color::Magenta,
      14 => Color::Cyan,
      15 => Color::White,

      _ => panic!("Invalid theme. Alpha = 0 indicates 16 color in red."),
    }
  }
  else {
    Color::Rgb{r: c.r, g: c.g, b: c.b}
  }
}
fn apply_style(
  style: syntect::highlighting::Style,
  out: &mut impl Write,
) -> Result<(), ErrorKind> {
  use syntect::highlighting::FontStyle;
  use crossterm::style::{SetColors, SetAttribute, Colors, Attribute};

  // Prepare and apply colors
  let colors = Colors::new(
    syntect_to_crossterm_color(style.foreground),
    syntect_to_crossterm_color(style.background)
  );
  out.queue(SetColors(colors))?;

  // Interpret and apply styling
  if style.font_style.contains(FontStyle::BOLD) {
    out.queue(SetAttribute(Attribute::Bold))?;
  }
  if style.font_style.contains(FontStyle::ITALIC) {
    out.queue(SetAttribute(Attribute::Italic))?;
  }
  if style.font_style.contains(FontStyle::UNDERLINE) {
    out.queue(SetAttribute(Attribute::Underlined))?;
  }
  Ok(())
}
fn reset_style(out: &mut impl Write) -> Result<(), ErrorKind> {
  use crossterm::style::{ResetColor, SetAttribute, Attribute};
  out.queue(ResetColor)?;
  out.queue(SetAttribute(Attribute::Reset))?;
  Ok(())
}
fn print_separator(
  out: &mut impl Write,
  width: usize,
) -> Result<(), ErrorKind> {
  let mut sep = String::with_capacity(width);
  let mut skip = 0;
  for i in 0 .. width {
    if i % 20 == 0 {
      let num = i.to_string();
      if i + num.len() < width {
        skip = num.len() - 1; // -1 since we skipped one by going here
        sep.push_str(&num);
      } else {
        sep.push('-');
      }
    }
    else if skip > 0 {
      skip -= 1;
    }
    else {
      sep.push('-');
    }
  }
  sep.push('\n');
  sep.push('\r');
  out.queue(Print(sep))?;
  Ok(())
}

// Create a struct to return which clarifies what is returned
pub struct PrintData {
  // Total height of the print
  pub height: u16,
  // The position of the cursor (relative bottom left)
  pub cursor_x: u16,
  pub cursor_y: u16,
}

// Create a struct to define print settings
pub struct PrintConf {
  // Print prefix char at start of every line, before numbering if any
  // Intended to support prefix at command input
  pub prefix: Option<char>,
  // Position (x,y in text) to leave cursor at
  // Intended for when printing an actively edited buffer
  pub cursor: Option<(usize, usize)>,
  // Index in iterator from which to print
  // Intended to feed syntax highlighter with preceding lines without printing them
  pub start_line: usize,
  // If true print line number at start of every line
  pub numbered: bool,
  // If true print like 'ed's literal print mode
  pub literal: bool,
  // If true print a separator before the given text
  pub separator: bool,
}

// Uses state to print the given iterator with given syntax highlighting
pub fn internal_print(
  state: &HighlightingUI,
  syntax: &syntect::parsing::SyntaxReference,
  text: &mut dyn Iterator<Item = (char, &str)>,
  conf: PrintConf,
) -> Result<PrintData, ErrorKind> {
  let mut stdout = std::io::stdout();

  let theme = &state.theme;
  let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);

  // Variables for tracking cursor positions
  // i is used for width to detect when we need to wrap lines over to next line
  let mut i = 0;
  // print height is returned in case we wish to overwrite this printout later
  let mut print_height: u16 = 0;
  // x, y and passed are for returning the terminal position of an optional buffer
  // position, if such was given. Always 0,0,false if not given or not found.
  let mut passed = false;
  let mut x: u16 = 0;
  let mut y: u16 = 0;

  if conf.separator {
    // Print a separator from whatever came before
    // potentially add more info to it later
    print_separator(&mut stdout, state.term_size.0)?;
    print_height += 1;
  }

  // Arguably one should give the highlighter all lines before the selection.
  // Otherwise it fails to understand multiline stuff over the selection edges.
  // Currently too minor for me to bother, PRs welcome
  for (linenr, line) in text.enumerate() {

    // For each new line the byte index starts at 0
    // Used for tracking the offsets of the characters as in a string
    // Needed to understand the cursor which gives byte offsets
    let mut byte_index = 0;

    // Highlight the line first
    let highlighted = highlighter.highlight_line(line.1, &state.syntax_lib)
      .unwrap(); // TODO: this should be handled, requires change of error type
    // Iterate over syntactic segments, setting the style for each
    for (style, text) in highlighted {
      apply_style(style, &mut stdout)?;
      for ch in text.chars() {

        // If prefix is given, print at start of real but not wrapped lines
        if let Some(pre) = conf.prefix {
          if i == 0 {
            reset_style(&mut stdout)?;
            let pre_len = pre.len_utf8();
            stdout.queue(Print(pre))?;
            i += pre_len;
            apply_style(style, &mut stdout)?;
          }
        }

        // If line numbers are active, check if start of line
        if conf.numbered && (i % state.term_size.0 == 0) {
          reset_style(&mut stdout)?;
          // Calculate number and convert to string
          let tmp_num = (conf.start_line + linenr).to_string();
          let tmp_num_len = tmp_num.len(); // Only works because linenr is ascii
          // If this is a new line, print number
          if i == 0 {
            // If no line tag, print number
            if line.0 == '\0' {
              stdout.queue(Print(tmp_num))?;
            }
            // Else print the tag instead 
            else {
              stdout.queue(Print(line.0))?;
              for _ in 1 .. tmp_num_len { stdout.queue(Print(' '))?; }
            }
          }
          // If a wrapped line, print inwards offset equal to the numbering
          else {
            for _ in 0 .. tmp_num_len { stdout.queue(Print(' '))?; }
          }
          // Print a separator and mark how many chars we printed
          stdout.queue(Print('│'))?;
          i += tmp_num_len + 1; // +1 for the separator
          // Finally we MUST restore the styling
          apply_style(style, &mut stdout)?;
        }

        // After printing potential prefixes we check againts our given cursor, if given
        // We must check before printing ch, since printing newline resets i
        // Specifically we check if the cursor is before the current ch
        if let Some(cur) = conf.cursor {
          if ! passed {
            if (cur.0 == linenr && cur.1 <= byte_index) || cur.0 < linenr {
              // This all means we have passed by the given cursor for the first time
              // Due to needing to place the cursor one step down in that case we specially handle '\n'
              // Calculate current column and save in x
              x = (i % state.term_size.0) as u16 + 1;
              // Mark that we have passed, this will increment y for each new line started
              passed = true;
            }
            // For each char while not passed add their len to byte-index
            // Add after checking, since we otherwise cannot go to char index 0
            byte_index += ch.len_utf8();
          }
        }

        // Print the actual character
        // If literal mode, handle edge cases
        match ch {
          '\n' => {
            if conf.literal { stdout.queue(Print('$'))?; }
            i = 0;
          },
          '$' => if conf.literal {
            stdout.queue(Print("\\$"))?;
            i += 2;
          } else {
            stdout.queue(Print('$'))?;
            i += 1;
          },
          '\t' => {
            if conf.literal { stdout.queue(Print("--->"))?; }
            else { stdout.queue(Print("    "))?; }
            i += 4;
          },
          c => {
            stdout.queue(Print(c))?;
            i += 1;
          },
        }

        // Check if a new line is needed, aka. newline or wrapping
        if i % state.term_size.0 == 0 {
          stdout.queue(Print("\n\r"))?;
          print_height += 1;
          if passed { y += 1; }
        }
      }
    }
  }
  // Closing cleanup and flush
  reset_style(&mut stdout)?;
  // Note that this increases height and y
  stdout.flush()?;
  Ok(PrintData{
    height: print_height,
    cursor_x: x,
    cursor_y: y,
  })
}
