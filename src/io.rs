/// IO abstractions
use crate::State;

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::Theme;
use syntect::util::as_24_bit_terminal_escaped;

pub fn format_print(
    state: &State,
    lines: &[String],
    offset: usize,
    n: bool,
    _l: bool
) {
    if true {
        let theme = &state.theme_lib.themes["base16-ocean.dark"];
        let tmp = state.syntax_lib
          .find_syntax_for_file(&state.file.as_ref().unwrap_or(&String::new()));
        let syntax = tmp
            .unwrap_or_else(|_| Some(state.syntax_lib.find_syntax_plain_text()))
            .unwrap_or_else(|| state.syntax_lib.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, theme);
        for (i, line) in lines.iter().enumerate() {
            let highlighted = highlighter.highlight(line, &state.syntax_lib);
            let escaped = as_24_bit_terminal_escaped(&highlighted[..], false);
            if n {
                print!("{}: {}",i + offset + 1, escaped);
            }
            else {
                print!("{}", escaped);
            }
            print!("\x1b[0m");
        }
    }
    else {
        for (i, line) in lines.iter().enumerate() {
            if n {
                print!("{}: {}",i + offset + 1, line);
            }
            else {
                print!("{}", line);
            }
        }
    }
}
pub fn read_command(state: &mut crate::State, command: &mut String) {
    // Clear the line, since read_line appends
    command.clear();
    // Print prompt if relevant
    match &state.prompt {
        Some(p) => print!("{}", p),
        None => {},
    }
    // Read input
    loop {
        match state.stdin.read_line(command) {
            Ok(_bytes_read) => {
                #[cfg(feature = "debug")]
                {
                    println!("Read {} bytes from stdin.", _bytes_read);
                    println!("Read {}", command);
                }
                if command.ends_with("\\\n") { // If newline was escaped
                    command.replace_range(command.len() - 2 .., "\n");
                }
                else {
                    break;
                }
            },
            e => {
                e.expect("Failed to read from stdin.");
                break;
            },
        }
    }
}
pub fn read_insert(state: &crate::State) -> Vec<String>
{
    // Create a variable to save the inserted text into
    let mut insert = Vec::new();
    // Loop until the insert is ended by a lone dot
    loop {
        // Get a new line, since read_line appends into a string
        let mut line = String::new();
        // Read the input
        match state.stdin.read_line(&mut line) {
            Ok(_bytes_read) => {
                #[cfg(feature = "debug")]
                {
                    println!("Read {} bytes from stdin.", _bytes_read);
                }
                ()
            },
            e => {
                e.expect("Failed to read from stdin.");
                ()
            },
        }
        // If it is a lone dot the entry ends
        if line.trim() == "." {
            break;
        }
        // Else append it into the insert vector
        else {
            insert.push(line);
        }
    }
    // Finally return the collected lines
    insert
}
