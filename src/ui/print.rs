/// UI printing abstractions
use crate::State;
use crate::error_consts::*;

use crossterm::{QueueableCommand, ErrorKind, style::Print};
use std::io::Write;

// A small println replacement for raw mode. Uses crossterm print commands and the state's stdout.
pub fn println(out: &mut impl Write, text: &str) {
  out.queue(Print(text)).expect(TERMINAL_WRITE);
  out.queue(Print("\n\r")).expect(TERMINAL_WRITE);
  out.flush().expect(TERMINAL_WRITE);
}
pub fn print(out: &mut impl Write, text: &str) {
  out.queue(Print(text)).expect(TERMINAL_WRITE);
  out.flush().expect(TERMINAL_WRITE);
}

// Start with the printing helpers

fn syntect_to_crossterm_color(c: syntect::highlighting::Color)
  -> crossterm::style::Color
{
  use crossterm::style::Color;
  // If the alpha is zero, read as 16 color
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

// Set the style given, all parts explicitly set to given style
fn apply_style(style: syntect::highlighting::Style, out: &mut impl Write)
  -> Result<(), ErrorKind>
{
  use syntect::highlighting::FontStyle;
  use crossterm::style::{SetColors, SetAttribute, Colors, Attribute};

  // Prepare and apply colors
  let colors = Colors::new(
    syntect_to_crossterm_color(style.foreground),
    syntect_to_crossterm_color(style.background)
  );
  out.queue(SetColors(colors))?;
  
  // Prepare and apply styling
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
// Resets the style to the default for UI components
// Should arguably be updated to follow the theme (For white background people)
fn reset_style(out: &mut impl Write)
  -> Result<(), ErrorKind>
{
  use crossterm::style::{ResetColor, SetAttribute, Attribute};
  // Reset colors
  out.queue(ResetColor)?;
  // Reset attributes
  out.queue(SetAttribute(Attribute::Reset))?;
  Ok(())
}

// Creates a horizontal separator with the terminals default colors
fn print_separator(out: &mut impl Write, width: usize)
  -> Result<(), ErrorKind>
{
  // Create the string to hold the separator with the capacity we know it will use
  let mut sep = String::with_capacity(width);
  for _ in 0 .. width {
    sep.push('─');
  }
  sep.push('\n');
  sep.push('\r');
  // Print the generated separator
  out.queue(Print(sep))?;
  Ok(())
}

//// Appends blankspaces to pad from given index to given width
//fn pad_line(out: &mut impl Write, width: usize, index: usize)
//  -> Result<(), ErrorKind>
//{
//  // Get the position the index represents
//  let pos = index % width;
//  // Check if we really need to pad or if index is 0
//  if pos != 0 {
//    let mut pad = String::with_capacity(2 + width - pos);
//    for _ in pos .. width {
//      pad.push(' ');
//    }
//    out.queue(Print(pad))?;
//  }
//  Ok(())
//}

// Wrapper that adjusts the error type (loosing some data, though)
pub fn format_print(
  state: &State,
  text: &[String],
  line_nr: usize,
  as_view: bool,
  n: bool,
  l: bool,
) -> Result<(), &'static str> {
  format_print_internal(state, text, line_nr, as_view, n, l)
    .map_err(|_| "Error occured while printing.")
}

// The big function of this file. Prints the given lines with highligting.
// Needs a wrapper since Crossterm's error type made most sense in it
fn format_print_internal(
  state: &State,
  //text: &[&str],
  text: &[String],
  mut line_nr: usize,
  as_view: bool, // If we are to be limited by terminal height
  n: bool,
  l: bool,
) -> Result<(), ErrorKind> {
  // Get the highlighting settings
  let theme = &state.theme;
  let syntax = state.syntax_lib.find_syntax_for_file(&state.file)
    .unwrap_or(None)
    .unwrap_or_else(|| state.syntax_lib.find_syntax_plain_text());
  // Create the highlighter, which statefully styles the text over lines.
  let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);

  // Create a connection to the terminal, that we print through
  let mut out = &state.stdout;

  // Track lines printed, to break when we have printed the terminal height
  let mut lines_printed = 0;
  // Count characters printed, for wrapping. Use 'i' since everything uses it
  let mut i = 0;

  // Arguably one should give the highlighter all lines before the selection.
  // Otherwise it fails to understand multiline stuff, but not worth it to me. 
  // PR's welcome
  'lines: for line in text {
    // To handle wrapping, print character by character

    // Highlight the text.
    let highlighted = highlighter.highlight(line, &state.syntax_lib);

    // Iterate over the segments, setting style before each
    for (style, text) in highlighted {
      apply_style(style, &mut out)?;

      for ch in text.chars() {

        // Print line numbers, if active
        if n && (i % state.term_size.0 == 0) {
          // To not colour our numbering we reset styling for this
          reset_style(&mut out)?;

          // Then we convert our 0-indexed int to a 1 indexed string
          let tmp_num = (line_nr + 1).to_string();
          let tmp_num_len = tmp_num.len();

          // If this is a new text line, print line number
          if i == 0 {
            out.queue(Print(tmp_num))?;
            line_nr += 1;
          }
          // If only a new terminal line, print a neat border
          else {
            for _ in 0 .. tmp_num_len { out.queue(Print(' '))?; }
          }
          // Print the separator and mark that we added chars to the line
          out.queue(Print('│'))?;
          i += tmp_num_len + 1;
          // And finally restore the styling
          apply_style(style, &mut out)?;
        }

        // Next we print the character in question
        match ch {
          '\n' => {
            // If literal mode, also print $
            if l { out.queue(Print('$'))?; /* i += 1; */ }
            // This primarily means we reset i, since a new line is created
            // but that requires the following cleanup
            //pad_line(&mut out, state.term_size.0, i);
            i = 0;
          },
          '$' => if l {
            out.queue(Print("\\$"))?;
            i += 2;
          } else {
            out.queue(Print('$'))?;
            i += 1;
          },
          c => {
            out.queue(Print(c))?;
            i += 1;
          },
        }

        // Then we check if we need to move to a new line
        if i % state.term_size.0 == 0 {
          out.queue(Print("\n\r"))?;

          // So we increment lines printed and check if done
          lines_printed += 1;
          if as_view && lines_printed + 3 >= state.term_size.1 {break 'lines;}
        }
      }
    }
  }
  // Pad and terminate the last line
  reset_style(&mut out)?;
  print_separator(&mut out, state.term_size.0)?;
  // Finally we flush the buffer, to make sure we actually have printed everything
  out.flush()?;
  Ok(())
}

