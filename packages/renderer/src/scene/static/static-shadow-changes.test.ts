import { describe, expect, it } from "@jest/globals";
import { Box3, Sphere, Vector3, Vector4 } from "three/webgpu";

import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";

const BOX: Box3 = new Box3(new Vector3(0, 0, 0), new Vector3(1, 1, 1));
const LIGHT: Sphere = new Sphere(new Vector3(), 3);
const PLANES: ReadonlyArray<Vector4> = [
  new Vector4(1, 0, 0, 2),
  new Vector4(-1, 0, 0, 2),
  new Vector4(0, 1, 0, 2),
  new Vector4(0, -1, 0, 2),
  new Vector4(0, 0, 1, 2),
  new Vector4(0, 0, -1, 2),
];
const OTHER: Box3 = new Box3(new Vector3(5, 0, 0), new Vector3(6, 1, 1));

describe("StaticShadowChanges", () => {
  it("logs where a casting slot came, went or cut out anew, and nothing for a slot that casts nothing", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, BOX, true, false);
    changes.put(2, OTHER, false, false);

    const seen: number = changes.version;

    changes.touch(2);
    changes.touch(1);
    changes.withdraw(1);
    changes.put(3, null, true, false);

    expect(changes.since(0)).toEqual([BOX, BOX, BOX, null]);
    expect(changes.since(seen)).toEqual([BOX, BOX, null]);
    expect(changes.since(changes.version)).toEqual([]);
  });

  it("moves a slot put again from its old box to its new one", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, BOX, true, false);
    changes.put(1, OTHER, true, false);

    expect(changes.since(1)).toEqual([BOX, OTHER]);
  });

  it("answers null for a version older than the log keeps, which stands for a change anywhere", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    for (let slot: number = 0; slot < 5000; slot += 1) {
      changes.put(slot, BOX, true, false);
    }

    expect(changes.since(0)).toBeNull();
    expect(changes.since(changes.version - 1)).toEqual([BOX]);
  });

  it("tracks animated intersections as casting slots come and go", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, BOX, true, true);
    changes.put(2, OTHER, true, false);

    const version: number = changes.swayingVersion;

    expect(changes.hasSwaying(LIGHT, PLANES)).toBe(true);

    changes.withdraw(1);

    expect(changes.hasSwaying(LIGHT, PLANES)).toBe(false);
    expect(changes.swayingVersion).toBeGreaterThan(version);
  });
});

describe("animated caster queries", () => {
  it("conservatively includes an unknown bound and forgets it when its slot no longer casts", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, null, true, true);
    expect(changes.hasSwaying(LIGHT, PLANES)).toBe(true);
    changes.put(1, null, false, true);
    expect(changes.hasSwaying(LIGHT, PLANES)).toBe(false);
  });

  it("requires the same active instance to intersect both the light and its face", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const bounds: Box3 = new Box3(new Vector3(-10, -10, -10), new Vector3(10, 10, 10));
    // One inactive instance in the face, one outside its planes, one outside the light's reach.
    const spheres: Float32Array = new Float32Array([0, 0, 0, -1, 0, 2.5, 0, 0.1, 0, 0, 5, 0.1]);
    const planes: ReadonlyArray<Vector4> = [...PLANES.slice(0, 4), new Vector4(0, 0, 1, 10), new Vector4(0, 0, -1, 10)];

    changes.put(1, bounds, true, true, spheres);
    expect(changes.hasSwaying(LIGHT, planes)).toBe(false);
    spheres.set([0, 0, 0, 1]);
    changes.put(1, bounds, true, true, spheres);
    expect(changes.hasSwaying(LIGHT, PLANES)).toBe(true);
    changes.put(1, bounds, true, true, new Float32Array());
    expect(changes.hasSwaying(LIGHT, PLANES)).toBe(false);
  });
});
