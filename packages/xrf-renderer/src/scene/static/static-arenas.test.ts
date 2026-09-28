import { describe, expect, it, jest } from "@jest/globals";
import { BufferAttribute } from "three/webgpu";

import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticArenas } from "#/scene/static/static-arenas";
import { IStaticRange } from "#/scene/static/static-range";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";

function createGeometry(vertices: number = 3, hasUv: boolean = false): SceneGeometry {
  return new SceneGeometry({
    groups: [],
    position: new Float32Array(vertices * 3),
    ...(hasUv ? { uv: new Float32Array(vertices * 2) } : {}),
  });
}

function createArenas(upcoming: Array<SceneGeometry> = []): {
  arenas: StaticArenas;
  retired: Array<BufferAttribute>;
} {
  const retirement: StorageRetirement = new StorageRetirement();
  const retired: Array<BufferAttribute> = [];

  jest
    .spyOn(retirement, "retire")
    .mockImplementation((attributes: Iterable<BufferAttribute>) => void retired.push(...attributes));

  const buffers: StaticDrawBuffers = new StaticDrawBuffers(retirement, {
    [EStaticPool.SHADOW_LIST]: 1,
    [EStaticPool.SURFACE_LIST]: 1,
  });

  return { arenas: new StaticArenas(buffers, () => upcoming), retired };
}

describe("StaticArenas", () => {
  it("keeps one arena a layout, made the first time a geometry of it asks", () => {
    const { arenas } = createArenas();
    const arena: StaticArena = arenas.toArena(createGeometry()) as StaticArena;

    expect(arenas.toArena(createGeometry(6))).toBe(arena);
    expect(arenas.toArena(createGeometry(3, true))).not.toBe(arena);
  });

  it("places a geometry once however many objects draw it, and frees its room once none does", () => {
    const { arenas } = createArenas();
    const shared: SceneGeometry = createGeometry();
    const range: IStaticRange = arenas.acquire(shared) as IStaticRange;

    expect(arenas.acquire(shared)).toBe(range);

    arenas.release(shared);

    // Still drawn by one: its room is kept.
    expect((arenas.acquire(createGeometry()) as IStaticRange).vertexStart).toBe(3);

    arenas.release(shared);

    expect((arenas.acquire(createGeometry()) as IStaticRange).vertexStart).toBe(0);
  });

  // Every growth copies the whole arena again: one for all that is coming is one hitch, not one a geometry.
  it("grows an arena once for the geometries of its layout still coming, not those of another", () => {
    const coming: SceneGeometry = createGeometry(1 << 17);
    const { arenas } = createArenas([coming, createGeometry(1 << 18, true)]);
    const first: SceneGeometry = createGeometry();

    arenas.acquire(first);

    const arena: StaticArena = arenas.toArena(first) as StaticArena;
    const generation: number = arenas.generation;

    arenas.acquire(coming);

    expect(arenas.generation).toBe(generation);
    expect(arena.generation).toBe(1);
  });

  it("gives every arena's buffers up as it goes, to be freed once nothing binds them", () => {
    const { arenas, retired } = createArenas();
    const arena: StaticArena = arenas.toArena(createGeometry()) as StaticArena;

    arenas.dispose();

    expect(retired).toEqual([arena.wordNode.value, arena.indexNode.value]);
  });
});
