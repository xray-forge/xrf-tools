import { describe, expect, it } from "@jest/globals";
import { ERenderTextureGroup, ERenderTextureState, RenderTextureReport } from "@xrf/ipc/types/xrf-renderer";

import { ELevelSurfaceDressing, ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import {
  ILevelTextureReport,
  listLevelSurfaceDressing,
  toLevelTextureReport,
} from "@/core/level/lib/texture/level-texture-report";

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

function mockReport(...dressing: Array<ILevelSurfaceDressing>): ILevelTextureReport {
  return {
    deferred: 0,
    dressing: new Map(dressing.map((it: ILevelSurfaceDressing) => [it.reference, it])),
    problems: dressing
      .filter((it: ILevelSurfaceDressing) => it.reason)
      .map((it: ILevelSurfaceDressing) => ({
        group: ERenderTextureGroup.SURFACES,
        reason: it.reason as string,
        reference: it.reference,
      })),
    uploaded: dressing.length,
  };
}

function uploaded(reference: string, upload: string | null = null): ILevelSurfaceDressing {
  return { reason: null, reference, state: ELevelSurfaceDressing.UPLOADED, upload };
}

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

describe("listLevelSurfaceDressing", () => {
  it("says a texture arrived", () => {
    const [dressing] = listLevelSurfaceDressing(["decal\\decal_poteki"], mockReport(uploaded("decal\\decal_poteki")));

    expect(dressing).toMatchObject({
      reason: null,
      reference: "decal\\decal_poteki",
      state: ELevelSurfaceDressing.UPLOADED,
    });
  });

  // The whole point of the answer: a surface drawn from a checker looks like a blending fault and is not one, and
  // nothing else in the viewer says which of the two happened.
  it("carries the reason a checker stands in", () => {
    const report: ILevelTextureReport = mockReport({
      reason: "Nothing in the mounted roots answers to it",
      reference: "decal\\decal_poteki",
      state: ELevelSurfaceDressing.STOOD_IN,
      upload: null,
    });

    expect(listLevelSurfaceDressing(["decal\\decal_poteki"], report)).toEqual([
      {
        reason: "Nothing in the mounted roots answers to it",
        reference: "decal\\decal_poteki",
        state: ELevelSurfaceDressing.STOOD_IN,
        upload: null,
      },
    ]);
  });

  it("tells a texture nothing has asked for from one that failed", () => {
    const [dressing] = listLevelSurfaceDressing(["wall\\wall_panel"], mockReport());

    expect(dressing.state).toBe(ELevelSurfaceDressing.UNREAD);
    expect(dressing.reason).toBeNull();
  });

  it("reads nothing before a level is open", () => {
    expect(listLevelSurfaceDressing(["wall\\wall_panel"])).toEqual([
      {
        reason: null,
        reference: "wall\\wall_panel",
        state: ELevelSurfaceDressing.UNREAD,
        upload: null,
      },
    ]);
  });

  it("carries how a texture was uploaded through to the entry that names it", () => {
    const [dressing] = listLevelSurfaceDressing(
      ["decal\\decal_poteki"],
      mockReport(uploaded("decal\\decal_poteki", "1 level · linear · clamped · aniso 1"))
    );

    expect(dressing.upload).toBe("1 level · linear · clamped · aniso 1");
  });

  it("answers for each texture in the order the entry names them", () => {
    const dressing: Array<ILevelSurfaceDressing> = listLevelSurfaceDressing(["a", "b", "c"], mockReport(uploaded("b")));

    expect(dressing.map((it) => it.reference)).toEqual(["a", "b", "c"]);
    expect(dressing.map((it) => it.state)).toEqual([
      ELevelSurfaceDressing.UNREAD,
      ELevelSurfaceDressing.UPLOADED,
      ELevelSurfaceDressing.UNREAD,
    ]);
  });
});
