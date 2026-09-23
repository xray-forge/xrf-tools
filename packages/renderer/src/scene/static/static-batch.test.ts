import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry, Mesh } from "three/webgpu";

import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch } from "#/scene/static/static-batch";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

function createBuffer(vertices: number): BufferGeometry {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(vertices * 3), 3));

  return buffer;
}

function createArena(): StaticArena {
  const arena: StaticArena = new StaticArena(createBuffer(3));

  arena.place(createBuffer(3));

  return arena;
}

function toMesh(batch: StaticBatch): Mesh {
  return batch.bundle.children[0] as Mesh;
}

describe("StaticBatch", () => {
  it("issues a draw a slot, the last taking the place of one removed", () => {
    const batch: StaticBatch = new StaticBatch(createArena(), new StaticDrawPool(new StaticDrawBuffers()));

    [4, 7, 9].forEach((slot: number) => batch.add(slot));
    batch.remove(4);

    expect(toMesh(batch).geometry.indirectOffset).toEqual([9 * 20, 7 * 20]);

    batch.remove(9);
    batch.remove(7);

    expect(batch.isEmpty).toBe(true);
  });

  it("draws the arena's new buffers once it grew, over a new mesh, recorded again", () => {
    const arena: StaticArena = createArena();
    const pool: StaticDrawPool = new StaticDrawPool(new StaticDrawBuffers());
    const batch: StaticBatch = new StaticBatch(arena, pool);
    const before: Mesh = toMesh(batch);
    const version: number = batch.bundle.version;

    batch.add(1);
    arena.place(createBuffer(1 << 17));
    batch.refresh(pool);

    expect(toMesh(batch)).not.toBe(before);
    expect(toMesh(batch).geometry.getAttribute("position").count).toBeGreaterThan(1 << 16);
    expect(toMesh(batch).geometry.indirectOffset).toEqual([20]);
    expect(batch.bundle.version).toBeGreaterThan(version);
  });
});
