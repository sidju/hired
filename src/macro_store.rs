use std::collections::HashMap;
use add_ed::EdError;
use add_ed::macros::{
  Macro,
  MacroGetter,
};

/// Aggregating macro getter
///
/// Tries to get macros, in order, from:
/// - Configuration
/// - TODO: Files in specific path
pub struct MacroStore<'a> {
  pub config_macros: &'a HashMap<String, Macro>,
}

impl<'a> MacroGetter for MacroStore<'a> {
  fn get_macro(&self, name: &str) -> Result<Option<&Macro>, EdError> {
    Ok(self.config_macros.get(name).into())
  }
}
