import { Nullable } from "@xrf/types";
import { Material } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";

/** A material a staging stands in, over the layout it compiles for, in the scene of the pass drawing it. */
export interface IStagedPipeline {
  readonly material: Material;
  readonly layout: string;
  /** The pass whose target it compiles against, or null for the shadow maps. */
  readonly pass: Nullable<ERendererPass>;
}
