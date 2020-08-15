// Consts for selection parsing
pub const INDEX_PARSE_ERR: &str = "Could not parse given index.";
pub const SELECTION_OUT_OF_BOUNDS: &str = "Selection out of bounds.";
pub const SELECTION_INVERTED: &str = "Selection is empty or inverted.";
pub const NO_COMMAND_ERR: &str = "No valid command given.";

pub const UNDEFINED_COMMAND: &str = "Command not defined.";

// Command specific errors
pub const SELECTION_FORBIDDEN: &str = "That command doesn't take a selection.";
pub const UNSAVED_CHANGES: &str = "Unsaved changes. Force with the capitalised version of your command or save with 'w'.";

pub const EXPRESSION_TOO_SHORT: &str = "Expression too short or not closed.";
pub const ONE_EXPRESSION_COMMANDS: &str =
    "gG";
pub const TWO_EXPRESSION_COMMANDS: &str =
    "s";

pub const UNDEFINED_FLAG: &str = "Unknown flag entered.";
