import { describe, expect, it } from "@jest/globals";

import { isShadowCascadeRedrawn } from "#/pass/shadow-cascade-redraw";
import { SHADOW_SWAY_INTERVAL } from "#/scene/static/shadow-sway-interval";

const STILL = { isChanged: false, isMoving: false, isSwaying: false, sinceDrawn: 0 };

describe("isShadowCascadeRedrawn", () => {
  it("draws a cascade whose box or casters changed, or over a moving caster, every frame it is due", () => {
    expect(isShadowCascadeRedrawn({ ...STILL, isChanged: true })).toBe(true);
    expect(isShadowCascadeRedrawn({ ...STILL, isMoving: true })).toBe(true);
    expect(isShadowCascadeRedrawn({ ...STILL, isChanged: true, isSwaying: true })).toBe(true);
    expect(isShadowCascadeRedrawn(STILL)).toBe(false);
  });

  // Uncapped, the nearest cascade drew its trees every frame: most of its cost, for a sway too slow to show it.
  it("draws over what sways at most once a sway interval, however fast the frames come", () => {
    const step: number = 1 / 240;
    let since: number = 0;
    let draws: number = 0;

    for (let frame: number = 0; frame < 240; frame += 1) {
      since += step;

      if (isShadowCascadeRedrawn({ ...STILL, isSwaying: true, sinceDrawn: since })) {
        draws += 1;
        since = 0;
      }
    }

    expect(draws).toBeLessThanOrEqual(1 / SHADOW_SWAY_INTERVAL + 20);
    expect(draws).toBeGreaterThanOrEqual(1 / SHADOW_SWAY_INTERVAL);
  });

  // A frame a hair short of the interval would otherwise skip it, leaving every other frame at 60 Hz.
  it("draws every frame at a 60 Hz display whose frames come a little early", () => {
    expect(isShadowCascadeRedrawn({ ...STILL, isSwaying: true, sinceDrawn: SHADOW_SWAY_INTERVAL * 0.95 })).toBe(true);
    expect(isShadowCascadeRedrawn({ ...STILL, isSwaying: true, sinceDrawn: SHADOW_SWAY_INTERVAL / 2 })).toBe(false);
  });
});
