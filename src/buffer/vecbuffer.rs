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

  fn set_saved(&mut self) {
    self.saved = true;
  }

  fn saved(&self) -> bool {
    self.saved
  }

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
    if index <= selection.0 {
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

  fn find_matching(&self, pattern: &str, selection: (usize, usize))
    -> Result<Vec<usize>, &'static str>
  {
    use regex::RegexBuilder;
    self.verify_selection(selection)?;
    let regex = RegexBuilder::new(pattern)
      .multi_line(true)
      .build()
      .map_err(|_| INVALID_REGEX)
    ?;
    let mut ret = Vec::new();
    for index in selection.0 .. selection.1 {
      if regex.is_match(&(self.buffer[index])) {
        ret.push(index);
      }
    }
    Ok(ret)
  }
}

// Tests of the trickier cases
#[cfg(test)]
mod tests {
  use super::*;
  use super::super::Buffer;

  fn create_data() -> Vec<String> {
    vec![
      "1\n".to_string(),
      "2\n".to_string(),
      "3\n".to_string(),
      "4\n".to_string(),
      "5\n".to_string(),
      "6\n".to_string()
    ]
  }

  fn create_buffer() -> VecBuffer {
    let mut buf = VecBuffer::new();
    buf.insert(&mut create_data(), 0).unwrap();
    buf
  }

  #[test]
  fn verify_index() {
    let buffer = create_buffer();
    assert_eq!(Ok(()), buffer.verify_index(buffer.len()));
    assert_eq!(Ok(()), buffer.verify_index(0));
    assert_eq!(Err(INDEX_TOO_BIG), buffer.verify_index(buffer.len() + 1));
  }

  #[test]
  fn verify_selection() {
    let buffer = create_buffer();
    assert_eq!(Ok(()), buffer.verify_selection((0, buffer.len())));
    assert_eq!(Err(INDEX_TOO_BIG), buffer.verify_selection((0, buffer.len() + 1)));
    assert_eq!(Err(SELECTION_EMPTY), buffer.verify_selection((1, 0)));
  }

  #[test]
  fn saved() {
    let mut buffer = VecBuffer::new();
    assert!(buffer.saved());
    buffer.insert(&mut vec!["0".to_string()], 0).unwrap();
    assert!(!buffer.saved());
    buffer.set_saved();
    assert!(buffer.saved());
  }

  #[test]
  fn get_selection() {
    let data = create_data();
    let mut buffer = VecBuffer::new();
    buffer.insert(&mut data.clone(), 0).unwrap();
    assert_eq!(
      buffer.get_selection((0, 2)),
      Ok(&data[0 .. 2])
    );
  }

  #[test]
  fn insert() {
    // Todo, verify position of insert specifically.
    let data = create_data();
    let mut compare = data.clone();
    let mut buffer = VecBuffer::new();
    buffer.insert(&mut data.clone(), 0).unwrap();
    // Check that it inserts as expected
    let mut tmp = compare.split_off(3);
    compare.append(&mut data.clone());
    compare.append(&mut tmp);
    buffer.insert(&mut data.clone(), 3).unwrap();
    assert_eq!(Ok(&compare[..]), buffer.get_selection((0, buffer.len())));
  }

  #[test]
  fn delete() {
    let mut buffer = VecBuffer::new();
    let mut data = create_data();
    buffer.insert(&mut data.clone(), 0).unwrap();
    buffer.delete((0,3)).unwrap();
    let mut tail = data.split_off(3);
    let _deleted = data.split_off(0);
    data.append(&mut tail);
    assert_eq!(
      Ok(&data[..]),
      buffer.get_selection((0, buffer.len()))
    );
  }

  #[test]
  fn change() {
    let mut buffer1 = create_buffer();
    let mut buffer2 = create_buffer();
    let data = create_data();

    // Change should be the same as delete and insert
    // It exists only to allow optimisations
    buffer1.change(&mut data.clone(), (2,4)).unwrap();
    buffer2.delete((2,4)).unwrap();
    buffer2.insert(&mut data.clone(), 2).unwrap();
    assert_eq!(
      buffer1.get_selection((0, buffer1.len())).unwrap(),
      buffer2.get_selection((0, buffer2.len())).unwrap()
    );
  }

