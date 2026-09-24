import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry, Scene } from "three/webgpu";

import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch } from "#/scene/static/static-batch";
import { StaticBundleChunks } from "#/scene/static/static-bundle-chunks";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

function createBatches(count: number): Array<StaticBatch> {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));

  const arena: StaticArena = new StaticArena(buffer, 64);
  const pool: StaticDrawPool = new StaticDrawPool(new StaticDrawBuffers());

  arena.place(buffer, () => ({ indices: 0, vertices: 0 }));

  return Array.from({ length: count }, () => new StaticBatch(arena, EStaticDrawKind.SINGLE, pool));
}

describe("StaticBundleChunks", () => {
  it("records batches a chunk of them to a pair of bundles, one in each phase's scene", () => {
    const scene: Scene = new Scene();
    const late: Scene = new Scene();
    const chunks: StaticBundleChunks = new StaticBundleChunks(scene, late);

    createBatches(40).forEach((batch: StaticBatch) => chunks.attach(batch));

    expect(scene.children).toHaveLength(2);
    expect(late.children).toHaveLength(2);
    expect(scene.children[0].children).toHaveLength(32);
    expect(chunks.bundles).toBe(4);
  });

  it("records a chunk again when a batch leaves it, and takes an emptied chunk out of the scenes", () => {
    const scene: Scene = new Scene();
    const late: Scene = new Scene();
    const chunks: StaticBundleChunks = new StaticBundleChunks(scene, late);
    const [batch] = createBatches(1);

    chunks.attach(batch);

    const bundle = scene.children[0];
    const version: number = (bundle as unknown as { version: number }).version;

    chunks.detach(batch);

    expect((bundle as unknown as { version: number }).version).toBeGreaterThan(version);
    expect(scene.children).toEqual([]);
    expect(late.children).toEqual([]);
  });
});
