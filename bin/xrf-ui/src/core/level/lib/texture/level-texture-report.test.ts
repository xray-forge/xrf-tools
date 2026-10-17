import { describe, expect, it } from "@jest/globals";

import { ERenderTextureGroup, ERenderTextureState, RenderTextureReport } from "@/core/ipc/types/xrf-renderer";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTextureReport, toLevelTextureReport } from "@/core/level/lib/texture/level-texture-report";

const REPORTS: Array<RenderTextureReport> = [
  {
    group: ERenderTextureGroup.SURFACES,
    reference: "wall\\wall_panel",
    state: { height: 512, isExpanded: false, kind: ERenderTextureState.LOADED, layout: "BC1", levels: 10, width: 512 },
  },
  {
    group: ERenderTextureGroup.SURFACES,
    reference: "terrain\\terrain_zaton",
    state: { kind: ERenderTextureState.DEFERRED },
  },
  {
    group: ERenderTextureGroup.SURFACES,
    reference: "decal\\decal_poteki",
    state: { kind: ERenderTextureState.LOADING },
  },
  { group: ERenderTextureGroup.SURFACES, reference: "wpn\\gone", state: { kind: ERenderTextureState.MISSING } },
  {
    group: ERenderTextureGroup.SURFACES,
    reference: "wpn\\broken",
    state: { kind: ERenderTextureState.FAILED, reason: "Unknown DDS format" },
  },
];

describe("toLevelTextureReport", () => {
  it("counts a stand-in as uploaded, a load on its way as not yet, and a deferred one apart", () => {
    const report: ILevelTextureReport = toLevelTextureReport(REPORTS);

    expect(report.uploaded).toBe(3);
    expect(report.deferred).toBe(1);
    expect(report.dressing.get("terrain\\terrain_zaton")).toEqual({
      reason: null,
      reference: "terrain\\terrain_zaton",
      state: ELevelSurfaceDressing.DEFERRED,
      upload: null,
    });
  });

  it("reports every reference that is stood in for, with why, in the order they were read, and no deferred one", () => {
    expect(toLevelTextureReport(REPORTS).problems).toEqual([
      {
        group: ERenderTextureGroup.SURFACES,
        reason: "Nothing in the mounted roots answers to it",
        reference: "wpn\\gone",
      },
      { group: ERenderTextureGroup.SURFACES, reason: "Unknown DDS format", reference: "wpn\\broken" },
    ]);
  });

  it("says how a texture was uploaded, and that an expanded one was", () => {
    const report: ILevelTextureReport = toLevelTextureReport([
      ...REPORTS,
      {
        group: ERenderTextureGroup.SURFACES,
        reference: "sky\\sky_night",
        state: {
          height: 256,
          isExpanded: true,
          kind: ERenderTextureState.LOADED,
          layout: "RGBA16F",
          levels: 1,
          width: 256,
        },
      },
    ]);

    expect(report.dressing.get("wall\\wall_panel")).toEqual({
      reason: null,
      reference: "wall\\wall_panel",
      state: ELevelSurfaceDressing.UPLOADED,
      upload: "512×512 · BC1 · 10 levels",
    });
    expect(report.dressing.get("sky\\sky_night")?.upload).toBe("256×256 · RGBA16F · 1 level · expanded");
    expect(report.dressing.get("decal\\decal_poteki")?.state).toBe(ELevelSurfaceDressing.FETCHING);
  });
});
