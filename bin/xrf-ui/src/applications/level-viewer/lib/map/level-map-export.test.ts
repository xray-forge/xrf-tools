import { describe, expect, it } from "@jest/globals";

import {
  DEFAULT_LEVEL_MAP_EXPORT,
  ILevelMapExportSettings,
  toLevelMapExportSettings,
  toLevelMapExportSize,
  toMapExportRequest,
} from "@/applications/level-viewer/lib/map/level-map-export";
import { EMapExportAnnotations } from "@/core/ipc/types/xrf-app-levels";

describe("level map export", () => {
  it("keeps an SVG only while the annotations stand alone", () => {
    const svg: ILevelMapExportSettings = toLevelMapExportSettings(DEFAULT_LEVEL_MAP_EXPORT, {
      annotations: EMapExportAnnotations.ALONE,
      kind: "svg",
    });

    expect(svg.kind).toBe("svg");
    expect(toLevelMapExportSettings(svg, { annotations: EMapExportAnnotations.OVER }).kind).toBe("png");
    expect(toLevelMapExportSettings(svg, { annotations: null }).kind).toBe("png");
    expect(toLevelMapExportSettings(DEFAULT_LEVEL_MAP_EXPORT, { kind: "svg" }).kind).toBe("png");
  });

  // The backend sizes its export by the same cases (`map_export.rs`).
  it("sizes an export as the backend does, a DDS in whole blocks", () => {
    expect(toLevelMapExportSize(4096, 2048, 2048, "png")).toEqual([2048, 1024]);
    expect(toLevelMapExportSize(1000, 333, 500, "png")).toEqual([500, 167]);
    expect(toLevelMapExportSize(1000, 333, 500, "dds")).toEqual([500, 168]);
    expect(toLevelMapExportSize(512, 512, 4096, "dds")).toEqual([512, 512]);
  });

  it("asks for no more than the capture's longer side", () => {
    expect(toMapExportRequest(DEFAULT_LEVEL_MAP_EXPORT, "map.png", "map.dds", 1024).longSide).toBe(1024);
  });
});
