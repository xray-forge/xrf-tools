use serde::{Deserialize, Serialize};

use crate::provenance::object_provenance::ObjectProvenance;

/// How [`ObjectCompiler`](crate::ObjectCompiler) builds what the Actor Editor's export leaves to it: the choices where
/// matching the editor is not the only sound answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectCompileOptions {
  pub tangents: TangentMethod,
}

/// How a compile gives its vertices tangents and binormals, which a normal map is read against.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TangentMethod {
  /// The Actor Editor's export, NVIDIA's MeshMender (2003): the basis vanilla's OGFs carry, and the one its bump maps,
  /// generated from height maps rather than baked, shade alike under.
  #[default]
  ActorEditor,
  /// MikkTSpace, the basis Blender, `blender-xray`'s OGF export, glTF and the common bakers use: a normal map baked
  /// against it shades as baked.
  MikkTSpace,
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
