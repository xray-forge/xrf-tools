import { describe, expect, it } from "@jest/globals";

import { ILevelTextureReport, listLevelSurfaceDressing } from "@/core/level/lib/texture/level-texture-report";

import { describeLevelSurfaceDressing, ELevelSurfaceDressing, ILevelSurfaceDressing } from "./level-surface-dressing";

function mockReport(...dressing: Array<ILevelSurfaceDressing>): ILevelTextureReport {
  return {
    dressing: new Map(dressing.map((it: ILevelSurfaceDressing) => [it.reference, it])),
    problems: dressing
      .filter((it: ILevelSurfaceDressing) => it.reason)
      .map((it: ILevelSurfaceDressing) => ({ reason: it.reason as string, reference: it.reference })),
    uploaded: dressing.length,
  };
}

function uploaded(reference: string, upload: string | null = null): ILevelSurfaceDressing {
  return { reason: null, reference, state: ELevelSurfaceDressing.UPLOADED, upload };
}

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
      { reason: null, reference: "wall\\wall_panel", state: ELevelSurfaceDressing.UNREAD, upload: null },
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

describe("describeLevelSurfaceDressing", () => {
  it("says only the reference where there is nothing to report", () => {
    expect(describeLevelSurfaceDressing(uploaded("decal\\decal_poteki"))).toBe("decal\\decal_poteki");
  });

  it("says how an uploaded texture was sampled", () => {
    expect(describeLevelSurfaceDressing(uploaded("decal\\decal_poteki", "1 level · linear · clamped · aniso 1"))).toBe(
      "decal\\decal_poteki · 1 level · linear · clamped · aniso 1"
    );
  });

  it("carries the reason a checker stands in", () => {
    expect(
      describeLevelSurfaceDressing({
        reason: "Nothing in the mounted roots answers to it",
        reference: "decal\\decal_poteki",
        state: ELevelSurfaceDressing.STOOD_IN,
        upload: null,
      })
    ).toBe("decal\\decal_poteki · a checker stands in: Nothing in the mounted roots answers to it");
  });

  it("says a texture has not been read yet", () => {
    expect(
      describeLevelSurfaceDressing({
        reason: null,
        reference: "wall\\wall_panel",
        state: ELevelSurfaceDressing.UNREAD,
        upload: null,
      })
    ).toBe("wall\\wall_panel · not read yet");
  });
});
