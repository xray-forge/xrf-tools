import { EObjectInference, ObjectInference } from "@xrf/ipc/types/xrf-rawdata";

/** What each of the decompiler's inferences says was guessed rather than read. */
const INFERENCE_LABELS: Readonly<Record<ObjectInference, string>> = {
  [EObjectInference.BONE_LENGTHS]: "Bone lengths: the SDK's 0.5, an OGF stores none",
  [EObjectInference.MESH_NAME]: "Mesh name: `skin` for a skeleton, `mesh` for a static object",
  [EObjectInference.MOTION_CURVES]: "Motion curves: keyed at every frame, the authored curves are not stored",
  [EObjectInference.MOTION_LENGTHS]: "Motion lengths: one frame longer than declared, as the keys hold",
  [EObjectInference.SMOOTHING]: "Smoothing: where the stored normals agree",
  [EObjectInference.SURFACE_COMPILER_SHADERS]: "Compiler shaders: the Actor Editor's `default`",
  [EObjectInference.SURFACE_GAME_MATERIALS]: "Game materials: the Actor Editor's `default`",
  [EObjectInference.SURFACE_NAMES]: "Surface names: after their textures",
  [EObjectInference.TWO_SIDED_SURFACES]: "Two-sided surfaces: where faces came in mirrored pairs",
};

/**
 * @param inference - One of the decompiler's inferences.
 * @returns What it says was guessed rather than read.
 */
export function describeObjectInference(inference: ObjectInference): string {
  return INFERENCE_LABELS[inference];
}
