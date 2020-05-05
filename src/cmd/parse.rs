use crate::io;
use crate::Buffer;

use super::command::*;

// Consts for selection parsing
const SELECTION_PARSE_ERR: &str = "Could not parse given selection.";
const SELECTION_OUT_OF_BOUNDS: &str = "Selection out of bounds.";
const SELECTION_INVERTED: &str = "Selection is empty or inverted.";

// Consts for parsing commands
const NO_COMMAND_ERR: &str = "No valid command given.";
const UNDEFINED_COMMAND: &str = "Command not defined.";
const VALID_COMMANDS: &str =
    "abcdefthijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

// Consts for (regular) expression parsing
const EXPRESSION_TOO_SHORT_ERR: &str = "Expression too short or not closed.";
const ONE_EXPRESSION_COMMANDS: &str =
    "gG";
const TWO_EXPRESSION_COMMANDS: &str =
    "s";

// Consts for flag parsing
const UNDEFINED_FLAG: &str = "Unknown flag entered.";

// Utility function needed a few times while parsing
fn find_any_of(options: &str, input: &str) -> Option<usize> {
    for (index, ch) in input.char_indices() {
        if options.chars().any(|cmd| cmd == ch) {
            return Some(index);
        }
    }
    None
}
fn unpk<T>(option: Option<T>, decounter: &mut usize) -> bool {
    match option {
        Some(_) => {
            *decounter -= 1;
            true
        },
        None => false,
    }
}

// A parsing state machine
// Each state is the task completed when the state is active
struct Parse<S> {
    stage: S
}
struct Init<'a> {
    input: &'a str,
    state: &'a mut crate::State,
}
struct Command<'a> {
    input: &'a str,
    selection: (usize, usize),
    command: char,
    state: &'a mut crate::State,
}
#[derive(Derivative)]
#[derivative(Debug)]
struct Expression<'a> {
    input: &'a str,
    selection: (usize, usize),
    command: char,
    expression: Vec<&'a str>,
    #[derivative(Debug="ignore")]
    state: &'a mut crate::State,
}

