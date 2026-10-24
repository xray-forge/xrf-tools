// Auto-generated rust bindings. Do not edit it manually.

import { AssetSource, AssetTextureDescriptor } from "@/core/ipc/types/xrf-app-core";
import { XrayMaterialDescriptor, XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { XrayAsset, XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { VisualDependencies, VisualDescription } from "@/core/ipc/types/xrf-visual";

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
};
