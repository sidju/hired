use crossterm::QueueableCommand;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::Theme;
use std::io::stdout;

// Import the custom syntect theme for 16 color printing
use super::THEME;
use super::SYNTAXES;

// use the UI trait, to implement it
use add_ed::ui::UI;
use add_ed::EdState;
use add_ed::error_consts::*;

mod print;
mod input;

pub struct HighlightingUI {
  syntax_lib: SyntaxSet,
  theme: Theme,
  term_size: (usize, usize),
  command_history: Vec<String>,
}
impl HighlightingUI {
  pub fn new() -> Self {
    let theme: Theme = syntect::dumps::from_binary(THEME);
    let syntax: SyntaxSet = syntect::dumps::from_binary(SYNTAXES);
    Self{
      syntax_lib: syntax,
      theme: theme,
      term_size: crossterm::terminal::size().map(|(a,b)| (a as usize, b as usize)).unwrap_or((80,24)),
      command_history: Vec::new(),
    }
  }
}

use std::io::Write; // Needed for the queue and flush functions on stdout

impl UI for HighlightingUI {
  fn print(
    &mut self,
    _ed: EdState,
    text: &str,
  ) -> Result<(), &'static str> {
    use crossterm::style::Print;
    let mut stdout = stdout();
    stdout.queue(Print(text)).map_err(|_| TERMINAL_WRITE)?;
    stdout.queue(Print("\n\r")).map_err(|_| TERMINAL_WRITE)?;
    stdout.flush().map_err(|_| TERMINAL_WRITE)?;
    Ok(())
  }
  fn get_command(
    &mut self,
    _ed: EdState,
    prefix: Option<char>,
  ) -> Result<String, &'static str> {
    let command = input::event_input(
        self,
        prefix,
        None, // We want one line specifically
      )
        .map_err(|_| TERMINAL_READ)?
        .remove(0)
    ;
    self.command_history.push(command.clone());
    Ok(command)
  }
  fn get_input(
    &mut self,
    _ed: EdState,
    terminator: char,
  ) -> Result<Vec<String>, &'static str> {
    input::event_input(
      self,
      None, // No line prefix for input
      Some(terminator)
    )
      .map_err(|_| TERMINAL_READ)
  }
  fn print_selection(
    &mut self,
    ed: EdState,
    selection: (usize, usize),
    numbered: bool,
    literal: bool,
  ) -> Result<(), &'static str> {
    // First we get the data needed to call the internal function
    let mut iter = ed.buffer.get_selection(selection)?;
    let syntax = self.syntax_lib.find_syntax_for_file(ed.path)
      .unwrap_or(None)
      .unwrap_or_else(|| self.syntax_lib.find_syntax_plain_text());
    // Then we call the internal print
    print::internal_print(
      &self,
      &syntax,
      &mut iter,
    // The optional inputs are for prefix on start of each line and
    // returning the position of a particular character in the terminal print respectively
      None,
      None,
      selection.0,
      numbered,
      literal,
    ).map_err(|_| TERMINAL_WRITE)?;
    Ok(())
  }
}
