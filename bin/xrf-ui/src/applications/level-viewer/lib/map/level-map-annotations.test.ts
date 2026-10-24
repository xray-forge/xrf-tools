import { describe, expect, it } from "@jest/globals";
import { ELevelAnnotationKind } from "@xrf/ipc/types/xrf-app-levels";

import {
  DEFAULT_LEVEL_MAP_ANNOTATIONS,
  toLevelMapAnnotationsWith,
  toMapAnnotationsRequest,
} from "@/applications/level-viewer/lib/map/level-map-annotations";

describe("level map annotations", () => {
  it("draws none by default", () => {
    expect(DEFAULT_LEVEL_MAP_ANNOTATIONS.kinds).toEqual([]);
    expect(DEFAULT_LEVEL_MAP_ANNOTATIONS.isLabelled).toBe(false);
  });

  it("switches a kind on or off, keeping the dialog's order", () => {
    const withGraph = toLevelMapAnnotationsWith(DEFAULT_LEVEL_MAP_ANNOTATIONS, ELevelAnnotationKind.GRAPH_POINTS, true);
    const withBoth = toLevelMapAnnotationsWith(withGraph, ELevelAnnotationKind.SMART_TERRAINS, true);
    const withoutGraph = toLevelMapAnnotationsWith(withBoth, ELevelAnnotationKind.GRAPH_POINTS, false);

    expect(withBoth.kinds).toEqual([ELevelAnnotationKind.SMART_TERRAINS, ELevelAnnotationKind.GRAPH_POINTS]);
    expect(withoutGraph.kinds).toEqual([ELevelAnnotationKind.SMART_TERRAINS]);
  });

  it("asks the backend for the ground the map covers", () => {
    expect(
      toMapAnnotationsRequest({ kinds: [ELevelAnnotationKind.PATROLS], isLabelled: true }, "map.png", {
        minX: -600,
        minZ: -615,
        maxX: 600,
        maxZ: 585,
      })
    ).toEqual({
      path: "map.png",
      area: { minX: -600, minZ: -615, maxX: 600, maxZ: 585 },
      kinds: [ELevelAnnotationKind.PATROLS],
      isLabelled: true,
    });
  });
});
