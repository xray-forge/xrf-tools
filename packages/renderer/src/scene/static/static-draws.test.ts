import { describe, expect, it } from "@jest/globals";
import { MeshBasicNodeMaterial, Scene, Vector3 } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { ILightShadowRequest, LightShadowPlanner } from "#/scene/lights/light-shadow-planner";
import { StaticDraws } from "#/scene/static/static-draws";
import { IStaticRange } from "#/scene/static/static-range";
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

  return { clusters: 0, geometry, places, sections };
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

    // Two slots of 32 bytes each, which the buffers hold already.
    buffers.storageLimit = 64;
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

// Exercises the real registration path: listed draw -> batches -> caster changes -> shadow planner.
describe("listed tree shadow invalidation", () => {
  it("keeps a lamp between distant instances cached, but follows a tree moved into its face", () => {
    const { draws } = createDraws();
    const geometry: SceneGeometry = new SceneGeometry({
      groups: [],
      packed: { normal: new Uint8Array(12), uv: new Int16Array(12) },
      position: new Float32Array(9),
    });
    const range: IStaticRange = draws.acquire(geometry) as IStaticRange;
    const slot: number = draws.allocate() as number;
    const places: number = draws.allocatePlaces(2) as number;
    const surface: ISurfaceMaterial = {
      dispose: () => {},
      isImpostor: false,
      keys: [],
      material: new MeshBasicNodeMaterial(),
      pass: ERendererPass.DEFERRED,
      shadow: new MeshBasicNodeMaterial(),
      shadowKeys: [],
    };
    const shadows: LightShadowPlanner = new LightShadowPlanner(draws.shadowChanges);
    const light: ILightShadowRequest = {
      cone: Math.PI / 2,
      direction: new Vector3(0, -1, 0),
      distance: 0,
      duel: 1,
      intensity: 1,
      isSpot: true,
      near: 0.1,
      position: new Vector3(0, 3, 0),
      range: 8,
      up: new Vector3(0, 0, 1),
    };
    const spheres: Float32Array = new Float32Array([-20, 0, 0, 1, 20, 0, 0, 1]);

    function put(count: number = 3): void {
      const run = count ? geometry.clusters.toRun(0, count) : { count: 0, start: 0 };

      expect(
        draws.drawListed(slot, surface, range, geometry.clusters, run ?? { count: 0, start: 0 }, places, spheres)
      ).toBe(true);
    }

    function frame(isWindy: boolean = true): number {
      shadows.begin(isWindy);
      shadows.request(0, light);
      shadows.finish(8);

      const count: number = shadows.queue.length;

      shadows.markDrawn();

      return count;
    }

    try {
      expect(range.arena.isSwaying).toBe(true);
      put();
      expect(frame()).toBe(1);
      expect(frame()).toBe(0);

      // A caller reusing its sphere array must not silently alter registered caster bounds.
      spheres[0] = 0;
      expect(frame()).toBe(0);
      put();
      expect(frame()).toBe(1);
      expect(frame()).toBe(1);
      expect(frame(false)).toBe(0);
      expect(frame()).toBe(1);

      spheres[0] = -20;
      put();
      expect(frame()).toBe(1);
      expect(frame()).toBe(0);

      // A zero-index draw contributes no animated instances, even while its placement is under the lamp.
      spheres[0] = 0;
      put(0);
      expect(frame()).toBe(1);
      expect(frame()).toBe(0);
      put();
      expect(frame()).toBe(1);
      expect(frame()).toBe(1);

      draws.free(slot);
      expect(frame()).toBe(1);
      expect(frame()).toBe(0);
      expect(draws.allocate()).toBe(slot);
      spheres[0] = -20;
      put();
      expect(frame()).toBe(1);
      expect(frame()).toBe(0);
    } finally {
      draws.dispose();
      surface.material.dispose();
      surface.shadow?.dispose();
    }
  });
});
