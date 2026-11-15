import { formatCount } from "@xrf/format";
import { RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";

/** What a level's load does once its sectors and their textures are in, as the cover says it. */
const LEVEL_LOAD_REST: string = "Reading spawned objects, grass, lights and particles";

/**
 * @param load - How far the renderer has read the open level, short of ready.
 * @param rest - What the load does once the sectors and textures are in: a level's spawned objects and the rest read
 *   beside them, or whatever else draws as models.
 * @returns What it is doing, as the cover says it: sectors first, then the textures they name, then `rest`.
 */
export function describeLevelLoad(load: RenderLoadReport, rest: string = LEVEL_LOAD_REST): string {
  if (load.sectors < load.sectorsTotal) {
    return `Reading sectors, ${formatCount(load.sectors)} of ${formatCount(load.sectorsTotal)}`;
  }

  if (load.textures < load.texturesTotal) {
    return `Uploading textures, ${formatCount(load.textures)} of ${formatCount(load.texturesTotal)}`;
  }

  return rest;
}
