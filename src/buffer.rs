/// Trait that defines a buffer supporting 'ed's base commands
pub trait Buffer {
  /// Check that the index is safe to operate on
  fn verify_index(&self, index: usize) -> Result<(), &'static str> ;
  /// Check that the selection is safe to operate on
  fn verify_selection(&self, selection: (usize, usize)) -> Result<(), &'static str> ;
  /// Takes a iterator over lines in strings and inserts at given index
  fn insert(&mut self, data: &mut Vec<String>, index: usize) -> Result<(), &'static str> ;
  /// Deletes the lines in the selection
  fn delete(&mut self, selection: (usize, usize)) -> Result<(), &'static str> ;
  /// Delete the given selection and insert the given data in its place
  fn change(&mut self, data: &mut Vec<String>, selection: (usize, usize)) -> Result<(), &'static str> ;
  /// Move selection to index
  fn mov(&mut self, selection: (usize, usize), index: usize) -> Result<(), &'static str> ;
  /// Copy selection to index
  fn copy(&mut self, selection: (usize, usize), index: usize) -> Result<(), &'static str> ;
  /// Join all lines in selection into one line
  fn join(&mut self, selection: (usize, usize)) -> Result<(), &'static str> ;
  /// Perform regex search and replace on the selection changing pattern.0 to pattern.1
  /// Returns selection, since it may delete or add lines
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool) -> Result<(usize, usize), &'static str> ;
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
  fn mov(&mut self, selection: (usize, usize), index: usize) -> Result<(), &'static str> {
    self.verify_selection(selection)?;
    self.verify_index(index - 1)?;
    if index - 1 < selection.0 {
      // split out the relevant parts of the buffer
      let mut tail = self.buffer.split_off(selection.1);
      let mut data = self.buffer.split_off(selection.0);
      let mut middle = self.buffer.split_off(index - 1);
      // Reassemble
      self.buffer.append(&mut data);
      self.buffer.append(&mut middle);
      self.buffer.append(&mut tail);
      Ok(())
    }
    else if index - 1 >= selection.1 {
      // split out the relevant parts of the buffer
      let mut tail = self.buffer.split_off(index);
      let mut middle = self.buffer.split_off(selection.1);
      let mut data = self.buffer.split_off(selection.0);
      // Reassemble
      self.buffer.append(&mut middle);
      self.buffer.append(&mut data);
      self.buffer.append(&mut tail);
      Ok(())
    }
    else {
      Err("Cannot move selection into itself.")
    }
  }
  fn copy(&mut self, selection: (usize, usize), index: usize) -> Result<(), &'static str> {
    self.verify_selection(selection)?;
    self.verify_index(index)?;
    // Get the data
    let mut data = Vec::new();
    for line in &self.buffer[selection.0 .. selection.1] {
      data.push(line.clone());
    }
    // Insert it
    let mut tail = self.buffer.split_off(index);
    self.buffer.append(&mut data);
    self.buffer.append(&mut tail);
    Ok(())
  }
  fn join(&mut self, selection: (usize, usize)) -> Result<(), &'static str> {
    self.verify_selection(selection)?;
    let mut tail = self.buffer.split_off(selection.1);
    let data = self.buffer.split_off(selection.0);
    let mut newline = String::new();
    for line in data {
      newline.push_str(&line); // Add in the line
      newline.pop(); // Remove the newline from it
    }
    newline.push('\n');
    self.buffer.push(newline);
    self.buffer.append(&mut tail);
    Ok(())
  }
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool) -> Result<(usize, usize), &'static str>
  {
    use regex::Regex;
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= self.buffer.len() {
      self.saved = false; // TODO: actually check if changes are made
      // Compile the regex used to match/extract data
      let regex = Regex::new(pattern.0)
        .map_err(|_| "Invalid regex entered.")
        ?;
      let mut selection_after = selection;
      if global {
        // Cut out the whole selection from buffer
        let mut tail = self.buffer.split_off(selection.1);
        let before = self.buffer.split_off(selection.0);
        let mut after = String::new();
        for line in before {
          let tmp = regex.replace_all(&line, pattern.1);
          after.push_str(&tmp);
          #[cfg(feature = "debug")]
          { print!("Replacing:\n{}\nwith:\n{}\n",line, tmp); }
        }
        // If the changed area doesn't end with newline we bind in next line
        if (!after.ends_with('\n')) && (tail.len() > 0) {
          after.push_str(&tail.remove(0));
        }
        // Add to the buffer
        for newline in after.split('\n') {
          if newline.len() > 0 {
            self.buffer.push(format!("{}\n", newline));
          }
        }
        selection_after.1 = self.buffer.len();
        self.buffer.append(&mut tail); // And put the tail back on
      }
      else {
        // Check each line for a match. If found, replace and break
        for index in selection.0 .. selection.1 {
          if regex.is_match(&(self.buffer[index])) {
            let mut tail = self.buffer.split_off(index + 1);
            let before = self.buffer.pop().unwrap();
            let mut after = regex.replace(&before, pattern.1).to_string();
            #[cfg(feature = "debug")]
            { print!("Replacing:\n{}\nwith:\n{}\n", before, after); }
            // If the after doesn't end with newline we append next line
            if (!after.ends_with('\n')) && (tail.len() > 0) {
              after.push_str(&tail.remove(0));
            }
            // Add to the buffer
            for newline in after.split('\n') {
              if newline.len() > 0 {
                self.buffer.push(format!("{}\n", newline));
              }
            }
            selection_after.1 = self.buffer.len();
            self.buffer.append(&mut tail); // And put the tail back on
            break;
          }
        }
      }
      Ok(selection_after)
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
