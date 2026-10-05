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
  /// Returns an error when `P` declares a binding that parameters added before declare otherwise.
  pub fn add<P: PassParameters>(&mut self) -> XrfResult<&mut Self> {
    let wgsl: String = P::get_wgsl_bindings();

    self.enables.extend(P::ENABLES);

    for (index, line) in P::BINDINGS.iter().zip(wgsl.lines()) {
      match self.declarations.get(&(P::GROUP, *index)) {
        Some(declared) if declared != line => {
          return Err(XrfError::new_invalid_error(format!(
            "'{}' declares '{line}' where another pass declares '{declared}'",
            P::LAYOUT_KEY
          )));
        }
        Some(_) => {}
        None => {
          self.declarations.insert((P::GROUP, *index), line.to_string());
        }
      }
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
