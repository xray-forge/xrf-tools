import { describe, expect, it } from "@jest/globals";
import { Scene } from "three/webgpu";

import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { StaticDraws } from "#/scene/static/static-draws";
import { IStaticUpcoming } from "#/scene/static/static-upcoming";
import { EStaticPool, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** Static draws over buffers of two slots and two places, with what the queue brings. */
function createDraws(upcoming: Array<IStaticUpcoming> = []): { buffers: StaticDrawBuffers; draws: StaticDraws } {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.SLOTS]: 2, [EStaticPool.PLACES]: 2 });
  const draws: StaticDraws = new StaticDraws(buffers, new Scene(), () => upcoming);

  draws.isEnabled = true;

  return { buffers, draws };
}

/** An object still to come, drawing `sections` sections statically in `places` places. */
function toUpcoming(sections: number, places: number): IStaticUpcoming {
  const geometry: SceneGeometry = new SceneGeometry({ groups: [], position: new Float32Array(9) });

  return { geometry, places, sections };
}

describe("StaticDraws", () => {
  it("grows the slots once for what the queue brings, where every one is taken", () => {
    const { buffers, draws } = createDraws([toUpcoming(10, 0)]);

    draws.allocate();
    draws.allocate();

    const layout: number = buffers.layout;

    expect(draws.allocate()).toBe(2);
    // The taken two, the one asking and the ten to come, with a quarter more.
    expect(buffers.capacity(EStaticPool.SLOTS)).toBe(17);
    expect(buffers.layout).toBe(layout + 1);
    expect(draws.report.slots).toEqual({ capacity: 17, used: 3 });
  });

  it("grows the places for a run and for what the queue brings", () => {
    const { buffers, draws } = createDraws([toUpcoming(1, 3)]);

    expect(draws.allocatePlaces(5)).toBe(0);
    expect(buffers.capacity(EStaticPool.PLACES)).toBe(10);
    expect(draws.report.places).toEqual({ capacity: 10, used: 5 });
  });

  it("draws plainly where the device's limit stops a pool growing, and says how often", () => {
    const { buffers, draws } = createDraws();

    // Two slots of 64 bytes each, which the buffers hold already.
    buffers.storageLimit = 128;
    draws.allocate();
    draws.allocate();

    expect(draws.allocate()).toBeNull();
    expect(draws.allocatePlaces(3)).toBeNull();
    expect(draws.report.fallbacks).toBe(2);
    expect(buffers.capacity(EStaticPool.SLOTS)).toBe(2);
  });

  it("hands out nothing, and counts no fallback, while static draws are off", () => {
    const { draws } = createDraws();

    draws.isEnabled = false;

    expect(draws.allocate()).toBeNull();
    expect(draws.allocatePlaces(1)).toBeNull();
    expect(draws.report.fallbacks).toBe(0);
  });
});
