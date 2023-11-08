/// Error type for HighlightingUI
#[derive(Debug)]
pub enum HighlightingUIError {
  // Separate, so we can print a guide for how to recover the terminal if needed
  RawmodeSwitchFailed(std::io::Error),
  // Can't do much smarter stuff. Possibly squeeze in some filename/linenumber.
  TerminalIOFailed(std::io::Error),
  // Received Ctrl+c, aborting input and returning to editor.
  Interrupted,
}
impl std::fmt::Display for HighlightingUIError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    use HighlightingUIError as HE;
    match self {
      HE::RawmodeSwitchFailed(e) => {
        write!(f,
          concat!(
            "Failed to switch terminal to/from rawmode.\n\n",
            "If your terminal is in rawmode when the editor quits, run 'reset'.\n\n",
            "Underlying error: {:?}"
          ),
          e
        )
      },
      HE::TerminalIOFailed(e) => {
        write!(f,
          concat!(
            "Failed to interact with terminal.\n\n",
            "Underlying error: {:?}"
          ),
          e
        )
      },
      HE::Interrupted => {
        write!(f, "Interrupted!")
      },
    }
  }
}
impl std::error::Error for HighlightingUIError{}
impl add_ed::error::UIErrorTrait for HighlightingUIError{}
