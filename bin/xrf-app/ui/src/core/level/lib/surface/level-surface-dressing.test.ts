import { describe, expect, it } from "@jest/globals";

import {
  describeLevelSurfaceDressing,
  ELevelSurfaceDressing,
  ILevelSurfaceDressing,
} from "@/core/level/lib/surface/level-surface-dressing";

function uploaded(reference: string, upload: string | null = null): ILevelSurfaceDressing {
  return { reason: null, reference, state: ELevelSurfaceDressing.UPLOADED, upload };
}

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

  it("says a deferred texture waits for what reads it, with no checker", () => {
    expect(
      describeLevelSurfaceDressing({
        reason: null,
        reference: "terrain\\terrain_zaton",
        state: ELevelSurfaceDressing.DEFERRED,
        upload: null,
      })
    ).toBe("terrain\\terrain_zaton · held until an enhanced parallax or the heights view reads it");
  });
});
