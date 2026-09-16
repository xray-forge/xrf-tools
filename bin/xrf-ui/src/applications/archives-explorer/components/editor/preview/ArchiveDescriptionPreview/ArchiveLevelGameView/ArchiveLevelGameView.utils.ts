import { ArchiveLevelGameSpawn } from "@/core/ipc/types/xrf-app";
import { Nullable } from "@/lib/types/general";

import { formatCount } from "../ArchiveDescriptionPreview.utils";

/**
 * What one group of respawn points spawns.
 *
 * @param spawn - Group to name.
 * @returns The engine's own name for the kind, or the stored number for one it does not name.
 */
export function describeSpawnKind(spawn: ArchiveLevelGameSpawn): string {
  if (!spawn.label) {
    return `Kind ${spawn.kind}`;
  }

  return spawn.label.charAt(0).toUpperCase() + spawn.label.slice(1);
}

/**
 * What qualifies a group of respawn points, which is the preset only some of them name.
 *
 * @param spawn - Group to describe.
 * @returns A phrase for the presets in the group, or null for a kind that names none.
 */
export function describeSpawnProfiles(spawn: ArchiveLevelGameSpawn): Nullable<string> {
  return spawn.profiled ? `${formatCount(spawn.profiled)} of them name a spawn preset` : null;
}
