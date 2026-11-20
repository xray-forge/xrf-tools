use xrf_editable_object::EditableSurface;

/// How the SDK lays out a mesh of the kind of object an OGF was exported from.
///
/// A skeleton's single mesh is `skin`, its UV maps `texture` beside a weight map per bone. A static object's mesh is
/// named after the object, its UV maps `Texture`, the name a surface's vertex map field gives them, and it holds no
/// weight map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MeshKind<'a> {
  /// A skeleton's mesh, its points weighed to these bones, by index.
  Skinned {
    bone_names: &'a [String],
  },
  Static,
}

impl<'a> MeshKind<'a> {
  /// The name the SDK gives a skinned object's mesh.
  const SKINNED_MESH_NAME: &'static str = "skin";

  /// The name of a static object's mesh, which the SDK takes from the object and the OGF does not keep.
  const STATIC_MESH_NAME: &'static str = "mesh";

  /// The name of a skinned mesh's UV maps.
  const SKINNED_UV_MAP_NAME: &'static str = "texture";

  pub(crate) fn get_mesh_name(self) -> &'static str {
    match self {
      Self::Skinned { .. } => Self::SKINNED_MESH_NAME,
      Self::Static => Self::STATIC_MESH_NAME,
    }
  }

  pub(crate) fn get_uv_map_name(self) -> &'static str {
    match self {
      Self::Skinned { .. } => Self::SKINNED_UV_MAP_NAME,
      Self::Static => EditableSurface::UV_MAP_NAME,
    }
  }

  /// The bones a point can be weighed to: none for a static mesh.
  pub(crate) fn get_bone_names(self) -> &'a [String] {
    match self {
      Self::Skinned { bone_names } => bone_names,
      Self::Static => &[],
    }
  }
}
