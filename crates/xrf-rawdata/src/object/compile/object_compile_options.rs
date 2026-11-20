use crate::object::compile::tangent_method::TangentMethod;
use crate::provenance::object_provenance::ObjectProvenance;

/// How [`ObjectCompiler`](crate::ObjectCompiler) builds what the Actor Editor's export leaves to it: the choices where
/// matching the editor is not the only sound answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectCompileOptions {
  pub tangents: TangentMethod,
}

impl ObjectCompileOptions {
  /// The options an object decompiled from an OGF compiles back under, as its `provenance` sidecar records them: the
  /// tangent method its source was made with. An object with no sidecar, an authored one, takes the defaults.
  pub fn following(provenance: Option<&ObjectProvenance>) -> Self {
    Self {
      tangents: provenance.map(|it| it.tangents).unwrap_or_default(),
    }
  }
}
