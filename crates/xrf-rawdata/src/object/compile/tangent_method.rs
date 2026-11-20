use serde::{Deserialize, Serialize};

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
