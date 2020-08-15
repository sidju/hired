pub fn parse_expressions(input: &str)
  -> Vec<&str>
{
  let separator = match input.chars().next() {
    Some(ch) => ch,
    None => return Vec::new(),
  };
  input
    .split(separator)
    .collect()
}
