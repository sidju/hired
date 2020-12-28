// Relevant non-error consts
pub const HELP_TEXT: &str = 
"Application commands:
\r  q: Quit the application
\r  Q: Quit the application regardless of unsaved changes
\r  h: Print last occured error
\r  H: Toggle printing errors as they occur
\r  ?: Print this help text
\rFile commands:
\r  f: If filepath is given, sets active file to that. Else prints current filepath.
\r  e: Closes current file and opens given path.
\r  E: Closes current file and opens given path regardless of unsaved changes
\r  r: Append data from given path to selection.
\r  w: Write the selection to given filepath. Default selection is whole file and default path active file.
\r  W: Same as 'w' but appends to given filepath instead of overwriting.
\rPrint commands:
\r  p: Print the selection.
\r  n: Print the selection with line numbers.
\rBasic editing commands:
\r  a: Append lines entered after the command to selection. Stop line entry with only '.' on a line.
\r  i: Insert. Same as 'a' but places before selection.
\r  c: Change. Replace selection with lines entered after the command. Stop line ently with only '.' on a line.
\r  d: Delete. Deletes the selection.
\rAdvanced editing commands:
\r  m: Move selection to index given after command.
\r  t: Transfer (copy) selection to index given after command.
\r  j: Join selected lines into one line.
\rRegex commands:
\r  s: Substitute selection with regex replace very similar to 'sed'.
\rSpecial cases:
\r  No command: Takes the given selection (if any) and sets current selection to that.
\r";
// Pre-command parsing errors
pub const INDEX_PARSE: &str = "Could not parse given index.\n\r";
pub const NO_COMMAND: &str = "No valid command given.\n\r";

// Command handling errors
pub const UNDEFINED_COMMAND: &str = "Command not defined.\n\r";
pub const SELECTION_FORBIDDEN: &str = "That command doesn't take a selection.\n\r";
pub const UNSAVED_CHANGES: &str = "Unsaved changes. Force with the capitalised version of your command or save with 'w'.\n\r";
pub const NO_ERROR: &str = "No errors recorded.\n\r";

// Post-command parsing errors
pub const EXPRESSION_TOO_SHORT: &str = "Expression too short or not closed.\n\r";
pub const UNDEFINED_FLAG: &str = "Unknown flag entered.\n\r";
pub const DUPLICATE_FLAG: &str = "A flag was entered twice.\n\r";

// Buffer command errors
pub const BUFFER_NOT_IMPLEMENTED: &str = "Feature not implemented in buffer.\n\r";
pub const INDEX_TOO_BIG: &str = "Selection overshoots buffer length.\n\r";
pub const SELECTION_EMPTY: &str = "Selection empty or inverted.\n\r";
pub const MOVE_INTO_SELF: &str = "Cannot move selection into itself.\n\r";
pub const INVALID_REGEX: &str = "Invalid regex entered.\n\r";

// File interaction errors
pub const PERMISSION_DENIED: &str = "Could not open file. Permission denied.\n\r";
pub const NOT_FOUND: &str = "Could not open file. Not found.\n\r";
pub const UNKNOWN: &str = "Unknown error while reading file.\n\r";

// Terminal interaction errors
// No carriage returns, since only used through panic messages.
pub const TERMINAL_READ: &str = "Failed to read from terminal.";
pub const TERMINAL_WRITE: &str = "Failed to write to terminal.";
pub const DISABLE_RAWMODE: &str = "Failed to clear raw mode. Either restart terminal or run 'reset'. Good luck!";

