use crate::Buffer;

// A command abstraction
mod command;
use command::Cmd;
// And a parser that outputs structs for it
mod parse;
use parse::parse;

pub fn parse_and_run(state: &mut crate::State, mut command: &mut String) -> Result<(), &'static str> {
    // Parse the command
    let cmd = parse(state, &mut command);

    Ok(())
}


