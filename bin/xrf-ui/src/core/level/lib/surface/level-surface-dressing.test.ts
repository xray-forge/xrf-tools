import { describe, expect, it } from "@jest/globals";
import { Texture } from "three";

import {
  describeLevelSurfaceDressing,
  ELevelSurfaceDressing,
  ILevelSurfaceDressing,
  listLevelSurfaceDressing,
} from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { Nullable } from "@/lib/types/general";

/** A lookup holding exactly what it is given, which is all a reader of the set needs. */
function mockLookup(entries: Record<string, ILevelTexture>): ILevelTextureLookup {
  return {
    get: (reference: string): Nullable<ILevelTexture> => entries[reference] ?? null,
    listProblems: () => [],
    size: Object.keys(entries).length,
  };
}

const UPLOADED: ILevelTexture = { isAlphaRead: true, isMipped: true, reason: null, texture: new Texture() };

describe("listLevelSurfaceDressing", () => {
  it("says a texture arrived", () => {
    const [dressing] = listLevelSurfaceDressing(
      ["decal\\decal_poteki"],
      mockLookup({ "decal\\decal_poteki": UPLOADED })
    );

    expect(dressing).toEqual({ reason: null, reference: "decal\\decal_poteki", state: ELevelSurfaceDressing.UPLOADED });
  });

  // The whole point of the answer: a surface drawn from a checker looks like a blending fault and is not one, and
  // nothing else in the viewer says which of the two happened.
  it("says a checker stands in, and why", () => {
    const lookup: ILevelTextureLookup = mockLookup({
      "decal\\decal_poteki": {
        isAlphaRead: true,
        isMipped: true,
        reason: "Nothing in the mounted roots answers to it",
        texture: null,
      },
    });

    expect(listLevelSurfaceDressing(["decal\\decal_poteki"], lookup)).toEqual([
      {
        reason: "Nothing in the mounted roots answers to it",
        reference: "decal\\decal_poteki",
        state: ELevelSurfaceDressing.STOOD_IN,
      },
    ]);
  });

  it("tells a texture nothing has asked for from one that failed", () => {
    const [dressing] = listLevelSurfaceDressing(["wall\\wall_panel"], mockLookup({}));

    expect(dressing.state).toBe(ELevelSurfaceDressing.UNREAD);
    expect(dressing.reason).toBeNull();
  });

  it("reads nothing before a level is open", () => {
    expect(listLevelSurfaceDressing(["wall\\wall_panel"], null)).toEqual([
      { reason: null, reference: "wall\\wall_panel", state: ELevelSurfaceDressing.UNREAD },
    ]);
  });

  // A set answering with neither a texture nor a reason still leaves the surface drawn from nothing.
  it("counts an empty answer as a stand-in rather than an arrival", () => {
    const lookup: ILevelTextureLookup = mockLookup({
      "wall\\wall_panel": { isAlphaRead: false, isMipped: true, reason: null, texture: null },
    });
    const [dressing] = listLevelSurfaceDressing(["wall\\wall_panel"], lookup);

    expect(dressing.state).toBe(ELevelSurfaceDressing.STOOD_IN);
    expect(dressing.reason).toBe("Nothing was uploaded for it");
  });

  it("answers for each texture in the order the entry names them", () => {
    const dressing: Array<ILevelSurfaceDressing> = listLevelSurfaceDressing(
      ["a", "b", "c"],
      mockLookup({ b: UPLOADED })
    );

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
    expect(
      describeLevelSurfaceDressing({
        reason: null,
        reference: "decal\\decal_poteki",
        state: ELevelSurfaceDressing.UPLOADED,
      })
    ).toBe("decal\\decal_poteki");
  });

  it("carries the reason a checker stands in", () => {
    expect(
      describeLevelSurfaceDressing({
        reason: "Nothing in the mounted roots answers to it",
        reference: "decal\\decal_poteki",
        state: ELevelSurfaceDressing.STOOD_IN,
      })
    ).toBe("decal\\decal_poteki · a checker stands in: Nothing in the mounted roots answers to it");
  });

  it("says a texture has not been read yet", () => {
    expect(
      describeLevelSurfaceDressing({ reason: null, reference: "wall\\wall_panel", state: ELevelSurfaceDressing.UNREAD })
    ).toBe("wall\\wall_panel · not read yet");
  });
});
