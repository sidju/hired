/// Trait that defines a buffer supporting 'ed's base commands
pub trait Buffer {
  /// Check that the index is safe to operate on
  fn verify_index(&self, index: usize) -> Result<(), &str> ;
  /// Check that the selection is safe to operate on
  fn verify_selection(&self, selection: (usize, usize)) -> Result<(), &str> ;
  /// Outputs the selection as a single string
  fn format_selection(&self, selection: (usize, usize), numbered: bool) -> Result<String, &str> ;
  /// Takes a iterator over lines in strings and inserts at given index
  fn insert(&mut self, data: &mut Vec<String>, index: usize) -> Result<(), &str> ;
  /// Deletes the lines in the selection
  fn delete(&mut self, selection: (usize, usize)) -> Result<(), &str> ;
  /// Delete the given selection and insert the given data in its place
  fn change(&mut self, data: &mut Vec<String>, selection: (usize, usize)) -> Result<(), &str> ;
  /// Perform regex search and replace on the selection changing pattern.0 to pattern.1
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool) -> Result<(), &str> ;
  // Find the indices in the selection whose lines match the regex pattern
  // fn find_matching(&self, pattern: &str, selection: (usize, usize)) -> Result<(), &str> ;
  fn len(&self) -> usize ;
}

pub struct VecBuffer {
  buffer: Vec<String>
}
impl VecBuffer {
  pub fn new() -> Self
  {
    Self{
      buffer: Vec::new(),
    }
  }
}
impl Buffer for VecBuffer
{
  fn verify_index(&self, index: usize) -> Result<(), &str>
  {
    if index > self.buffer.len() {
      return Err("Selection overshoots buffer length.");
    }
    Ok(())
  }
  fn verify_selection(&self, selection: (usize, usize)) -> Result<(), &str> 
  {
    if selection.0 >= selection.1 {
      return Err("Selection empty or inverted.");
    }
    if selection.1 > self.buffer.len() {
      return Err("Selection overshoots buffer length.");
    }
    Ok(())
  }
  fn format_selection(&self, selection: (usize, usize), numbered: bool) -> Result<String, &str>
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
    Ok(ret)
  }
  fn insert(&mut self, data: &mut Vec<String>, mut index: usize) -> Result<(), &str>
  {
    if index > self.buffer.len() {
      //0 is valid but needs to be specially handled
      if index != 0 { index += 1; }
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
      Err("Invalid selection.")
    }
  }
  fn delete(&mut self, selection: (usize, usize)) -> Result<(), &str>
  {
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= self.buffer.len() {
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
        if selection.1 > buffer.len() {
          println!("The selection overshoots the buffer.");
        }
      }
      Err("Invalid selection.")
    }
  }
  fn change(&mut self, data: &mut Vec<String>, selection: (usize, usize)) -> Result<(), &str>
  {
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= self.buffer.len() {
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
        if selection.1 > buffer.len() {
          println!("The selection overshoots the buffer.");
        }
      }
      Err("Invalid selection.")
    }
  }
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool) -> Result<(), &str>
  {
    use regex::Regex;
    // ensure that the selection is valid
    if selection.0 < selection.1 && selection.1 <= self.buffer.len() {
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
              print!("Replacing:\n{}\nwith:\n{}",
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
        if selection.1 > buffer.len() {
          println!("The selection overshoots the buffer.");
        }
      }
      Err("Invalid selection.")
    }
  }
  // fn find_matching(&self, pattern: &str, selection: (usize, usize)) -> Result<(), &str> ;
    fn len(&self) -> usize {
        self.buffer.len()
    }
}
