use crate::Buffer;
use crate::State;

// A command abstraction
#[derive(Debug)]
pub enum Cmd<'a> {
    Quit(&'a mut State), // Quit
    ForceQuit(&'a mut State), // Quit ignoring unsaved changes
    Perror(&'a mut State), // Print last error
    SetPerror(&'a mut State), // Toggle printing errors as they occur
    // Undo(&'a mut State), // Revert buffer one command back (including Undo)

    Print(Print<'a>), // Print the defined lines in defined mode

    Insert(Insert<'a>), // Insert given text at given index
    Delete(Delete<'a>), // Delete the selection
    Change(Change<'a>), // Replace selection with given text

    // Move(Move<'a>), // Moves selection to given index
    // Copy(Copy<'a>), // Copy the selection to given index
    // Join(Join<'a>), // Join the selection into one line

    // Cut(Cut<'a>), // Cut selection from buffer
    // Paste(Paste<'a>), // Append what was last cut to selection

    // Global(Global<'a>), // Apply the given commands to lines matching expr.
    // Substitute(Substitute<'a>), // Regex search and replace over selection

    // ForceOpen(Open<'a>), // Open given file into new buffer ignoring unsaved
    // Open(Open<'a>), // Open given file into new buffer
    // Read(Read<'a>), // Append contents of given file to selection
    // Write(Write<'a>), // Write the selection to given file
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Print<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub selection: (usize, usize),
    // Flags
    pub n: bool,
    pub l: bool,
}
impl <'a> Print<'a> {
    // A method to create a print command after another command
    // To handle the p, n, l flags in non-print commands
    fn from_state(
        state: &'a mut State,
        n: bool,
        l: bool,
    ) -> Self {
        Self {
            selection: state.selection
                .unwrap_or_else(||(0 as usize, state.buffer.len())),
            state: state,
            n: n,
            l: l
        }
    }
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Insert<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub index: usize,
    pub input: Vec<String>,
    // Flags
    pub p: bool,
    pub n: bool,
    pub l: bool,
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Delete<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub selection: (usize, usize),
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Change<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub selection: (usize, usize),
    pub input: Vec<String>,
    // Flags
    pub p: bool,
    pub n: bool,
    pub l: bool,
}
impl <'a> Cmd<'a> {
    /// A simple debug printout
    pub fn debug_print(&self) {
        println!("{:?}", self);
    }
    /// Execute the command with stored arguments
    /// Returns the selection after command execution or error msg
    pub fn execute(self) -> Result<(), &'static str> {
        match self {
            Self::Quit(state) => {
                if state.buffer.saved() {
                    state.done = true;
                    Ok(())
                }
                else {
                    Err("Unsaved changes!")
                }
            },
            Self::ForceQuit(state) => { state.done = true; Ok(()) },
            Self::Perror(state) => { println!("{:?}", state.error); Ok(()) },
            Self::SetPerror(state) => {
                state.print_errors = ! state.print_errors;
                Ok(())
            },
            //Self::Undo => { state.done = true; Ok(()) },
            Self::Print(print) => Ok(()),
            Self::Insert(insert) => Ok(()),
            Self::Delete(delete) => Ok(()),
            Self::Change(change) => Ok(()),
        }
    }
}
