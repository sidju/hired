use crate::Buffer;

// A command abstraction
pub enum Cmd<'a> {
    A(Append<'a>),
}
pub struct Append<'a> {
    pub buffer: &'a dyn Buffer,
    pub selection: (usize, usize),
    pub input: Vec<String>,
    // Flags
    pub p: bool,
}
//impl Cmd {
//  /// Execute the command with stored arguments
//  fn execute() -> Result<(), &str> ;
//  /// A simple debug printout seems prudent as well
//  fn debug_print();
//}
