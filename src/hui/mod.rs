use crossterm::QueueableCommand;
use two_face::re_exports::syntect::parsing::SyntaxSet;
use two_face::re_exports::syntect::highlighting::Theme;
use std::io::stdout;

// use the UI trait, to implement it
use add_ed::ui::{
  UI,
  UILock,
};
use add_ed::{
  Ed,
  error::{
    Result,
    EdError,
  },
};

mod print;
mod doc_print;
mod input;
pub mod error;
use error::HighlightingUIError as HUIError;

pub struct HighlightingUI {
  syntax_lib: SyntaxSet,
  theme: Theme,
  term_size: (usize, usize),
  command_history: Vec<String>,
}
impl HighlightingUI {
  pub fn new() -> Self {
    let theme: Theme = two_face::theme::extra().get(two_face::theme::EmbeddedThemeName::Base16).clone();
    let syntax: SyntaxSet = two_face::syntax::extra_newlines();
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
  fn print_message(
    &mut self,
    text: &str,
  ) -> Result<()> {
    (|| -> std::io::Result<()> {
      use crossterm::style::Print;
      let mut stdout = stdout();
      if crossterm::cursor::position()?.0 != 0 {
        stdout.queue(Print("\n\r"))?;
      }
      for line in text.lines() {
        stdout.queue(Print(line))?;
        stdout.queue(Print("\n\r"))?;
      }
      stdout.flush()?;
      Ok(())
    })()
      .map_err(HUIError::TerminalIOFailed)
      .map_err(add_ed::error::UIError::from)
      .map_err(EdError::UI)
  }
  fn print_commands(&mut self) -> Result<()> {
    doc_print::display_doc(add_ed::messages::COMMAND_LIST.into())
      .map_err(HUIError::from_termimad)
      .map_err(add_ed::error::UIError::from)
      .map_err(EdError::UI)
  }
  fn print_command_documentation(&mut self) -> Result<()> {
    doc_print::display_doc(add_ed::messages::COMMAND_DOCUMENTATION.into())
      .map_err(HUIError::from_termimad)
      .map_err(add_ed::error::UIError::from)
      .map_err(EdError::UI)
  }
  fn get_command(
    &mut self,
    _ed: &Ed,
    prefix: Option<char>,
  ) -> Result<String> {
    let command = input::event_input(
        self,
        Vec::new(),
        prefix,
        None, // We want one line specifically
      )
        .map_err(|e|add_ed::EdError::UI(e.into()))?
        .remove(0)
    ;
    self.command_history.push(command.clone());
    Ok(command)
  }
  fn get_input(
    &mut self,
    _ed: &Ed,
    terminator: char,
    initial_buffer: Option<Vec<String>>,
  ) -> Result<Vec<String>> {
    input::event_input(
      self,
      initial_buffer.unwrap_or(Vec::new()),
      None, // No line prefix for input
      Some(terminator)
    )
      .map_err(|e|add_ed::EdError::UI(e.into()))
  }
  fn print_selection(
    &mut self,
    ed: &Ed,
    selection: (usize, usize),
    numbered: bool,
    literal: bool,
  ) -> Result<()> {
    // First we get the data needed to call the internal function
    let mut iter = ed.history.current().get_tagged_lines(selection)?;
    let syntax = self.syntax_lib.find_syntax_for_file(&ed.file)
      .unwrap_or(None)
      .unwrap_or_else(|| self.syntax_lib.find_syntax_plain_text());
    // Then we call the internal print
    print::internal_print(
      &self,
      &syntax,
      &mut iter,
      print::PrintConf {
        prefix: None,
        cursor: None,
        start_line: selection.0,
        numbered: numbered,
        literal: literal,
        separator: true,
      },
    )
      .map_err(HUIError::TerminalIOFailed)
      .map_err(add_ed::error::UIError::from)
    ?;
    Ok(())
  }
  fn lock_ui(&mut self) -> UILock {
    // Before handing over to shell escaped commands we need to disable raw mode
    crossterm::terminal::disable_raw_mode()
      .map_err(HUIError::RawmodeSwitchFailed)
      .unwrap()
    ;
    UILock::new(self)
  }
  fn unlock_ui(&mut self) {
    // Re-enable raw mode, to go back to using the UI
    crossterm::terminal::enable_raw_mode()
      .map_err(HUIError::RawmodeSwitchFailed)
      .unwrap()
    ;
  }
}
