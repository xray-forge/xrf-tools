use serde::{Deserialize, Serialize};

/// Something a decompiled object holds that its OGF did not store, so the decompiler inferred or defaulted it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjectInference {
  /// Surfaces are named after their texture; the authored names are not exported.
  SurfaceNames,
  /// Every surface's compiler shader is the Actor Editor's `default`.
  SurfaceCompilerShaders,
  /// Every surface's game material is the Actor Editor's `default`.
  SurfaceGameMaterials,
  /// The mesh is named `skin`, as the SDK names a skinned object's mesh, or `mesh` in a static object, whose mesh the
  /// SDK names after the object.
  MeshName,
  /// Every bone is the SDK's default 0.5 long; an OGF stores no length.
  BoneLengths,
  /// Smoothing edge flags come from where welded corners' normals agree.
  Smoothing,
  /// A surface is two-sided where its faces came in mirrored pairs.
  TwoSidedSurfaces,
  /// The object's own motions are keyed at every frame, linearly between them: the OGF keeps the sampled poses, not
  /// the authored curves or which bones kept their world orientation.
  MotionCurves,
  /// A motion holding one key per bone more than it declares is as long as its keys: `blender-xray` declares one frame
  /// fewer than it samples for an action starting between frames, which the engine misreads.
  MotionLengths,
}
