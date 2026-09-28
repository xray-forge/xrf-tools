import { Node } from "three/webgpu";

import { IGrassBuffers } from "#/scene/grass/grass-buffers";
import { createGrassCull } from "#/scene/grass/grass-cull.tsl";
import { createGrassDecompress } from "#/scene/grass/grass-decompress.tsl";
import { IGrassPlanting } from "#/scene/grass/grass-planting";
import { createGrassRank, createGrassScheduleClear, createGrassSelect } from "#/scene/grass/grass-schedule.tsl";
import { createGrassArrange, createGrassClear, createGrassScatter } from "#/scene/grass/grass-sorting.tsl";
import { GrassRingUniforms } from "#/uniforms/grass-ring-uniforms";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

/**
 * The passes planting the grass, as `CDetailManager` does. A ring of slots around the camera is its cache: a slot
 * coming into it is planted once, the nearest first within a budget a frame, and every frame culls what the ring
 * holds, then sorts what it keeps into a draw a model.
 *
 * @param buffers - What the grass is planted from and into.
 * @param uniforms - Where the camera stands and what the planting is set to.
 * @param discard - `r_ssaDISCARD`, the screen area below which the engine drops what it would draw.
 * @returns The passes; those a thread a cell or an item take their dispatch sizes before each frame's.
 */
export function createGrassPlanting(
  buffers: IGrassBuffers,
  uniforms: GrassUniforms,
  discard: Node<"float">
): IGrassPlanting {
  const ring: GrassRingUniforms = new GrassRingUniforms();

  return {
    arrange: createGrassArrange(buffers.level),
    clear: createGrassClear(buffers.level),
    clearSchedule: createGrassScheduleClear(buffers.cache),
    cull: createGrassCull(buffers, uniforms, ring, discard),
    decompress: createGrassDecompress(buffers, uniforms, ring),
    rank: createGrassRank(buffers.cache, uniforms, ring),
    ring,
    scatter: createGrassScatter(buffers),
    select: createGrassSelect(buffers.cache),
  };
}