  #[test]
  fn mov() {
    // Create identical buffers and verify the move by
    // by comparing it to get_selection and insert
    let mut tmp = create_data();
    let mut buffer1 = VecBuffer::new();
    let mut buffer2 = VecBuffer::new();
    buffer1.insert(&mut tmp.clone(), 0).unwrap();
    buffer2.insert(&mut tmp, 0).unwrap();
    // forward
    buffer1.mov((0,3), 5).unwrap();
    let mut tmp = buffer2.get_selection((0,3)).unwrap().to_vec();
    buffer2.insert(&mut tmp, 5).unwrap();
    buffer2.delete((0,3)).unwrap();
    assert_eq!(
      buffer1.get_selection((0, buffer1.len())),
      buffer2.get_selection((0, buffer2.len()))
    );

    // Recreate buffers and test backward
    let mut tmp = create_data();
    let mut buffer1 = VecBuffer::new();
    let mut buffer2 = VecBuffer::new();
    buffer1.insert(&mut tmp.clone(), 0).unwrap();
    buffer2.insert(&mut tmp, 0).unwrap();
    // backward
    buffer1.mov((3,5), 2).unwrap();
    let mut tmp = buffer2.get_selection((3,5)).unwrap().to_vec();
    buffer2.delete((3,5)).unwrap();
    buffer2.insert(&mut tmp, 2 - 1).unwrap();
    assert_eq!(
      buffer1.get_selection((0, buffer1.len())),
      buffer2.get_selection((0, buffer2.len()))
    );

    // Verify the edges of moving into self
    let mut buffer = create_buffer();
    assert_eq!(
      Err(MOVE_INTO_SELF),
      buffer.mov((1,4), 2)
    );
    assert_eq!(
      Err(MOVE_INTO_SELF),
      buffer.mov((1,4), 3)
    );
    assert_eq!(
      Ok(()),
      buffer.mov((1,4), 4)
    );
    assert_eq!(
      Ok(()),
      buffer.mov((1,4), 1)
    );
  }

  #[test]
  fn copy() {
    // Compare with get_selection and insert
    let data = create_data();
    let mut buffer1 = VecBuffer::new();
    let mut buffer2 = VecBuffer::new();
    buffer1.insert(&mut data.clone(), 0).unwrap();
    buffer2.insert(&mut data.clone(), 0).unwrap();
    // copy forward
    buffer1.copy((1,3), 4).unwrap();
    let mut tmp = buffer2.get_selection((1,3)).unwrap().to_vec();
    buffer2.insert(&mut tmp, 4).unwrap();
    assert_eq!(
      buffer1.get_selection((0, buffer1.len())),
      buffer2.get_selection((0, buffer2.len()))
    );
  }

  #[test]
  fn join() {
    let mut data = create_data();
    let mut buffer = VecBuffer::new();
    buffer.insert(&mut data.clone(), 0).unwrap();
    // First join two lines together, 1 and 2
    buffer.join((1,3)).unwrap();
    let tmp = data.remove(2);
    data[1].pop(); // Get rid of the newline
    data[1].push_str(&tmp);
    assert_eq!(
      &data[..],
      buffer.get_selection((0,buffer.len())).unwrap()
    );
    // Then join the whole buffer
    buffer.join((0,buffer.len())).unwrap();
    let mut tmp = String::new();
    for line in data {
      tmp.push_str(&line[0 .. line.len() - 1]);
    }
    tmp.push('\n');
    assert_eq!(
      &tmp,
      &buffer.get_selection((0,1)).unwrap()[0]
    );
  }

  #[test]
  fn search_replace() {
    let mut data = vec!["test line\n".to_string(), "also a test line\n".to_string()];
    let mut buffer = VecBuffer::new();
    buffer.insert(&mut data.clone(), 0).unwrap();
    // Verify non-global against replacen
    buffer.search_replace(("test", "best"), (0, buffer.len()), false).unwrap();
    let line = data[0].replacen("test", "best", 1);
    data[0] = line;
    assert_eq!(
      &data[..],
      buffer.get_selection((0,buffer.len())).unwrap()
    );
    // And verify global using replace
    buffer.search_replace(("line", "string"), (0, buffer.len()), true).unwrap();
    data = data.iter().map(|line| line.replace("line", "string")).collect();
    assert_eq!(
      &data[..],
      buffer.get_selection((0,buffer.len())).unwrap()
    );
  }

  #[test]
  fn find_matching() {
    let mut data = vec![
      "test line\n".to_string(),
      "hard to write random works\n".to_string(),
      "to test over\n".to_string(),
      "None the less, it is needed\n".to_string(),
      "in this case\n".to_string()
    ];
    let mut buffer = VecBuffer::new();
    buffer.insert(&mut data, 0).unwrap();
    // Verify the return using known indices
    assert_eq!(
      vec![0,2],
      buffer.find_matching("test", (0, buffer.len())).unwrap()
    );

  }
}
