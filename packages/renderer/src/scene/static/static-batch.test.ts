import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry, BundleGroup, Mesh } from "three/webgpu";

import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch } from "#/scene/static/static-batch";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

function createBuffer(vertices: number): BufferGeometry {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(vertices * 3), 3));

  return buffer;
}

function createArena(): StaticArena {
  const arena: StaticArena = new StaticArena(createBuffer(3), 64);

  arena.place(createBuffer(3), () => ({ indices: 0, vertices: 0 }));

  return arena;
}

function toMesh(batch: StaticBatch): Mesh {
  return batch.meshes[0];
}

describe("StaticBatch", () => {
  it("issues a draw a slot from each phase's arguments, the last taking the place of one removed", () => {
    const batch: StaticBatch = new StaticBatch(
      createArena(),
      EStaticDrawKind.SINGLE,
      new StaticDrawPool(new StaticDrawBuffers())
    );

    [4, 7, 9].forEach((slot: number) => batch.add(slot));
    batch.remove(4);

    expect(toMesh(batch).geometry.indirectOffset).toEqual([9 * 20, 7 * 20]);
    expect(batch.meshes[1].geometry.indirectOffset).toEqual([9 * 20, 7 * 20]);
    expect(batch.meshes[1].geometry.indirect).not.toBe(toMesh(batch).geometry.indirect);

    batch.remove(9);
    batch.remove(7);

    expect(batch.isEmpty).toBe(true);
  });

  it("draws the arena's new buffers once it grew, over a new mesh, recorded again", () => {
    const arena: StaticArena = createArena();
    const pool: StaticDrawPool = new StaticDrawPool(new StaticDrawBuffers());
    const batch: StaticBatch = new StaticBatch(arena, EStaticDrawKind.SINGLE, pool);
    const bundle: BundleGroup = new BundleGroup();

    bundle.add(toMesh(batch));

    const before: Mesh = toMesh(batch);
    const version: number = bundle.version;

    batch.add(1);
    arena.place(createBuffer(1 << 17), () => ({ indices: 0, vertices: 0 }));
    batch.refresh();

    expect(toMesh(batch)).not.toBe(before);
    expect(toMesh(batch).geometry.getAttribute("position").count).toBeGreaterThan(1 << 16);
    expect(toMesh(batch).geometry.indirectOffset).toEqual([20]);
    expect(toMesh(batch).parent).toBe(bundle);
    expect(bundle.version).toBeGreaterThan(version);
  });
});
