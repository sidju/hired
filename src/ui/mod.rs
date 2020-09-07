/// IO abstractions
use crate::State;

// Start with the printing helpers
fn print_separator(width: usize) {
  // Create the string to hold the separator with the capacity we know it will use
  let mut sep = String::with_capacity(width);
  for _ in 0 .. width {
    sep.push('─');
  }
  // Print the generated separator
  println!("{}", sep);
}
fn print_view(
  size: (usize, usize),
  text: &[&str],
  line_nr: mut usize,
  n: bool,
) {
  // Iterate through the text adding the resulting text into a buffer
  let buffer = String::new();
  let lines_printed = 0;

  for line in text {
    // To handle wrapping, print character by character
    for (i, ch) in line.char_indices() {
      // If i is divisible by terminal width it is time to split
      if i % size.0 == 0 {
        buffer.push('\n');
        lines_printed += 1;
        if lines_printed >= size.1 { break; }

        if n {
          if i == 0 {
            buffer.push_str(&(line_nr.to_string()));
          }
          else {
            for _ in 0 .. line.to_string().len() { buffer.push(' '); }
          }  
          buffer.push('│');
          line_nr += 1; // Only increment here, since we don't care about it otherwise
        }
      }
    }
  }
  // Finally, when we have gone through all the relevant stuff, we print the buffer
  // This way we save sysem-calls, which are often quite slow
  print!("{}", buffer);
}
