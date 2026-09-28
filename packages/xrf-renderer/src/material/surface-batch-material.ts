import { Nullable } from "@xrf/types";

import { SurfaceNodeMaterial } from "#/material/surface-node-material";

/** A static batch's shared material, and the texture keys it binds of its own. */
export interface ISurfaceBatchMaterial {
  material: SurfaceNodeMaterial;
  keys: ReadonlyArray<string>;
  /** What draws its cut-out casters into a shadow map, sharing its arrays; null for a variant casting as opaque. */
  shadow: Nullable<SurfaceNodeMaterial>;
  /** The texture keys that shadow binds of its own: the base, where no array holds it. */
  shadowKeys: ReadonlyArray<string>;
  dispose(): void;
}
