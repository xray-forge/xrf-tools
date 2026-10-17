import { formatCount } from "@xrf/format";

import { RenderLoadReport } from "@/core/ipc/types/xrf-renderer";

/**
 * @param load - How far the renderer has read the open level, short of ready.
 * @returns What it is doing, as the cover says it: sectors first, then the textures they name, then what is read
 *   beside them.
 */
export function describeLevelLoad(load: RenderLoadReport): string {
  if (load.sectors < load.sectorsTotal) {
    return `Reading sectors, ${formatCount(load.sectors)} of ${formatCount(load.sectorsTotal)}`;
  }

  if (load.textures < load.texturesTotal) {
    return `Uploading textures, ${formatCount(load.textures)} of ${formatCount(load.texturesTotal)}`;
  }

  return "Reading spawned objects, grass, lights and particles";
}
