import { describe, expect, it } from "@jest/globals";
import { Box3, Sphere, Vector3, Vector4 } from "three/webgpu";

import { EShadowCasterMotion, IShadowChange, StaticShadowChanges } from "#/scene/static/static-shadow-changes";

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

/** Where each change since a version was. */
function toBoxes(changes: StaticShadowChanges, since: number): Array<Box3 | null> {
  return changes.since(since).changes.map((change: IShadowChange) => change.box);
}

describe("StaticShadowChanges", () => {
  it("logs where a caster came, went or cut out anew, and nothing for one that casts nothing", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, BOX, true);
    changes.put(2, OTHER, false);

    const seen: number = changes.version;

    changes.touch(2);
    changes.touch(1);
    changes.withdraw(1);
    changes.put(3, null, true);

    expect(toBoxes(changes, 0)).toEqual([BOX, BOX, BOX, null]);
    expect(toBoxes(changes, seen)).toEqual([BOX, BOX, null]);
    expect(changes.since(seen).isEverywhere).toBe(true);
    expect(changes.since(changes.version)).toEqual({ changes: [], isEverywhere: false });
  });

  it("moves a caster put again from its old box to its new one", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, BOX, true);
    changes.put(1, OTHER, true);

    expect(toBoxes(changes, 1)).toEqual([BOX, OTHER]);
  });

  it("answers everywhere for a version older than the log keeps", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    for (let slot: number = 0; slot < 5000; slot += 1) {
      changes.put(slot, BOX, true);
    }

    expect(changes.since(0)).toEqual({ changes: [], isEverywhere: true });
    expect(changes.since(changes.version - 1)).toEqual({
      changes: [{ box: BOX, isAnimated: false }],
      isEverywhere: false,
    });
  });

  it("tells a caster that sways or moves by its own changes, and the fastest standing in a face", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const part: object = {};

    changes.put(1, BOX, true, EShadowCasterMotion.SWAYING);
    changes.put(2, OTHER, true);

    expect(changes.since(0).changes.map((change: IShadowChange) => change.isAnimated)).toEqual([true, false]);
    expect(changes.getMotion(PLANES, LIGHT)).toBe(EShadowCasterMotion.SWAYING);

    changes.put(part, BOX, true, EShadowCasterMotion.MOVING);

    expect(changes.getMotion(PLANES, LIGHT)).toBe(EShadowCasterMotion.MOVING);

    changes.withdraw(part);
    changes.withdraw(1);

    expect(changes.getMotion(PLANES, LIGHT)).toBe(EShadowCasterMotion.STILL);
    expect(changes.since(changes.version - 1).changes[0].isAnimated).toBe(true);
  });

  it("finds what moves in a cascade's box by its planes alone", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, OTHER, true, EShadowCasterMotion.SWAYING);

    expect(changes.getMotion(PLANES)).toBe(EShadowCasterMotion.STILL);
    expect(changes.getMotion([...PLANES.slice(0, 1), new Vector4(-1, 0, 0, 10), ...PLANES.slice(2)])).toBe(
      EShadowCasterMotion.SWAYING
    );
  });
});

describe("animated caster queries", () => {
  it("takes an unknown bound to stand everywhere, and forgets it once its caster casts no more", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, null, true, EShadowCasterMotion.SWAYING);
    expect(changes.getMotion(PLANES, LIGHT)).toBe(EShadowCasterMotion.SWAYING);
    changes.put(1, null, false, EShadowCasterMotion.SWAYING);
    expect(changes.getMotion(PLANES, LIGHT)).toBe(EShadowCasterMotion.STILL);
  });

  it("asks the same place of a listed draw to stand in both the light and its face", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const bounds: Box3 = new Box3(new Vector3(-10, -10, -10), new Vector3(10, 10, 10));
    // One place gone, one outside the face's planes, one outside the light's reach.
    const spheres: Float32Array = new Float32Array([0, 0, 0, -1, 0, 2.5, 0, 0.1, 0, 0, 5, 0.1]);
    const planes: ReadonlyArray<Vector4> = [...PLANES.slice(0, 4), new Vector4(0, 0, 1, 10), new Vector4(0, 0, -1, 10)];
    const swaying: EShadowCasterMotion = EShadowCasterMotion.SWAYING;

    changes.put(1, bounds, true, swaying, spheres);
    expect(changes.getMotion(planes, LIGHT)).toBe(EShadowCasterMotion.STILL);
    spheres.set([0, 0, 0, 1]);
    changes.put(1, bounds, true, swaying, spheres);
    expect(changes.getMotion(PLANES, LIGHT)).toBe(swaying);
    changes.put(1, bounds, true, swaying, new Float32Array());
    expect(changes.getMotion(PLANES, LIGHT)).toBe(EShadowCasterMotion.STILL);
  });
});
