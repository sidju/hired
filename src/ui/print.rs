/// UI printing abstractions
use crate::State;

use crossterm::{QueueableCommand, ErrorKind, style::Print};
use std::io::{Write, stdout};

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
  // Print the generated separator
  out.queue(Print(sep))?;
  Ok(())
}

// Appends blankspaces to pad from given index to given width
fn pad_line(out: &mut impl Write, width: usize, index: usize)
  -> Result<(), ErrorKind>
{
  // Get the position the index represents
  let pos = index % width;
  // Check if we really need to pad or if index is 0
  if pos != 0 {
    let mut pad = String::with_capacity(2 + width - pos);
    for _ in pos .. width {
      pad.push(' ');
    }
    out.queue(Print(pad))?;
  }
  Ok(())
}
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
  let mut out = stdout();

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
            if l { out.queue(Print('$'))?; i += 1; }
            // This primarily means we reset i, since a new line is created
            // but that requires the following cleanup
            pad_line(&mut out, state.term_size.0, i);
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
          out.queue(Print('\n'))?;

          // So we increment lines printed and check if done
          lines_printed += 1;
          if as_view && lines_printed + 2 >= state.term_size.1 {break 'lines;}
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
