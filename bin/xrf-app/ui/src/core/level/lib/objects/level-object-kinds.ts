import {
  ELevelObjectGroup,
  ELevelObjectId,
  LEVEL_OBJECT_KINDS,
  LevelObjectGroup,
  LevelObjectId,
  LevelObjectKind,
  LevelObjectKindEntry,
} from "@xrf/ipc/types/xrf-app-levels";
import { assertExhaustive, Nullable } from "@xrf/types";

/** One family of kinds as the Outliner and the Show popover head it. */
export interface ILevelObjectGroupEntry {
  group: LevelObjectGroup;
  label: string;
}

/** Every family of kinds, in the order the viewer lists them. */
export const LEVEL_OBJECT_GROUPS: ReadonlyArray<ILevelObjectGroupEntry> = [
  { group: ELevelObjectGroup.SPAWNED, label: "Spawned" },
  { group: ELevelObjectGroup.CHARACTERS, label: "Characters" },
  { group: ELevelObjectGroup.VOLUMES, label: "Logic volumes" },
  { group: ELevelObjectGroup.COVERS, label: "Smart covers" },
  { group: ELevelObjectGroup.PATHS, label: "Paths" },
  { group: ELevelObjectGroup.NAVIGATION, label: "Navigation" },
  { group: ELevelObjectGroup.PLACEMENTS, label: "Placements" },
  { group: ELevelObjectGroup.LIGHTS, label: "Lights" },
];

const KINDS: ReadonlyMap<LevelObjectKind, LevelObjectKindEntry> = new Map(
  LEVEL_OBJECT_KINDS.map((entry: LevelObjectKindEntry) => [entry.kind, entry])
);

/**
 * @param kind - A kind of level object.
 * @returns Its row of the kind table.
 */
export function getLevelObjectKind(kind: LevelObjectKind): LevelObjectKindEntry {
  return KINDS.get(kind) as LevelObjectKindEntry;
}

/**
 * @param group - A family of kinds.
 * @returns Its kinds, in the kind table's order.
 */
export function listLevelObjectGroupKinds(group: LevelObjectGroup): Array<LevelObjectKindEntry> {
  return LEVEL_OBJECT_KINDS.filter((entry: LevelObjectKindEntry) => entry.group === group);
}

/**
 * @param id - What names a level object.
 * @returns It as one string, which keys the object in sets, maps and tree rows.
 */
export function toLevelObjectKey(id: LevelObjectId): string {
  switch (id.source) {
    case ELevelObjectId.SPAWN:
      return `spawn:${id.index}`;
    case ELevelObjectId.PATROL:
      return `patrol:${id.path}`;
    case ELevelObjectId.PATROL_POINT:
      return `patrolPoint:${id.path}:${id.point}`;
    case ELevelObjectId.GRAPH_POINT:
      return `graphPoint:${id.vertex}`;
    case ELevelObjectId.PLACEMENT:
      return `placement:${id.file}:${id.index}`;
    case ELevelObjectId.LIGHT:
      return `light:${id.index}`;
    default:
      return assertExhaustive(id);
  }
}

/**
 * @param id - What names a level object.
 * @returns Its index among the level's spawned objects, as the renderer and the world name drawn objects; null for an
 *   object the spawn does not place.
 */
export function getLevelSpawnIndex(id: LevelObjectId): Nullable<number> {
  return id.source === ELevelObjectId.SPAWN ? id.index : null;
}
