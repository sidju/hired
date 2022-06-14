// Read in the syntaxes under assets/syntaxes and dump them

use std::env;

fn main() {
  // Get a path to which we can safely write the compressed files
  let out_dir = env::var("OUT_DIR").unwrap();

  // Define to recreate when syntax folder is changed
  println!("cargo:rerun-if-changed=assets/syntaxes");

  // Read in all the syntaxes
  let mut syntax_builder = syntect::parsing::SyntaxSetBuilder::new();
  syntax_builder.add_plain_text_syntax();
  syntax_builder.add_from_folder("assets/syntaxes/", true).unwrap();

  // Dump the created structure
  let mut syntaxes = syntax_builder.build();
  syntect::dumps::dump_to_file(&mut syntaxes, &format!("{}/compressed_syntaxes", &out_dir)).unwrap();


  // Define to recreate when the theme is changed
  println!("cargo:rerun-if-changed=assets/theme.xml");

  // Read in the theme
  let mut theme = syntect::highlighting::ThemeSet::get_theme("assets/theme.xml").unwrap();

  // Dump the created structure
  syntect::dumps::dump_to_file(&mut theme, &format!("{}/compressed_theme", &out_dir)).unwrap();
}
