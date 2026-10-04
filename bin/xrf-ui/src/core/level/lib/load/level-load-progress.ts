import { RenderLoadReport } from "@/core/ipc/types/xrf-renderer";

/**
 * @param load - How far the renderer has read the open level.
 * @returns What it is doing, as the cover says it: sectors first, then the textures they name.
 */
export function describeLevelLoad(load: RenderLoadReport): string {
  return load.sectors < load.sectorsTotal
    ? `Reading sectors, ${load.sectors} of ${load.sectorsTotal}`
    : `Uploading textures, ${load.textures} of ${load.texturesTotal}`;
}
