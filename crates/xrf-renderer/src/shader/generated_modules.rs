use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, ShaderBindings, ShaderDeclarations, ShaderType};

/// The modules under `shaders/generated/` as the passes declare them, each group its own: the structs shaders share
/// with Rust, in the order they were declared, and each module's bindings, gathered from the parameters of every pass
/// drawing with it.
#[derive(Default)]
pub struct GeneratedModules {
  structs: ShaderDeclarations,
  bindings: Vec<(&'static str, ShaderBindings)>,
}

impl GeneratedModules {
  /// The module the shared structs are written to.
  const STRUCTS: &'static str = "generated/structs";

  /// What each module of bindings opens with, for the structs its uniforms hold.
  const STRUCTS_IMPORT: &'static str = "#import \"generated/structs\"\n\n";

  pub fn new() -> Self {
    Self::default()
  }

  /// Adds a struct shaders share with Rust, and every struct it holds.
  pub fn declare<T: ShaderType>(&mut self) -> &mut Self {
    self.structs.declare::<T>();
    self
  }

  /// Adds the structs the uniforms and storage arrays of `P` hold.
  pub fn declare_held<P: PassParameters>(&mut self) -> &mut Self {
    P::declare(&mut self.structs);
    self
  }

  /// Adds the bindings of `P` to `module`, made on its first parameters.
  ///
  /// # Errors
  ///
  /// Returns an error when `P` declares a binding that parameters added to `module` before declare otherwise.
  pub fn add<P: PassParameters>(&mut self, module: &'static str) -> XrfResult<&mut Self> {
    let index: usize = match self.bindings.iter().position(|(name, _)| *name == module) {
      Some(index) => index,
      None => {
        self.bindings.push((module, ShaderBindings::new()));
        self.bindings.len() - 1
      }
    };

    self.bindings[index].1.add::<P>()?;

    Ok(self)
  }

  /// Every module by name, the structs first, as its WGSL.
  pub fn to_modules(&self) -> Vec<(&'static str, String)> {
    let bindings = self
      .bindings
      .iter()
      .map(|(name, bindings)| (*name, format!("{}{}", Self::STRUCTS_IMPORT, bindings.to_wgsl())));

    std::iter::once((Self::STRUCTS, self.structs.to_wgsl()))
      .chain(bindings)
      .collect()
  }
}
