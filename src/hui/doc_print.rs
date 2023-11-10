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
  let mangled = doc.into();
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
