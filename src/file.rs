use std::io::ErrorKind;
use crate::error_consts::*;

/// File IO abstractions
pub fn read_file(filepath: &str, must_exist: bool) -> Result<Vec<String>, &'static str> {
  match read(filepath) {
    Ok(x) => Ok(x),
    Err(e) => match e.kind() {
      ErrorKind::PermissionDenied => Err(PERMISSION_DENIED),
      ErrorKind::NotFound => {
        if must_exist { Err(NOT_FOUND) }
        else { Ok(Vec::with_capacity(0)) }
      }
      _ => {
        #[cfg(feature = "debug")] // Debug printouts if debug flag
        { println!("Error: {:?}", e); }
        Err(UNKNOWN)
      },
    },
  }
}
pub fn write_file(filepath: &str, data: &[String], append: bool)
  -> Result<(), &'static str>
{
  write(filepath, data, append)
    .map_err(|e: std::io::Error| match e.kind() {
      ErrorKind::PermissionDenied => PERMISSION_DENIED,
      ErrorKind::NotFound => NOT_FOUND,
      _ => {
        #[cfg(feature = "debug")] // Debug printouts if debug flag
        { println!("Error: {:?}", e); }
        UNKNOWN
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
fn write(filepath: &str, data: &[String], append: bool) -> std::io::Result<()> {
    use std::io::{BufWriter, Write};
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(append)
        .truncate(!append) // Delete current contents if any
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
