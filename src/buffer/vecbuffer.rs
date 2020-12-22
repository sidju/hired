use super::Buffer;
use crate::error_consts::*;

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
impl Buffer for VecBuffer {

  fn len(&self) -> usize {
      self.buffer.len()
  }

  fn verify_index(&self, index: usize) -> Result<(), &'static str>
  {
    if index > self.buffer.len() {
      return Err(INDEX_TOO_BIG);
    }
    Ok(())
  }

  fn verify_selection(&self, selection: (usize, usize)) -> Result<(), &'static str>
  {
    if selection.0 >= selection.1 {
      return Err(SELECTION_EMPTY);
    }
    if selection.1 > self.buffer.len() {
      return Err(INDEX_TOO_BIG);
    }
    Ok(())
  }

  fn set_saved(&mut self) {
    self.saved = true;
  }

  fn saved(&self) -> bool {
    self.saved
  }

  fn get_selection(&self, selection: (usize, usize)) -> Result<&[String], &'static str> {
    self.verify_selection(selection)?;
    Ok(&self.buffer[selection.0 .. selection.1])
  }

  fn insert(&mut self, data: &mut Vec<String>, index: usize) -> Result<(), &'static str>
  {
    self.verify_index(index)?; // TODO: Check this doesn't fail by one
    self.saved = false;
    // To minimise time complexity we split the vector immediately
    let mut tail = self.buffer.split_off(index);
    // And then append both the insert and the split off part
    self.buffer.append(data);
    self.buffer.append(&mut tail);
    Ok(())
  }

  fn delete(&mut self, selection: (usize, usize)) -> Result<(), &'static str>
  {
    self.verify_selection(selection)?;
    self.saved = false;
    let mut tail = self.buffer.split_off(selection.1);
    let _deleted = self.buffer.split_off(selection.0);
    self.buffer.append(&mut tail);
    Ok(())
  }

  fn change(&mut self, data: &mut Vec<String>, selection: (usize, usize)) -> Result<(), &'static str>
  {
    self.verify_selection(selection)?;
    self.saved = false;
    let mut tail = self.buffer.split_off(selection.1);
    let _deleted = self.buffer.split_off(selection.0);
    self.buffer.append(data);
    self.buffer.append(&mut tail);
    Ok(())
  }

  fn mov(&mut self, selection: (usize, usize), index: usize) -> Result<(), &'static str> {
    self.verify_selection(selection)?;
    self.verify_index(index)?;
    // Operation varies depending on moving forward or back
    if index < selection.0 {
      // split out the relevant parts of the buffer
      let mut tail = self.buffer.split_off(selection.1);
      let mut data = self.buffer.split_off(selection.0);
      let mut middle = self.buffer.split_off(index.saturating_sub(1));
      // Reassemble
      self.buffer.append(&mut data);
      self.buffer.append(&mut middle);
      self.buffer.append(&mut tail);
      Ok(())
    }
    else if index >= selection.1 {
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
      Err(MOVE_INTO_SELF)
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
    // Insert it, subtract one if copying to before selection
    let i = if index <= selection.0 {
      index.saturating_sub(1)
    }
    else {
      index
    };
    let mut tail = self.buffer.split_off(i);
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

  // THIS FAR LOOKED OVER!!!

  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool) -> Result<(usize, usize), &'static str>
  {
    use regex::RegexBuilder;
    // ensure that the selection is valid
    self.verify_selection(selection)?;
    self.saved = false; // TODO: actually check if changes are made
    // Compile the regex used to match/extract data
    let regex = RegexBuilder::new(pattern.0)
      .multi_line(true)
      .build()
      .map_err(|_| INVALID_REGEX)
    ?;

    let mut selection_after = selection;
    if global {
      // Cut out the whole selection from buffer
      let mut tail = self.buffer.split_off(selection.1);
      let before = self.buffer.split_off(selection.0);
      // Then join all selected lines together
      let mut tmp = String::new();
      for line in before {
        tmp.push_str(&line);
      }
      // Run the search-replace over it
      let mut after = regex.replace_all(&tmp, pattern.1).to_string();
      // If there is no newline at the end, join next line
      if (!after.ends_with('\n')) && (tail.len() > 0) {
        after.push_str(&tail.remove(0));
      }
      // Split on newlines and add all lines to the buffer
      for line in after.lines() {
        self.buffer.push(format!("{}\n", line));
      }
      // Get the end of the affected area from current bufferlen
      selection_after.1 = self.buffer.len(); 
      // Then put the tail back
      self.buffer.append(&mut tail); 
    }
    else {
      // Check each line for a match. If found, replace and break
      for index in selection.0 .. selection.1 {
        if regex.is_match(&(self.buffer[index])) {
          self.buffer[index] = regex.replace(&self.buffer[index], pattern.1).to_string();
          // If the after doesn't end with newline we append next line
          if ! self.buffer[index].ends_with('\n') {
            if index < self.len() - 1 {
              let tail = self.buffer.remove(index + 1);
              self.buffer[index].push_str(&tail);
            }
            else {
              self.buffer[index].push('\n');
            }
          }
          // If we find more than one newline we split it
          let mut changed_lines = 1;
          let tmp = self.buffer[index].clone();
          for newline in tmp.rmatch_indices('\n') {
            if newline.0 != tmp.len() - 1 {
              // Cut of the relevant part and insert after
              let tail = self.buffer[index].split_off(newline.0 + 1);
              self.buffer.insert(index + 1, tail);
              changed_lines += 1;
            }
          }
          selection_after = (index, index + changed_lines);
          break;
        }
      }
    }
    Ok(selection_after)
  }

  fn find_matching(&self, _pattern: &str)
    -> Result<Vec<usize>, &'static str>
  {
    Err(BUFFER_NOT_IMPLEMENTED)
  }
}
