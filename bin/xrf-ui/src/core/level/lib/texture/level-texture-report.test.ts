import { describe, expect, it } from "@jest/globals";

import { ERenderTextureState, RenderTextureReport } from "@/core/ipc/types/xrf-renderer";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTextureReport, toLevelTextureReport } from "@/core/level/lib/texture/level-texture-report";

const REPORTS: Array<RenderTextureReport> = [
  {
    reference: "wall\\wall_panel",
    state: { height: 512, isExpanded: false, kind: ERenderTextureState.LOADED, layout: "BC1", levels: 10, width: 512 },
  },
  { reference: "terrain\\terrain_zaton", state: { kind: ERenderTextureState.DEFERRED } },
  { reference: "decal\\decal_poteki", state: { kind: ERenderTextureState.LOADING } },
  { reference: "wpn\\gone", state: { kind: ERenderTextureState.MISSING } },
  { reference: "wpn\\broken", state: { kind: ERenderTextureState.FAILED, reason: "Unknown DDS format" } },
];

describe("toLevelTextureReport", () => {
  it("counts a stand-in as uploaded and a load on its way as not yet", () => {
    expect(toLevelTextureReport(REPORTS).uploaded).toBe(4);
  });

  it("reports every reference that is stood in for, with why, in the order they were read", () => {
    expect(toLevelTextureReport(REPORTS).problems).toEqual([
      {
        reason: "Held unread until an enhanced parallax or the heights view reads it",
        reference: "terrain\\terrain_zaton",
      },
      { reason: "Nothing in the mounted roots answers to it", reference: "wpn\\gone" },
      { reason: "Unknown DDS format", reference: "wpn\\broken" },
    ]);
  });

  it("says how a texture was uploaded, and that an expanded one was", () => {
    const report: ILevelTextureReport = toLevelTextureReport([
      ...REPORTS,
      {
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
