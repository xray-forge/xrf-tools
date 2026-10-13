use std::collections::{BTreeMap, BTreeSet};

use xrf_error::{XrfError, XrfResult};

use crate::param::pass_parameters::PassParameters;

/// The binding declarations of one WGSL module, gathered from the parameters of every pass whose pipelines it makes:
/// each once, in group and binding order. Passes sharing a module may share a binding only by declaring it alike.
#[derive(Default)]
pub struct ShaderBindings {
  declarations: BTreeMap<(u32, u32), String>,
  enables: BTreeSet<&'static str>,
}

impl ShaderBindings {
  pub fn new() -> Self {
    Self::default()
  }

  /// Adds the bindings of `P`.
  ///
  /// # Errors
  ///
  /// Returns an error when `P` declares a binding that parameters added before declare otherwise, or does not declare
  /// one line a binding; nothing of `P` is added then.
  pub fn add<P: PassParameters>(&mut self) -> XrfResult<&mut Self> {
    let lines: Vec<String> = P::list_wgsl_bindings();

    if lines.len() != P::BINDINGS.len() {
      return Err(XrfError::new_invalid_error(format!(
        "'{}' declares {} WGSL bindings for {} bindings",
        P::LAYOUT_KEY,
        lines.len(),
        P::BINDINGS.len()
      )));
    }

    for (index, line) in P::BINDINGS.iter().zip(&lines) {
      if let Some(declared) = self.declarations.get(&(P::GROUP, *index))
        && declared != line
      {
        return Err(XrfError::new_invalid_error(format!(
          "'{}' declares '{line}' where another pass declares '{declared}'",
          P::LAYOUT_KEY
        )));
      }
    }

    self.enables.extend(P::ENABLES);

    for (index, line) in P::BINDINGS.iter().zip(lines) {
      self.declarations.entry((P::GROUP, *index)).or_insert(line);
    }

    Ok(self)
  }

  /// The extensions its bindings need enabled, then the bindings.
  pub fn to_wgsl(&self) -> String {
    let enables = self.enables.iter().map(|name| format!("enable {name};\n"));
    let declarations = self.declarations.values().map(|line| format!("{line}\n"));

    enables.chain(declarations).collect()
  }
}
