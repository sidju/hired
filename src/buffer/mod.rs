// Include the buffer implementations
mod vecbuffer;
pub use vecbuffer::*;

/// Trait that defines a buffer supporting 'ed's base commands
pub trait Buffer {

  /// Return the number of lines stored in the buffer
  fn len(&self)
    -> usize
  ;

  /// Check that the index is safe to operate on
  fn verify_index(&self, index: usize)
    -> Result<(), &'static str>
  ;

  /// Check that the selection is safe to operate on
  fn verify_selection(&self, selection: (usize, usize))
    -> Result<(), &'static str>
  ;

  /// Inform the buffer that it has been saved
  fn set_saved(&mut self);

  /// Returns true if no changes have been made since last saving
  fn saved(&self)
    -> bool
  ;

  /// Return the given selection without any formatting
  fn get_selection(&self, selection: (usize, usize))
    -> Result<&[String], &'static str>
  ;

  /// Takes a iterator over lines in strings and inserts at given index
  fn insert(&mut self, data: &mut Vec<String>, index: usize)
    -> Result<(), &'static str>
  ;

  /// Deletes the lines in the selection
  fn delete(&mut self, selection: (usize, usize))
    -> Result<(), &'static str>
  ;

  /// Delete the given selection and insert the given data in its place
  fn change(&mut self, data: &mut Vec<String>, selection: (usize, usize))
    -> Result<(), &'static str>
  ;

  /// Move selection to index
  fn mov(&mut self, selection: (usize, usize), index: usize)
    -> Result<(), &'static str>
  ;

  /// Copy selection to index
  fn copy(&mut self, selection: (usize, usize), index: usize)
    -> Result<(), &'static str>
  ;

  /// Join all lines in selection into one line
  fn join(&mut self, selection: (usize, usize))
    -> Result<(), &'static str>
  ;

  /// Perform regex search and replace on the selection changing pattern.0 to pattern.1
  /// Returns selection, since it may delete or add lines
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool)
    -> Result<(usize, usize), &'static str>
  ;

  /// Return the indices in the selection whose lines contain the regex pattern
  fn find_matching(&self, pattern: &str, selection: (usize, usize))
    -> Result<Vec<usize>, &'static str>
  ;

}
