/// Error type for HighlightingUI
#[derive(Debug)]
pub enum HighlightingUIError {
  // Separate, so we can print a guide for how to recover the terminal if needed
  RawmodeSwitchFailed(std::io::Error),
  // Can't do much smarter stuff. Possibly squeeze in some filename/linenumber.
  TerminalIOFailed(std::io::Error),
  // Received Ctrl+c, aborting input and returning to editor.
  Interrupted,
  // Terminal not wide enough to print docs
  DocInsufficientWidth(termimad::InsufficientWidthError),
}
impl std::fmt::Display for HighlightingUIError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    use HighlightingUIError as HE;
    match self {
      HE::RawmodeSwitchFailed(e) => {
        write!(f,
          concat!(
            "Failed to switch terminal to/from rawmode.\r\n\r\n",
            "If your terminal is in rawmode when the editor quits, run 'reset'.\r\n\r\n",
            "Underlying error: {:?}"
          ),
          e
        )
      },
      HE::TerminalIOFailed(e) => {
        write!(f,
          concat!(
            "\r\n",
            "Failed to interact with terminal.\r\n\r\n",
            "Underlying error: {:?}"
          ),
          e
        )
      },
      HE::Interrupted => {
        write!(f, "\r\nInterrupted!")
      },
      HE::DocInsufficientWidth(e) => {
        write!(f,
          concat!(
            "Failed to render documentation.\n\n",
            "Underlying error: {}",
          ),
          e
        )
      },
    }
  }
}
impl std::error::Error for HighlightingUIError{}
impl add_ed::error::UIErrorTrait for HighlightingUIError{}
impl HighlightingUIError {
  pub fn from_termimad(e: termimad::Error) -> Self {
    use termimad::Error as TE;
    match e {
      TE::IO(inner) => Self::TerminalIOFailed(inner),
      TE::InsufficientWidth(inner) => Self::DocInsufficientWidth(inner),
    }
  }
}