// The state transitions for the parser
impl <'a> Parse<Init<'a>> {
    fn new(input: &'a str, state: &'a mut crate::State) -> Self {
        Self{stage: Init{
            input: input,
            state: state,
        }}
    }
}
impl <'a> Parse<Command<'a>> {
    // Get the selection and command, the basic data
    fn command(from: Parse<Init<'a>>) -> Result<Self, &'static str> {
        let bufferlen = from.stage.state.buffer.len();
        let selection = from.stage.state.selection.unwrap_or_else(||(0 as usize, bufferlen));
        let parse_index = | index: &str, selection: usize, default: usize | -> Result<usize, &str> {
            if index.len() == 0 {
                Ok(default)
            }
            else {
                match index {
                    "." => Ok(selection),
                    "$" => Ok(bufferlen),
                    "+" => Ok(selection + 1),
                    "-" => Ok(selection - 1),
                    _ => { match index.chars().next() {
                        Some('-') => index[1..]
                            .parse::<usize>()
                            .map(|x| x - 1),
                        Some('+') => index[1..]
                            .parse::<usize>()
                            .map(|x| x + 1),
                        _ => index.parse::<usize>()
                    }.map_err(|_| SELECTION_PARSE_ERR)},
                }
            }
        };
        // Separate out the index of the first command, an the selection str
        let (command_index, string) =
            match find_any_of(VALID_COMMANDS, from.stage.input) {
                Some(x) => Ok((x, &from.stage.input[..x])),
                None => Err(NO_COMMAND_ERR),
            }?;

        // If no string was given we use the default
        let parsed = if string.len() == 0 {
            Ok(selection)
        }
        // Else we parse what is given, if possible
        else {
            // split the parsing based on where there is a ','
            match find_any_of(",;", string) {
                Some(x) => {
                    // If the found one is ';' the default shift
                    // from 1,bufferlen to selection.0,bufferlen
                    let (start, end) = if string.chars().next() == Some(',') {
                        (parse_index(&string[..x], selection.0, 1)?,
                         parse_index(&string[x+1..], selection.1, bufferlen)?)
                    }
                    else {
                        (parse_index(&string[..x], selection.0, 1)?,
                         parse_index(&string[x+1..], selection.1, bufferlen)?)
                    };
                    // Handle assorted edge cases
                    if end > bufferlen {
                        Err(SELECTION_OUT_OF_BOUNDS)
                    }
                    else if start > end {
                        Err(SELECTION_INVERTED)
                    }
                    else {
                        if start == 0 {
                            Ok((start, end))
                        }
                        else {
                            Ok((start - 1, end ))
                        }
                    }
                },
                None => {
                    // If no ',' exists we check if one index was given
                    // It is treated as end
                    let lone = parse_index(string, selection.1, bufferlen)?;
                    // Avoid panics from overshooting the buffer length
                    if lone > bufferlen {
                        Err(SELECTION_OUT_OF_BOUNDS)
                    }
                    else if lone == 0 {
                        Ok((lone, lone))
                    }
                    else {
                        Ok((lone - 1, lone))
                    }
                },
            }
        }?;
        // Build the state for the next stage
        Ok(Self{stage: Command{
            input: &from.stage.input[command_index + 1 ..],
            selection: parsed,
            command:
            from.stage.input[command_index..].chars().next().unwrap(),
            state: from.stage.state,
        }})
    }
}
// Parse out the expression, if relevant
impl <'a> Parse<Expression<'a>> {
    fn expression(from: Parse<Command<'a>>) -> Result<Self, &'static str> {
        #[cfg(feature = "debug")]
        {
            println!("About to parse expressions. Input is: {}",
                     from.stage.input);
        }
        let mut segments = Vec::new(); // To store the expressions
        let mut input = from.stage.input;
        let mut remain = {// The number of segments to find
            let cmd = from.stage.command;
            if ONE_EXPRESSION_COMMANDS.chars().any(|x| x == cmd) {
                1
            }
            else if TWO_EXPRESSION_COMMANDS.chars().any(|x| x == cmd) {
                2
            }
            else {
                0
            }
        };
        // Only if there is something to parse do we parse
        if remain > 0 {
            // the first char is the separator
            let separator = match input.chars().next() {
                Some(ch) => ch,
                None => return Err(EXPRESSION_TOO_SHORT_ERR),
            };
            input = &input[1 ..];
            while remain > 0 {
                // find the next separator
                let index = match input.find(separator) {
                    Some(i) => i,
                    None => return Err(EXPRESSION_TOO_SHORT_ERR),
                };
                #[cfg(feature = "debug")]
                {
                    println!("Found separator at index {}.", index);
                }
                // extract the data and add to segments
                segments.push(&input[.. index]);
                // update input to exclude the extracted data
                input = &input[index+1 ..];
                // update remaining segments to find
                remain -= 1;
            }
        }
        // Build the state for the next stage
        let res = Expression{
            input: input,
            selection: from.stage.selection,
            command: from.stage.command,
            expression: segments,
            state: from.stage.state,
        };
        #[cfg(feature = "debug")]
        {
            println!("Done parsing expressions. Result is:\n{:?}",
                     res);
        }
        Ok(Self{stage: res})
    }
}
// Parse out the flags, get additional input and unwrap
impl <'a> Parse<Cmd<'a>> {
    fn flags(from: Parse<Expression<'a>>) -> Result<Self, &'static str> {
        // Keep track of the number of unidentified flags
        // If non-zero when returning invalid flags were given
        let mut unidentified = from.stage.input.len() - 1; // Subtract newline
        // The universal flags
        let p = unpk(from.stage.input.find('p'), &mut unidentified);
        let n = unpk(from.stage.input.find('n'), &mut unidentified);
        let h = unpk(from.stage.input.find('h'), &mut unidentified);
        let l = unpk(from.stage.input.find('l'), &mut unidentified);
        // Identify the command
        let cmd = match from.stage.command {
            // The simple ones
            'q' => Ok(Self{stage: Cmd::Quit(from.stage.state)}),
            'Q' => Ok(Self{stage: Cmd::ForceQuit(from.stage.state)}),
            'h' => Ok(Self{stage: Cmd::Perror(from.stage.state)}),
            'H' => Ok(Self{stage: Cmd::SetPerror(from.stage.state)}),
            // The more complex
            'p' => {
                Ok(Self{
                    stage: Cmd::Print(Print{
                        state: from.stage.state,
                        selection: from.stage.selection,
                        n: n,
                        h: h,
                        l: l,
                    })
                })
            },
            'a' => {
                let input = io::read_insert(&from.stage.state);
                Ok(Self{
                    stage: Cmd::Insert(Insert{
                        input: input,
                        state: from.stage.state,
                        index: from.stage.selection.1 + 1,
                        p: p,
                        n: n,
                        h: h,
                        l: l,
                    })
                })
            },
            'i' => {
                let input = io::read_insert(&from.stage.state);
                Ok(Self{
                    stage: Cmd::Insert(Insert{
                        input: input,
                        state: from.stage.state,
                        index: from.stage.selection.0,
                        p: p,
                        n: n,
                        h: h,
                        l: l,
                    })
                })
            },
            'c' => {
                let input = io::read_insert(&from.stage.state);
                Ok(Self{
                    stage: Cmd::Change(Change{
                        input: input,
                        state: from.stage.state,
                        selection: from.stage.selection,
                        p: p,
                        n: n,
                        h: h,
                        l: l,
                    })
                })
            },
            'd' => {
                Ok(Self{
                    stage: Cmd::Delete(Delete{
                        state: from.stage.state,
                        selection: from.stage.selection,
                    })
                })
            },
            's' => {
                let g = unpk(from.stage.input.find('g'), &mut unidentified);
                let reg = from.stage.expression[0];
                let ex = from.stage.expression[1];
                Ok(Self{
                    stage: Cmd::Substitute(Substitute{
                        state: from.stage.state,
                        selection: from.stage.selection,
                        expression: (reg,ex),
                        g: g,
                        p: p,
                        n: n,
                        h: h,
                        l: l,
                    })
                })
            },
            'e' => {
                // All of the input should be filename => no unidentified
                unidentified = 0;
                Ok(Self{
                    stage: Cmd::Open(Open{
                        state: from.stage.state,
                        path: &from.stage.input[..from.stage.input.len() - 1],
                    })
                })
            },
            'r' => {
                // All of the input should be filename => no unidentified
                unidentified = 0;
                Ok(Self{
                    stage: Cmd::Read(Read{
                        state: from.stage.state,
                        index: from.stage.selection.1,
                        path: &from.stage.input[..from.stage.input.len() - 1],
                    })
                })
            },
            'w' => {
                // All of the input should be filename => no unidentified
                unidentified = 0;
                Ok(Self{
                    stage: Cmd::Write(Write{
                        state: from.stage.state,
                        selection: from.stage.selection,
                        path: &from.stage.input[..from.stage.input.len() - 1],
                    })
                })
            },
            _ => Err(UNDEFINED_COMMAND)
        }?;
        // Verify that no unknown flags were given
        if unidentified > 0 {
            Err(UNDEFINED_FLAG)
        }
        else {
            Ok(cmd)
        }
    }
    fn unpack(self) -> Cmd<'a> {
        self.stage
    }
}

pub fn parse<'a>(state: &'a mut crate::State, command: &'a mut str)
             -> Result<Cmd<'a>, &'static str> {
    // Create the initial parser
    let init = Parse::new(&*command, state);
    // Get the selection and command char
    let command = Parse::command(init)?;
    // Then the expression, if any
    let expression = Parse::expression(command)?;
    // Finally the flags
    let flags = Parse::flags(expression)?;
    // Unwrap this to a command enum and return
    let cmd = flags.unpack();
    Ok(cmd)
}
