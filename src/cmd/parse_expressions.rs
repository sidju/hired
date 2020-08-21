pub fn parse_expressions(input: &str)
  -> Vec<&str>
{
  let separator = match input.chars().next() {
    Some(ch) => ch,
    None => return Vec::new(),
  };
  input[1..]
    .split(separator)
    .collect()
}
