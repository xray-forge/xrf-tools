import { describe, expect, it } from "@jest/globals";
import { ELevelObjectKind } from "@xrf/ipc/types/xrf-app-levels";

import { DEFAULT_LEVEL_HELPER_OPTIONS } from "@/core/level/lib/objects/level-helper-options";
import {
  DEFAULT_LEVEL_MAP_ANNOTATIONS,
  toLevelMapAnnotationsOf,
  toLevelMapAnnotationsWith,
  toMapAnnotationsWriteRequest,
} from "@/tools/level-viewer/lib/map/level-map-annotations";

describe("level map annotations", () => {
  it("draws none by default", () => {
    expect(DEFAULT_LEVEL_MAP_ANNOTATIONS.kinds).toEqual([]);
    expect(DEFAULT_LEVEL_MAP_ANNOTATIONS.isLabelled).toBe(false);
  });

  it("opens on the kinds whose helpers the view draws, and on none while it draws none", () => {
    const kinds = { [ELevelObjectKind.RESTRICTORS]: false, [ELevelObjectKind.STALKERS]: true };

    expect(toLevelMapAnnotationsOf({ ...DEFAULT_LEVEL_HELPER_OPTIONS, isShown: true, kinds })).toEqual({
      isLabelled: false,
      kinds: [
        ELevelObjectKind.STALKERS,
        ELevelObjectKind.SMART_TERRAINS,
        ELevelObjectKind.ANOMALIES,
        ELevelObjectKind.LEVEL_CHANGERS,
        ELevelObjectKind.CLIMBABLES,
        ELevelObjectKind.PATROLS,
      ],
    });
    expect(toLevelMapAnnotationsOf({ ...DEFAULT_LEVEL_HELPER_OPTIONS, kinds })).toEqual(DEFAULT_LEVEL_MAP_ANNOTATIONS);
  });

  it("switches kinds on or off, keeping the kind table's order", () => {
    const withGraph = toLevelMapAnnotationsWith(DEFAULT_LEVEL_MAP_ANNOTATIONS, [ELevelObjectKind.GRAPH_POINTS], true);
    const withMore = toLevelMapAnnotationsWith(
      withGraph,
      [ELevelObjectKind.SMART_TERRAINS, ELevelObjectKind.STALKERS],
      true
    );
    const withoutGraph = toLevelMapAnnotationsWith(withMore, [ELevelObjectKind.GRAPH_POINTS], false);

    expect(withMore.kinds).toEqual([
      ELevelObjectKind.STALKERS,
      ELevelObjectKind.SMART_TERRAINS,
      ELevelObjectKind.GRAPH_POINTS,
    ]);
    expect(withoutGraph.kinds).toEqual([ELevelObjectKind.STALKERS, ELevelObjectKind.SMART_TERRAINS]);
  });

  it("asks the backend for the ground the map covers", () => {
    expect(
      toMapAnnotationsWriteRequest({ kinds: [ELevelObjectKind.PATROLS], isLabelled: true }, "map.png", {
        minX: -600,
        minZ: -615,
        maxX: 600,
        maxZ: 585,
      })
    ).toEqual({
      path: "map.png",
      area: { minX: -600, minZ: -615, maxX: 600, maxZ: 585 },
      kinds: [ELevelObjectKind.PATROLS],
      isLabelled: true,
    });
  });
});
