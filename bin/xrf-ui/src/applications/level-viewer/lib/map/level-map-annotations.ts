import { ELevelAnnotationKind, MapAnnotationsRequest } from "@xrf/ipc/types/xrf-app-levels";

import { ILevelMapRect } from "./level-map-capture";

/** What of a level map's annotations is drawn, as its dialog holds it. */
export interface ILevelMapAnnotations {
  kinds: ReadonlyArray<ELevelAnnotationKind>;
  /** Whether every volume and patrol is named, rather than the smart terrains and level changers alone. */
  isLabelled: boolean;
}

/** The kinds offered, in the order the dialog lists them, each with its label. */
export const LEVEL_MAP_ANNOTATION_KINDS: ReadonlyArray<[ELevelAnnotationKind, string]> = [
  [ELevelAnnotationKind.SMART_TERRAINS, "Smart terrains"],
  [ELevelAnnotationKind.RESTRICTORS, "Space restrictors"],
  [ELevelAnnotationKind.ANOMALIES, "Anomalies"],
  [ELevelAnnotationKind.LEVEL_CHANGERS, "Level changers"],
  [ELevelAnnotationKind.PATROLS, "Patrol paths"],
  [ELevelAnnotationKind.GRAPH_POINTS, "Graph points"],
];

/** None drawn: a map opens bare, and each kind is switched on over it. */
export const DEFAULT_LEVEL_MAP_ANNOTATIONS: ILevelMapAnnotations = {
  kinds: [],
  isLabelled: false,
};

/** The extension a map's annotations are written beside it and exported alone with. */
export const LEVEL_MAP_ANNOTATIONS_EXTENSION: string = "svg";

/**
 * @param annotations - What is drawn.
 * @param kind - A kind to switch.
 * @param isShown - Whether it is drawn from now on.
 * @returns The annotations with `kind` drawn or not, in the order the dialog lists them.
 */
export function toLevelMapAnnotationsWith(
  annotations: ILevelMapAnnotations,
  kind: ELevelAnnotationKind,
  isShown: boolean
): ILevelMapAnnotations {
  return {
    ...annotations,
    kinds: LEVEL_MAP_ANNOTATION_KINDS.map(([it]) => it).filter((it) =>
      it === kind ? isShown : annotations.kinds.includes(it)
    ),
  };
}

/**
 * @param annotations - What is drawn.
 * @param path - The map's PNG.
 * @param area - The ground the map covers.
 * @returns What the backend is asked to draw beside the map.
 */
export function toMapAnnotationsRequest(
  annotations: ILevelMapAnnotations,
  path: string,
  area: ILevelMapRect
): MapAnnotationsRequest {
  return {
    path,
    area: { minX: area.minX, minZ: area.minZ, maxX: area.maxX, maxZ: area.maxZ },
    kinds: [...annotations.kinds],
    isLabelled: annotations.isLabelled,
  };
}
