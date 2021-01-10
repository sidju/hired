pub fn substitute(input: &str) -> String {
  let mut out = String::new();
  let mut escaped = false;

  // Iterate over the string and replace where matching
  for ch in input.chars() {
    // If escaped check for special characters
    if escaped {
      match ch {
        'n' => out.push('\n'),
        't' => out.push('\t'),
        c => {
          // If no special, insert the false
          // escape and the character after
          out.push('\\');
          out.push(c);
        },
      }
      escaped = false;
    }
    // If not escaped check if is escaping
    else {
      if ch == '\\' {
        escaped = true;
      }
      else {
        out.push(ch);
      }
    }
  }
  out
}

