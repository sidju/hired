// Relevant non-error consts
pub const HELP_TEXT: &str = 
"Application commands:
  q: Quit the application
  Q: Quit the application regardless of unsaved changes
  h: Print last occured error
  H: Toggle printing errors as they occur
  ?: Print this help text
File commands:
  f: If filepath is given, sets active file to that. Else prints current filepath.
  e: Closes current file and opens given path.
  E: Closes current file and opens given path regardless of unsaved changes
  r: Append data from given path to selection.
  w: Write the selection to given filepath. Default selection is whole file and default path active file.
  W: Same as 'w' but appends to given filepath instead of overwriting.
Print commands:
  p: Print the selection.
  n: Print the selection with line numbers.
Basic editing commands:
  a: Append lines entered after the command to selection. Stop line entry with only '.' on a line.
  i: Insert. Same as 'a' but places before selection.
  c: Change. Replace selection with lines entered after the command. Stop line ently with only '.' on a line.
  d: Delete. Deletes the selection.
Advanced editing commands:
  m: Move selection to index given after command.
  t: Transfer (copy) selection to index given after command.
  j: Join selected lines into one line.
Regex commands:
  s: Substitute selection with regex replace very similar to 'sed'.
Special cases:
  No command: Takes the given selection (if any) and sets current selection to that.
";
// Pre-command parsing errors
pub const INDEX_PARSE_ERR: &str = "Could not parse given index.";
pub const SELECTION_OUT_OF_BOUNDS: &str = "Selection out of bounds.";
pub const SELECTION_INVERTED: &str = "Selection is empty or inverted.";
pub const NO_COMMAND_ERR: &str = "No valid command given.";

// Command handling errors
pub const UNDEFINED_COMMAND: &str = "Command not defined.";
pub const SELECTION_FORBIDDEN: &str = "That command doesn't take a selection.";
pub const UNSAVED_CHANGES: &str = "Unsaved changes. Force with the capitalised version of your command or save with 'w'.";

// Post-command parsing errors
pub const EXPRESSION_TOO_SHORT: &str = "Expression too short or not closed.";
pub const UNDEFINED_FLAG: &str = "Unknown flag entered.";
pub const DUPLICATE_FLAG: &str = "A flag was entered twice.";
