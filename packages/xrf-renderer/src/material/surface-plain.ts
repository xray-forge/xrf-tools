import { Nullable } from "@xrf/types";
import { MeshBasicNodeMaterial } from "three/webgpu";

/** What a surface's parts drawn plainly draw with: its own material, and what casts them. */
export interface ISurfacePlain {
  material: MeshBasicNodeMaterial;
  /** What draws them into the sun's shadow maps, or null for a surface that casts none. */
  shadow: Nullable<MeshBasicNodeMaterial>;
}
