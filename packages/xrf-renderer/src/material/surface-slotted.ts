import { Nullable } from "@xrf/types";

import { TSurfaceArrayTargets } from "#/material/surface-array-targets";
import { TSurfaceSlotTargets } from "#/material/surface-slot-targets";

/** A material whose slots a shared shader samples. */
export interface ISurfaceSlotted {
  surfaceSlots: Nullable<TSurfaceSlotTargets>;
  /** The arrays its array slots sample, or null for a material sampling every slot from a texture of its own. */
  surfaceArrays: Nullable<TSurfaceArrayTargets>;
}
