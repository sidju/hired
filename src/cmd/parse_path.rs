// The function to extract a filename if any in given input
pub fn parse_path(input: &str)
  -> Option<&str>
{
  let trimmed = input.trim_start();
  if trimmed.len() == 0 {
    None
  }
  else {
    Some(trimmed)
  }
} 
