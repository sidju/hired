/// File IO abstractions
pub fn read_file(filepath: &str) -> Result<Vec<String>, &'static str> {
    read(filepath)
        .map_err(|e: std::io::Error| match e.kind() {
            std::io::ErrorKind::PermissionDenied => "Could not open file. Permission denied.",
            _ => {
                #[cfg(feature = "debug")] // Debug printouts if debug flag
                { println!("Error: {:?}", e); }
                "Unknown error while reading file."
            },
        })
}
pub fn write_file(filepath: &str, data: &[String]) -> Result<(), &'static str> {
    write(filepath, data)
        .map_err(|e: std::io::Error| match e.kind() {
            std::io::ErrorKind::PermissionDenied => "Could not open file. Permission denied.",
            _ => {
                #[cfg(feature = "debug")] // Debug printouts if debug flag
                { println!("Error: {:?}", e); }
                "Unknown error while writing file."
            },
        })
}

fn read(filepath: &str) -> std::io::Result<Vec<String>> {
    use std::io::{BufRead, BufReader};
    let mut data = Vec::new();
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(filepath)?;
    let mut reader = BufReader::new(file);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line)?
        {
            0 => break, // Is end of file
            _ => data.push(line),
        }
    }
    Ok(data)
}
fn write(filepath: &str, data: &[String]) -> std::io::Result<()> {
    use std::io::{BufWriter, Write};
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true) // Create if not found
        .open(filepath)?;
    let mut writer = BufWriter::new(file);
    for line in data {
        if line.len() != writer.write(line.as_bytes())? {
            panic!("Didn't write the entire line. Change write to write_all");
        }
    }
    writer.flush()?;
    Ok(())
}
