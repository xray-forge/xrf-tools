import {
  LEVEL_OBJECT_KINDS,
  LevelObjectKind,
  LevelObjectKindEntry,
  MapAnnotationsWriteRequest,
} from "@xrf/ipc/types/xrf-app-levels";

import { ILevelHelperOptions, isLevelHelperOn } from "@/core/level/lib/objects/level-helper-options";

import { ILevelMapRect } from "./level-map-capture";

/** What of a level map's annotations is drawn, as its dialog holds it. */
export interface ILevelMapAnnotations {
  /** The kinds of level object drawn, in the kind table's order. */
  kinds: ReadonlyArray<LevelObjectKind>;
  /** Whether every object is named, rather than the smart terrains and level changers alone. */
  isLabelled: boolean;
}

/** None drawn: a map opens bare where the view draws no helpers. */
export const DEFAULT_LEVEL_MAP_ANNOTATIONS: ILevelMapAnnotations = {
  kinds: [],
  isLabelled: false,
};

/** The extension a map's annotations are written beside it and exported alone with. */
export const LEVEL_MAP_ANNOTATIONS_EXTENSION: string = "svg";

/**
 * @param helpers - Which helpers the view draws.
 * @returns The annotations a captured map opens on: the kinds whose helpers the view draws, none while it draws none.
 */
export function toLevelMapAnnotationsOf(helpers: ILevelHelperOptions): ILevelMapAnnotations {
  if (!helpers.isShown) {
    return DEFAULT_LEVEL_MAP_ANNOTATIONS;
  }

  return {
    ...DEFAULT_LEVEL_MAP_ANNOTATIONS,
    kinds: LEVEL_OBJECT_KINDS.filter((entry: LevelObjectKindEntry) => isLevelHelperOn(helpers, entry.kind)).map(
      (entry: LevelObjectKindEntry) => entry.kind
    ),
  };
}

/** Kinds a map's annotations switch together, as a family of the dialog does, and whether they are drawn from now on. */
export interface ILevelMapAnnotationKindsChange {
  kinds: ReadonlyArray<LevelObjectKind>;
  isShown: boolean;
}

/**
 * @param annotations - What is drawn.
 * @param change - The kinds switched, and whether they are drawn from now on.
 * @returns The annotations with the kinds switched, in the kind table's order.
 */
export function toLevelMapAnnotationsWith(
  annotations: ILevelMapAnnotations,
  change: ILevelMapAnnotationKindsChange
): ILevelMapAnnotations {
  const { kinds, isShown } = change;

  return {
    ...annotations,
    kinds: LEVEL_OBJECT_KINDS.map((entry: LevelObjectKindEntry) => entry.kind).filter((kind: LevelObjectKind) =>
      kinds.includes(kind) ? isShown : annotations.kinds.includes(kind)
    ),
  };
}

/**
 * @param annotations - What is drawn.
 * @param path - The map's PNG.
 * @param area - The ground the map covers.
 * @returns What the backend is asked to draw beside the map.
 */
export function toMapAnnotationsWriteRequest(
  annotations: ILevelMapAnnotations,
  path: string,
  area: ILevelMapRect
): MapAnnotationsWriteRequest {
  return {
    path,
    area: { minX: area.minX, minZ: area.minZ, maxX: area.maxX, maxZ: area.maxZ },
    kinds: [...annotations.kinds],
    isLabelled: annotations.isLabelled,
  };
}
