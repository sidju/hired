/// Trait that defines a buffer supporting 'ed's base commands
pub trait Buffer {
  /// Check that the index is safe to operate on
  fn verify_index(&self, index: usize) -> Result<(), &'static str> ;
  /// Check that the selection is safe to operate on
  fn verify_selection(&self, selection: (usize, usize)) -> Result<(), &'static str> ;
  /// Outputs the selection as a single string
  fn format_selection(&self, selection: (usize, usize), numbered: bool, literal: bool) -> Result<String, &'static str> ;
  /// Takes a iterator over lines in strings and inserts at given index
  fn insert(&mut self, data: &mut Vec<String>, index: usize) -> Result<(), &'static str> ;
  /// Deletes the lines in the selection
  fn delete(&mut self, selection: (usize, usize)) -> Result<(), &'static str> ;
  /// Delete the given selection and insert the given data in its place
  fn change(&mut self, data: &mut Vec<String>, selection: (usize, usize)) -> Result<(), &'static str> ;
  /// Perform regex search and replace on the selection changing pattern.0 to pattern.1
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool) -> Result<(), &'static str> ;
  // Find the indices in the selection whose lines match the regex pattern
  // fn find_matching(&self, pattern: &str, selection: (usize, usize)) -> Result<(), &str> ;
  /// Return the given selection without any formatting
  fn get_selection(&self, selection: (usize, usize)) -> Result<&[String], &'static str>;
  fn len(&self) -> usize ;
  /// Inform the buffer that it has been saved
  fn set_saved(&mut self);
  /// Returns true if no changes have been made since last saving
  fn saved(&self) -> bool;
}

pub struct VecBuffer {
  saved: bool,
  buffer: Vec<String>
}
impl VecBuffer {
  pub fn new() -> Self
  {
    Self{
      saved: true,
      buffer: Vec::new(),
    }
  }
}
impl Buffer for VecBuffer
{
  fn verify_index(&self, index: usize) -> Result<(), &'static str>
  {
    if index > self.buffer.len() {
      return Err("Selection overshoots buffer length.");
    }
    Ok(())
  }
  fn verify_selection(&self, selection: (usize, usize)) -> Result<(), &'static str>
  {
    if selection.0 >= selection.1 {
      return Err("Selection empty or inverted.");
    }
    if selection.1 > self.buffer.len() {
      return Err("Selection overshoots buffer length.");
    }
    Ok(())
  }
  fn format_selection(&self, selection: (usize, usize), numbered: bool, _literal: bool) -> Result<String, &'static str>
  {
    self.verify_selection(selection)?;
    let mut ret = String::new();
    if numbered {
      let mut i = selection.0;
      for line in &self.buffer[selection.0 .. selection.1] {
        i += 1;
        ret.push_str(&format!("{}:\t{}", i, line));
      }
    }
    else {
      for line in &self.buffer[selection.0 .. selection.1] {
        ret.push_str(line);
      }
    }
    // Perform the syntax highlighting
    Ok(ret)
  }
  fn insert(&mut self, data: &mut Vec<String>, mut index: usize) -> Result<(), &'static str>
  {
    if index <= self.buffer.len() + 1 {
      self.saved = false;
      //0 is valid but needs to be specially handled
      if index != 0 { index -= 1; }
      #[cfg(feature = "debug")] // Debug printouts if debug flag
      { println!("inserting at index {}", index); }
      // To minimise time complexity we split the vector immediately
      let mut tail = self.buffer.split_off(index);
      // And then append both the insert and the split off part
      self.buffer.append(data);
      self.buffer.append(&mut tail);
      Ok(())
    }
    else {
      println!("Buffer len is: {} and index is: {}", self.buffer.len(), index);
      Err("Invalid selection.")
    }
  }
  fn delete(&mut self, selection: (usize, usize)) -> Result<(), &'static str>
  {
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= self.buffer.len() {
      self.saved = false;
      let mut tail = self.buffer.split_off(selection.1);
      let _deleted = self.buffer.split_off(selection.0);
      self.buffer.append(&mut tail);
      Ok(())
    }
    else {
      #[cfg(feature = "debug")]
      {
        println!("The selection was {:?}", selection);
        if selection.0 >= selection.1 {
          println!("The selection is empty or inverted");
        }
        if selection.1 > self.buffer.len() {
          println!("The selection overshoots the buffer.");
        }
      }
      Err("Invalid selection.")
    }
  }
  fn change(&mut self, data: &mut Vec<String>, selection: (usize, usize)) -> Result<(), &'static str>
  {
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= self.buffer.len() {
      self.saved = false;
      let mut tail = self.buffer.split_off(selection.1);
      let _deleted = self.buffer.split_off(selection.0);
      self.buffer.append(data);
      self.buffer.append(&mut tail);
      Ok(())
    }
    else {
      #[cfg(feature = "debug")]
      {
        println!("The selection was {:?}", selection);
        if selection.0 >= selection.1 {
          println!("The selection is empty or inverted");
        }
        if selection.1 > self.buffer.len() {
          println!("The selection overshoots the buffer.");
        }
      }
      Err("Invalid selection.")
    }
  }
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool) -> Result<(), &'static str>
  {
    use regex::Regex;
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= self.buffer.len() {
      self.saved = false; // TODO: actually check if changes are made
      // Compile the regex used to match/extract data
      let regex = Regex::new(pattern.0).expect("Failed to create pattern regex.");
      if global {
        for index in selection.0 .. selection.1 {
          let after = regex.replace_all(&(self.buffer[index]), pattern.1);
          #[cfg(feature = "debug")]
          {
            print!("Replacing:\n{}\nwith:\n{}",
              &(self.buffer[index]), after
            );
          }
          self.buffer[index] = after.to_string();
        }
        Ok(())
      }
      else {
        // Check each line for a match. If found, replace and break
        for index in selection.0 .. selection.1 {
          if regex.is_match(&(self.buffer[index])) {
            let after = regex.replace(&(self.buffer[index]), pattern.1);
            #[cfg(feature = "debug")]
            {
              print!("Replacing:\n{}with:\n{}",
                &(self.buffer[index]), after
              );
            }
            self.buffer[index] = after.to_string();
            break;
          }
        }
        Ok(())
      }
    }
    else {
      #[cfg(feature = "debug")]
      {
        println!("The selection was {:?}", selection);
        if selection.0 >= selection.1 {
          println!("The selection is empty or inverted");
        }
        if selection.1 > self.buffer.len() {
          println!("The selection overshoots the buffer.");
        }
      }
      Err("Invalid selection.")
    }
  }
  // fn find_matching(&self, pattern: &str, selection: (usize, usize)) -> Result<(), &str> ;
    fn get_selection(&self, selection: (usize, usize)) -> Result<&[String], &'static str> {
        self.verify_selection(selection)?;
        Ok(&self.buffer[selection.0 .. selection.1])
    }
    fn len(&self) -> usize {
        self.buffer.len()
    }
    fn set_saved(&mut self) {
        self.saved = true;
    }
    fn saved(&self) -> bool {
        self.saved
    }
}