// A print function that prints the current state of the buffer over the previous state
// Moves the cursor to the given position after printing
// Moves 'topdist' lines up before starting printing
// Returns cursor's distance to top and bottom, respectively
pub fn print_input(
  state: &mut crate::State,
  buffer: &Vec<String>,
  lindex: usize,
  chindex: usize,
  topdist: u16,
  command: bool,
)
  -> Result<(u16, u16), crossterm::ErrorKind>
{
  // First go to the top of the previous input
  if topdist != 0 {
    state.stdout.queue(crossterm::cursor::MoveUp(topdist))?;
  }
  state.stdout.queue(crossterm::cursor::MoveToColumn(0))?;
  // And clear all of the previous print
  state.stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown))?;

  // A bool to track if we have passed our current cursor index, to find its position
  let mut passed = false;

  // To track the position of the cursor, by when we passed it in the buffer
  let mut x: u16 = 0;
  let mut y: u16 = 0;
  // And the height of the print, for returning distance to top and bottom
  let mut height: u16 = 0;

  // Then start looping over the buffer
  for (linenr, line) in buffer.iter().enumerate() {

    // Create a character to track nr characters printed this line
    let mut chars_printed = 0;

    // If this is a command we print : (or whatever specified) to signify this
    if command {
      state.stdout.queue(Print(&state.prompt))?;
      chars_printed += state.prompt.chars().count();
    }

    for (i, ch) in line.char_indices() {
      // If wrapping or starting a new line increment lines printed
      // exceptions to this are the first line, which should be cleared by a preceeding newline.
      if (chars_printed % state.term_size.0) == 0 && linenr > 0 {
        // Print newline and carriage return
        state.stdout.queue(Print("\n\r"))?;
        // Increment height related variables
        height += 1;
        if passed { y += 1; }
      }

      // If we haven't reached our current cursor position before, check if we have now.
      // This by nesting if not found, if lindex == line_i, if chindex == i
      if ! passed {
        if lindex <= linenr && chindex <= i {
          // Set the x coordinate using chars_printed modulo terminal width
          x = (chars_printed % state.term_size.0) as u16;
          // And mark chindex as passed
          passed = true;
        }
      }

      // Print the current character (unless newline or carriage return)
      if ch != '\n' && ch != '\r' {
        // Increment the number of characters printed
        chars_printed += 1;
        // Finally, print the character
        state.stdout.queue(Print(ch))?;
      }
    } // End of chars
  } // End of lines

  // When done with looping, move the cursor to the calculated coordinates
  state.stdout.queue(crossterm::cursor::MoveToColumn(x + 1))?;
  if y != 0 {
    state.stdout.queue(crossterm::cursor::MoveUp(y))?;
  }
  state.stdout.flush()?;

  // Finally we return the distances
  Ok((height - y, y))
}
