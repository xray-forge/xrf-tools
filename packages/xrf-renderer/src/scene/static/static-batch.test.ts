import { describe, expect, it, jest } from "@jest/globals";
import { storage } from "three/tsl";
import {
  BufferAttribute,
  BufferGeometry,
  BundleGroup,
  LineSegments,
  Material,
  Mesh,
  StorageBufferAttribute,
} from "three/webgpu";

import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch } from "#/scene/static/static-batch";
import { STATIC_BATCH_ARGUMENT_BYTES, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticListSpace } from "#/uniforms/static-list-space";
import { EStaticPool } from "#/uniforms/static-pool";
import { EStaticView } from "#/uniforms/static-view";
import { StorageRetirement } from "#/uniforms/storage-retirement";

function createArena(): StaticArena {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));

  return new StaticArena(
    buffer,
    storage(new StorageBufferAttribute(new Uint32Array(2), 2), "uvec2", 1).toReadOnly(),
    storage(new StorageBufferAttribute(new Uint32Array(4), 4), "uvec4", 1).toReadOnly(),
    new StorageRetirement()
  );
}

function createBatch(buffers: StaticDrawBuffers, id: number = 3): StaticBatch {
  return new StaticBatch(createArena(), id, EStaticListSpace.SURFACES, [
    () => buffers.viewArgs[EStaticView.EARLY],
    () => buffers.viewArgs[EStaticView.LATE],
  ]);
}

function toMesh(batch: StaticBatch, phase: number = 0): Mesh {
  return batch.meshes[phase] as Mesh;
}

describe("StaticBatch", () => {
  // One command a view whatever it draws: its clusters are the instances of one indirect draw.
  it("issues one draw a phase from that phase's arguments, at its own offset", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const batch: StaticBatch = createBatch(buffers, 3);

    expect(toMesh(batch).geometry.indirect).toBe(buffers.viewArgs[EStaticView.EARLY]);
    expect(toMesh(batch, 1).geometry.indirect).toBe(buffers.viewArgs[EStaticView.LATE]);
    expect(toMesh(batch).geometry.indirectOffset).toBe(3 * STATIC_BATCH_ARGUMENT_BYTES);
    expect(toMesh(batch).geometry.index).toBeNull();
  });

  it("holds room for every entry its slots may list at once", () => {
    const batch: StaticBatch = createBatch(new StaticDrawBuffers(new StorageRetirement()));

    batch.put(4, 10);
    batch.put(7, 200);
    batch.put(4, 12);

    expect(batch.demand).toBe(212);

    batch.remove(7);
    batch.remove(4);

    expect(batch.demand).toBe(0);
    expect(batch.isEmpty).toBe(true);
  });

  // The same mesh, so a bundle holding it takes it out again as the batch goes.
  it("draws the arguments the batches' growth replaced from the same mesh, its bundle recorded again", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), { [EStaticPool.BATCHES]: 4 });
    const batch: StaticBatch = createBatch(buffers, 1);
    const bundle: BundleGroup = new BundleGroup();

    bundle.add(toMesh(batch));

    const before: Mesh = toMesh(batch);
    const version: number = bundle.version;

    buffers.grow(EStaticPool.BATCHES, 8);
    batch.refresh();

    expect(toMesh(batch)).toBe(before);
    expect(toMesh(batch).geometry.indirect).toBe(buffers.viewArgs[EStaticView.EARLY]);
    expect(toMesh(batch).geometry.indirectOffset).toBe(STATIC_BATCH_ARGUMENT_BYTES);
    expect(toMesh(batch).parent).toBe(bundle);
    expect(bundle.version).toBeGreaterThan(version);
  });

  it("draws its region's edges as lines while a wireframe draws, by the wireframe's arguments at its offset", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const batch: StaticBatch = createBatch(buffers, 2);
    const material: Material = new Material();

    batch.setWire([() => buffers.wireArgs[0], () => buffers.wireArgs[1]], material);

    const [first] = batch.wireMeshes;

    expect(first).toBeInstanceOf(LineSegments);
    expect(first.material).toBe(material);
    expect(first.geometry.indirect).toBe(buffers.wireArgs[0]);
    expect(first.geometry.indirectOffset).toBe(2 * STATIC_BATCH_ARGUMENT_BYTES);
  });

  // Three keeps an object's render objects, and with them its geometry, until the object itself is disposed.
  it("lets three forget every mesh it drew with as it goes", () => {
    const batch: StaticBatch = createBatch(new StaticDrawBuffers(new StorageRetirement()));
    const onDispose = jest.fn();

    batch.meshes.forEach((mesh) => mesh.addEventListener("dispose" as never, onDispose));
    batch.dispose();

    expect(onDispose).toHaveBeenCalledTimes(2);
  });
});
