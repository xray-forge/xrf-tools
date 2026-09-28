import { IGrassCacheBuffers } from "#/scene/grass/grass-cache-buffers";
import { IGrassItemBuffers } from "#/scene/grass/grass-item-buffers";
import { IGrassLevelBuffers } from "#/scene/grass/grass-level-buffers";

/** Everything the planting reads and writes: the level's, the ring's, and the frame's. */
export interface IGrassBuffers {
  level: IGrassLevelBuffers;
  cache: IGrassCacheBuffers;
  items: IGrassItemBuffers;
}
