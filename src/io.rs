/// IO abstractions to handle errors better

pub fn read_command(state: &mut crate::State, command: &mut String) {
    // Clear the line, since read_line appends
    command.clear();
    // Print prompt if relevant
    match &state.prompt {
        Some(p) => print!("{}", p),
        None => {},
    }
    // Read input
    match state.stdin.read_line(command) {
        Ok(bytes_read) => {
            #[cfg(feature = "debug")]
            {
                println!("Read {} bytes from stdin.", bytes_read);
            }
            ()
        },
        e => {
            e.expect("Failed to read from stdin.");
            ()
        },
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
            Ok(bytes_read) => {
                #[cfg(feature = "debug")]
                {
                    println!("Read {} bytes from stdin.", bytes_read);
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

pub fn print_selection(
    selection: (usize, usize),
    buffer: &Vec<String>,
    numbered: bool,
) -> Result<(), String> {
    #[cfg(feature = "debug")]
    {
        println!("the selection was: {:?}", selection);
    }
    if selection.0 != selection.1 {
        // Slice the data we want to print and iteratively print it
        if numbered {
            let mut i = selection.0;
            for line in &buffer[selection.0 .. selection.1] {
                i += 1;
                print!("{}:\t{}", i, line);
            }
        }
        else {
            for line in &buffer[selection.0 .. selection.1] {
                print!("{}", line);
            }
        }
        Ok(())
    }
    else {
        Err("Cannot print empty selection.".to_string())
    }
}
// Print last error if some, else create one
pub fn print_error(error: &Option<String>) -> Result<(), String> {
    match error {
        Some(e) => Ok(println!("{}", e)), // yes, this only returns Ok(())
        None => Err("No error message found.".to_string()), // Ironic, eh?
    }
}
