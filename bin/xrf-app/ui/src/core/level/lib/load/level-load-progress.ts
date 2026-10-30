import { formatCount } from "@xrf/format";
import { RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";

/**
 * @param load - How far the renderer has read the open level, short of ready.
 * @returns What it is doing, as the cover says it: sectors first, then the textures they name, then the tree
 *   impostors read or baked once those are resident, then what is read beside them.
 */
export function describeLevelLoad(load: RenderLoadReport): string {
  if (load.sectors < load.sectorsTotal) {
    return `Reading sectors, ${formatCount(load.sectors)} of ${formatCount(load.sectorsTotal)}`;
  }

  if (load.textures < load.texturesTotal) {
    return `Uploading textures, ${formatCount(load.textures)} of ${formatCount(load.texturesTotal)}`;
  }

  if (load.impostors < load.impostorsTotal) {
    return `Preparing tree impostors, ${formatCount(load.impostors)} of ${formatCount(load.impostorsTotal)}`;
  }

  return "Reading spawned objects, grass, lights and particles";
}
