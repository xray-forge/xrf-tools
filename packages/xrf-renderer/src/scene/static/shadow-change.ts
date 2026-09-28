import { Nullable } from "@xrf/types";
import { Box3 } from "three/webgpu";

/** One change to what the shadow views draw. */
export interface IShadowChange {
  /** Where it was, or null for anywhere. */
  readonly box: Nullable<Box3>;
  /** Whether what came or went there sways or moves, so a kept shadow there looks again for what does. */
  readonly isAnimated: boolean;
}
