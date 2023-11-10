use crossterm::{
  event::{
    self,
    Event,
    KeyEvent,
  },
  terminal::{
    Clear,
    ClearType,
    EnterAlternateScreen,
    LeaveAlternateScreen,
  },
  cursor::{
    Hide,
    Show,
  },
  queue,
};
use termimad::{
  Area,
  MadView,
  MadSkin,
  Error,
};
use std::io::Write;

fn view_area() -> Area {
  let mut a = Area::full_screen();
  a.pad_for_max_width(80);
  a
}

pub fn display_doc(
  doc: &str,
) -> Result<(), Error> {
  // The mangling didn't work, rolling back to this
  let mangled = join_joinable_lines(doc);
  let mut w = std::io::stdout();
  queue!(w, EnterAlternateScreen)?;
  queue!(w, Hide)?; // Cursor
  let res = (||{
    let mut view = MadView::from(mangled, view_area(), MadSkin::default_dark());
    // Event loop while printing
    loop {
      // Print (with scrollbar)
      view.write_on(&mut w)?;
      w.flush()?;
      // Get input and react to it
      match event::read() {
        Ok(Event::Key(KeyEvent{code, ..})) => {
          use crossterm::event::KeyCode as KC;
          match code {
            KC::Up => view.try_scroll_lines(-1),
            KC::Down | KC::Enter => view.try_scroll_lines(1),
            KC::PageUp => view.try_scroll_pages(-1),
            KC::PageDown | KC::Char(' ') => view.try_scroll_pages(1),
            _ => break,
          }
        },
        Ok(Event::Resize(..)) => {
          queue!(w, Clear(ClearType::All))?;
          view.resize(&view_area());
        },
        _ => {},
      }
    }
    Ok(())
  })();
  queue!(w, Show)?; // Restore cursor
  queue!(w, LeaveAlternateScreen)?;
  w.flush()?;
  res
}

#[derive(Debug)]
struct State {
  pub output: String,
  pub remove_newlines: bool,
  pub in_a_codeblock: bool,
}
impl State {
  fn new() -> Self {
    Self{
      output: String::new(),
      remove_newlines: true,
      in_a_codeblock: false,
    }
  }
  // We use this method to handle where a line affects state for the next, ie.
  // explicit newlines
  fn add_line(
    &mut self,
    line: &str,
  ) {
    // Explicit newline
    if line.ends_with("  ") {
      self.remove_newlines = false;
      // Remove all the trailing spaces
      self.output.push_str(line.trim_end_matches(' '));
    }
    else if line.ends_with('\\') {
      self.remove_newlines = false;
      self.output.push_str(line);
      // pop off the '\\' before going on
      self.output.pop();
    }
    // Normal line
    else {
      self.output.push_str(line);
    };
    // Finally we add the newline after the text, always.
    self.output.push('\n');
  }
}

// Does line joining according to markdown syntax. Ie. normal newlines become
// blankspaces, unless otherwise indicated.
// (Currently doesn't do any line joining within block-quotes, otherwise should
// be correct.)
// (Has a quirk that it always adds a trailing newline to every line.)
fn join_joinable_lines(
  input: &str,
) -> String {
  // Construct state for parsing
  let mut state = State::new();
  // Go over each line, copying each into output
  'lines: for line in input.lines() {
    // For each line check first if the preceeding line precludes joining
    // If so: no need to think, just reset state and add the line
    if !state.remove_newlines {
      state.remove_newlines = true;
      state.add_line(line);
      continue;
    }
    // Same for if we're in a codeblock
    // (but don't reset state, we handle that below)
    if state.in_a_codeblock {
      state.add_line(line);
      continue;
    }
    // Then if the current line precludes joining
    //   Start with the easiest, if the line starts with 4 spaces, tab or '>' it
    //   is a block which shouldn't join at all, so just add.
    //   Also, empty line means new paragraph, shouldn't be joined either.
    if
      line.starts_with('\t') ||
      line.starts_with('>') ||
      line.starts_with("    ") ||
      line == ""
   {
      state.remove_newlines = false;
      state.add_line(line);
      continue;
    }
    //   Same for codeblock edges, with different state tracking
    if line == "```" {
      state.in_a_codeblock = !state.in_a_codeblock;
      state.add_line(line);
      continue;
    }
    //   Next look for static starts which may be indented
    //   If we find, same as above, just add
    {
      let tmp = line.trim_start();
      if
        // Unordered list start
        tmp.starts_with("- ") ||
        tmp.starts_with("* ") ||
        tmp.starts_with("+ ")
      {
        state.add_line(line);
        continue;
      }
    }
    //   Last we do fancy parsing of line contents
    //   Ordered list entry starts
    let mut first_loop = true;
    'chars: for ch in line.trim_start().chars() {
      // Numbered list is recognized by possibly indented numbers
      if ch.is_numeric() { first_loop = false; continue 'chars; }
      // followed directly by a dot
      // If we get here it's a match, just add and take the next line
      if ch == '.' && !first_loop { 
        state.add_line(line);
        continue 'lines;
      };
      // Any other character before '.' or no digit before '.' means it isn't
      // an ordered list
      break 'chars;
    }

    // If we get this far we can actually join the line with the preceeding.
    // Handle trying to join the first line to non-existent preceeding line
    if let Some(ch) = state.output.pop() {
      // Paranoid levels of insurance we don't delete any non-newline character
      // (shouldn't be reachable, as state.add_line ALWAYS adds '\n' after each line)
      if ch != '\n' { state.output.push(ch); }
      else { state.output.push(' '); }
    }
    state.add_line(line);
  }
  state.output
}
#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn md_join_normal_lines(){
    assert_eq!(
      &join_joinable_lines("just\nsome\ntext\nto\njoin"),
      "just some text to join\n",
    )
  }
  #[test]
  fn md_join_list(){
    assert_eq!(
      &join_joinable_lines("- some\n+ list\nto\n* join"),
      "- some\n+ list to\n* join\n",
    )
  }
  #[test]
  fn md_join_paragraph() {
    assert_eq!(
      &join_joinable_lines("hello\nworld\n\nnice weather,\neh?\n"),
      "hello world\n\nnice weather, eh?\n"
    )
  }
  #[test]
  fn md_join_explicit_newlines() {
    assert_eq!(
      &join_joinable_lines("hello\nworld\\\nnice weather  \neh?\n"),
      "hello world\nnice weather\neh?\n"
    )
  }
  #[test]
  fn md_join_codeblock() {
    assert_eq!(
      &join_joinable_lines(
"Code:\n    source code
Other code:\n\tsourcerer\ncode
Other other code:\n```\nsourcerest\n```\ncode\nend\n"
      ),
"Code:\n    source code
Other code:\n\tsourcerer\ncode \
Other other code:\n```\nsourcerest\n```\ncode\nend\n"
    )
  }
  #[test]
  fn md_join_blockquote() {
    assert_eq!(
      &join_joinable_lines("> Hello world!\nNice weather!\n>Is it?\nYep!"),
      "> Hello world!\nNice weather!\n>Is it?\nYep!\n"
    )
  }
  #[test]
  fn md_join_ordered_list() {
    assert_eq!(
      &join_joinable_lines("1. Fine\nstuff\n244. Okay-ish other\nstuff\n. Not a\nlist\nentry.\n"),
      "1. Fine stuff\n244. Okay-ish other stuff . Not a list entry.\n"
    )
  }
}
