// Auto-generated rust bindings. Do not edit it manually.

/** Something a decompiled object holds that its OGF did not store, so the decompiler inferred or defaulted it. */
export enum EObjectInference {
  /** Surfaces are named after their texture; the authored names are not exported. */
  SURFACE_NAMES = "surfaceNames",
  /** Every surface's compiler shader is the Actor Editor's `default`. */
  SURFACE_COMPILER_SHADERS = "surfaceCompilerShaders",
  /** Every surface's game material is the Actor Editor's `default`. */
  SURFACE_GAME_MATERIALS = "surfaceGameMaterials",
  /**
   * The mesh is named `skin`, as the SDK names a skinned object's mesh, or `mesh` in a static object, whose mesh the
   * SDK names after the object.
   */
  MESH_NAME = "meshName",
  /** Every bone is the SDK's default 0.5 long; an OGF stores no length. */
  BONE_LENGTHS = "boneLengths",
  /** Smoothing edge flags come from where welded corners' normals agree. */
  SMOOTHING = "smoothing",
  /** A surface is two-sided where its faces came in mirrored pairs. */
  TWO_SIDED_SURFACES = "twoSidedSurfaces",
  /**
   * The object's own motions are keyed at every frame, linearly between them: the OGF keeps the sampled poses, not
   * the authored curves or which bones kept their world orientation.
   */
  MOTION_CURVES = "motionCurves",
  /**
   * A motion holding one key per bone more than it declares is as long as its keys: `blender-xray` declares one frame
   * fewer than it samples for an action starting between frames, which the engine misreads.
   */
  MOTION_LENGTHS = "motionLengths",
  /**
   * The mesh held one face, which the Level Editor refuses to load (it builds a picking model of every mesh and asserts
   * 4 points and 2 faces), so it comes back halved through its longest edge: the same area in two faces.
   */
  HALVED_FACE = "halvedFace",
}

/** Every `EObjectInference` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type ObjectInference = `${EObjectInference}`;
