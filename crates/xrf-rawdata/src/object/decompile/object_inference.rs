use serde::Serialize;

/// Something a decompiled object holds that its OGF did not store, so the decompiler inferred or defaulted it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjectInference {
  /// Surfaces are named after their texture; the authored names are not exported.
  SurfaceNames,
  /// Every surface's compiler shader is the Actor Editor's `default`.
  SurfaceCompilerShaders,
  /// Every surface's game material is the Actor Editor's `default`.
  SurfaceGameMaterials,
  /// The mesh is named `skin`, as the SDK names a skinned object's mesh.
  MeshName,
  /// Every bone is the SDK's default 0.5 long; an OGF stores no length.
  BoneLengths,
  /// Smoothing edge flags come from where welded corners' normals agree.
  Smoothing,
  /// A surface is two-sided where its faces came in mirrored pairs.
  TwoSidedSurfaces,
}
