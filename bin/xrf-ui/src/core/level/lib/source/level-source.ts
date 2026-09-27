import { LevelSource } from "@/core/ipc/types/xrf-app";

/**
 * @param source - Where a level was opened from.
 * @returns What names it: the directory's path on disk, or the asset's engine path.
 */
export function describeLevelSource(source: LevelSource): string {
  return source.kind === "directory" ? source.path : source.logicalPath;
}
