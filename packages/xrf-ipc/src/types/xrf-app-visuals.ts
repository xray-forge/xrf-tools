// Auto-generated rust bindings. Do not edit it manually.

import { AssetSource, AssetTextureDescriptor } from "#/types/xrf-app-core";
import { XrayMaterialDescriptor, XraySurfaceDescriptor } from "#/types/xrf-material";
import { ObjectInference } from "#/types/xrf-rawdata";
import { XrayAsset, XrayRoots } from "#/types/xrf-vfs";
import { VisualDependencies, VisualDescription } from "#/types/xrf-visual";

/** One bone of an object as it was authored: its physics as the Actor Editor sets them. */
export type SelectedObjectBone = {
  name: string;
  /** The bone it hangs from, empty for the root. */
  parent: string;
  gameMaterial: string;
  /** The shape it collides as: `None`, `Box`, `Sphere` or `Cylinder`. */
  shape: string;
  /** How it joins its parent: `Rigid`, `Cloth`, `Joint`, `Wheel`, `None` or `Slider`. */
  joint: string;
  /** Its mass in kilograms, or none where the object stores none. */
  mass: number | null;
};

/**
 * What an SDK object holds that the model compiled from it does not show: its surfaces and bones as authored, its
 * mesh's points and smoothing, and where a decompiled one came from.
 */
export type SelectedObjectDescription = {
  surfaces: Array<SelectedObjectSurface>;
  bones: Array<SelectedObjectBone>;
  meshCount: number;
  /** Points over every mesh, before the export splits them at seams and hard edges. */
  pointCount: number;
  faceCount: number;
  /** Distinct smoothing values over every face, `0` where no mesh stores smoothing. */
  smoothingGroupCount: number;
  motionReferences: Array<string>;
  /** Whether the object carries motions of its own, which the compile writes into the OGF. */
  hasEmbeddedMotions: boolean;
  classScript: string | null;
  lods: string | null;
  /** Where a decompiled object came from, or none for an authored one. */
  provenance: SelectedObjectProvenance | null;
};

/** Where a decompiled object came from, as its sidecar says, so it is never taken for an authored one. */
export type SelectedObjectProvenance = {
  /** The compiled file it was decompiled from, as the tool was given it. */
  source: string;
  sourceSha256: string;
  tool: string;
  toolVersion: string;
  /** Seconds since the Unix epoch. */
  decompiledAt: number;
  /** What the decompiler inferred or defaulted rather than read. */
  inferred: Array<ObjectInference>;
};

/**
 * One surface of an object as it was authored, with what its OGF keeps only part of: the compiler shader and game
 * material the export drops, and whether it is drawn from both sides.
 */
export type SelectedObjectSurface = {
  name: string;
  texture: string;
  /** The engine shader, which the OGF keeps. */
  shader: string;
  /** The shader the level compiler lights it by, which the OGF does not keep. */
  compilerShader: string;
  /** What it sounds and breaks like, which the OGF does not keep. */
  gameMaterial: string;
  isTwoSided: boolean;
  /** Faces drawn with it, over every mesh. */
  faceCount: number;
};

/** What the viewer is showing, paired with where it came from: described once at the open and answered as held. */
export type SelectedVisualDescription = {
  source: AssetSource;
  /** The roots the visual was opened in, which every later read searches. */
  roots: XrayRoots;
  description: VisualDescription;
  /** What the visual's own references came to, decided at open so a read is a lookup rather than a search. */
  dependencies: VisualDependencies;
  /** What each located texture file is, keyed by the logical path that located it. */
  textures: { [key in string]: AssetTextureDescriptor };
  /** What the renderer builds for each declared texture, keyed by the reference as the mesh declares it. */
  materials: { [key in string]: XrayMaterialDescriptor };
  /** How the renderer draws each submesh, in the order the model declares them. */
  surfaces: Array<XraySurfaceDescriptor>;
  /** A `textures.ltx` the searched roots hold, or `None`, since its declarations are not read. */
  texturesLtx: XrayAsset | null;
  /** What the SDK object the model was compiled from holds beyond it, or `None` for a compiled model opened as it is. */
  object: SelectedObjectDescription | null;
};
