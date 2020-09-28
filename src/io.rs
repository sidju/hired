/// IO abstractions

pub fn read_command(state: &mut crate::State, command: &mut String) {
    // Clear the line, since read_line appends
    command.clear();
    // Print prompt
    print!("{}", &state.prompt);
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
