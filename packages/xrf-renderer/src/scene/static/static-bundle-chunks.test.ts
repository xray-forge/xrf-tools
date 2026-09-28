import { describe, expect, it } from "@jest/globals";
import { storage } from "three/tsl";
import { BufferAttribute, BufferGeometry, Scene, StorageBufferAttribute } from "three/webgpu";

import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch } from "#/scene/static/static-batch";
import { StaticBundleChunks } from "#/scene/static/static-bundle-chunks";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticListSpace } from "#/uniforms/static-list-space";
import { EStaticView } from "#/uniforms/static-view";
import { StorageRetirement } from "#/uniforms/storage-retirement";

function createBatches(count: number): Array<StaticBatch> {
  const buffer: BufferGeometry = new BufferGeometry();
  const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));

  const arena: StaticArena = new StaticArena(
    buffer,
    storage(new StorageBufferAttribute(new Uint32Array(2), 2), "uvec2", 1).toReadOnly(),
    storage(new StorageBufferAttribute(new Uint32Array(4), 4), "uvec4", 1).toReadOnly(),
    new StorageRetirement()
  );

  return Array.from(
    { length: count },
    (_, id: number) =>
      new StaticBatch(arena, id, EStaticListSpace.SURFACES, [
        () => buffers.viewArgs[EStaticView.EARLY],
        () => buffers.viewArgs[EStaticView.LATE],
      ])
  );
}

describe("StaticBundleChunks", () => {
  it("records batches a chunk of them to a pair of bundles, one in each phase's scene", () => {
    const scene: Scene = new Scene();
    const late: Scene = new Scene();
    const chunks: StaticBundleChunks = new StaticBundleChunks([scene, late]);

    createBatches(40).forEach((batch: StaticBatch) => chunks.attach(batch, batch.meshes));

    expect(scene.children).toHaveLength(2);
    expect(late.children).toHaveLength(2);
    expect(scene.children[0].children).toHaveLength(32);
  });

  it("records a chunk again when a batch leaves it, and takes an emptied chunk out of the scenes", () => {
    const scene: Scene = new Scene();
    const late: Scene = new Scene();
    const chunks: StaticBundleChunks = new StaticBundleChunks([scene, late]);
    const [batch] = createBatches(1);

    chunks.attach(batch, batch.meshes);

    const bundle = scene.children[0];
    const version: number = (bundle as unknown as { version: number }).version;

    chunks.detach(batch);

    expect((bundle as unknown as { version: number }).version).toBeGreaterThan(version);
    expect(scene.children).toEqual([]);
    expect(late.children).toEqual([]);
  });
});
