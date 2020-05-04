use crate::State;

// A command abstraction
mod command;
// And a parser that outputs structs for it
mod parse;
use parse::parse;

pub fn parse_and_run(
    state: &mut State,
    mut command:
    &mut String
) -> Result<(), &'static str> {
    // Parse the command
    let cmd = parse(state, &mut command)?;
    cmd.debug_print();
    cmd.execute()
}
