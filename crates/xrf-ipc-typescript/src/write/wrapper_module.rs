use std::path::Path;

use crate::enumerations::Enumerations;
use crate::write::{GeneratedOutput, TypeOwnership};

/// A module of wrappers the registry spells by hand, a plugin's raw commands or bulk routes, as its parts render.
pub(crate) struct WrapperModule {
  /// The imports of what the wrappers call and are typed through.
  pub import: String,
  /// The enum naming each wrapped command or route.
  pub enumeration: String,
  /// `export const <name>` under its doc, the wrappers object's declaration.
  pub declaration: String,
  /// The object's fields, one wrapper a line.
  pub wrappers: String,
  /// The interface keying each wrapper's signature by its enum member.
  pub signatures: String,
}

impl WrapperModule {
  /// Writes the module at `path`, importing the types its wrappers name from the modules declaring them.
  pub fn write(self, path: &Path, plugin: &str, ownership: &TypeOwnership, enumerations: &Enumerations) {
    // The registry spells these argument types by hand, so they take the same retyping as a Specta signature.
    let wrappers: String = enumerations.rewrite_parameters(&self.wrappers);

    ownership.assert_no_foreign_references(&wrappers, plugin);

    GeneratedOutput::write(
      path,
      &format!(
        "{}\n{}\n{}\n{}\n{} = {{\n{wrappers}}};\n\n{}",
        GeneratedOutput::HEADER,
        self.import,
        ownership.imports_for(&wrappers),
        self.enumeration,
        self.declaration,
        self.signatures
      ),
    );
  }
}
