import { describe, expect, it } from "@jest/globals";
import { storage } from "three/tsl";
import { BufferAttribute, BufferGeometry, Matrix4, StorageBufferAttribute } from "three/webgpu";

import { SceneClusters } from "#/scene/geometry/scene-clusters";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticClusters } from "#/scene/static/static-clusters";
import { IStaticRange } from "#/scene/static/static-range";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";

const CLUSTERS: SceneClusters = new SceneClusters({
  ranges: new Uint32Array([0, 2, 0, 0, 6, 1, 0, 0]),
  spheres: new Float32Array([1, 0, 0, 1, 0, 2, 0, 0.5]),
});

function createRange(): IStaticRange {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));

  const arena: StaticArena = new StaticArena(
    buffer,
    storage(new StorageBufferAttribute(new Uint32Array(2), 2), "uvec2", 1).toReadOnly(),
    storage(new StorageBufferAttribute(new Uint32Array(4), 4), "uvec4", 1).toReadOnly(),
    new StorageRetirement()
  );

  return { arena, indexCount: 9, indexStart: 30, vertexCount: 3, vertexStart: 7 };
}

function toCluster(buffers: StaticDrawBuffers, cluster: number): Array<Array<number>> {
  return [
    Array.from((buffers.clusterRanges.array as Uint32Array).subarray(cluster * 4, cluster * 4 + 4)),
    Array.from((buffers.clusterSpheres.array as Float32Array).subarray(cluster * 4, cluster * 4 + 4)),
  ];
}

describe("StaticClusters", () => {
  it("copies a single draw's clusters in onto its arena, their spheres where its matrix stands them", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const clusters: StaticClusters = new StaticClusters(buffers);
    const start: number = clusters.allocate(2) as number;

    clusters.write(start, 5, CLUSTERS, { count: 2, start: 0 }, createRange(), new Matrix4().makeScale(2, 2, 2));

    // Its first index in the arena, its triangles, its base vertex and its slot; its sphere in renderer space.
    expect(toCluster(buffers, start)).toEqual([
      [30, 2, 7, 5],
      [2, 0, 0, 2],
    ]);
    expect(toCluster(buffers, start + 1)).toEqual([
      [36, 1, 7, 5],
      [0, 4, 0, 1],
    ]);
  });

  it("leaves an instanced draw's spheres in its mesh's own space, for each place to stand", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const clusters: StaticClusters = new StaticClusters(buffers);
    const start: number = clusters.allocate(1) as number;

    clusters.write(start, 5, CLUSTERS, { count: 1, start: 1 }, createRange(), null);

    expect(toCluster(buffers, start)).toEqual([
      [36, 1, 7, 5],
      [0, 2, 0, 0.5],
    ]);
  });

  // A cluster the cull still reaches past the last run draws nothing.
  it("leaves a freed run of no triangles, free for another, and uploads what changed as one span", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const clusters: StaticClusters = new StaticClusters(buffers);
    const start: number = clusters.allocate(2) as number;

    clusters.write(start, 5, CLUSTERS, { count: 2, start: 0 }, createRange(), null);

    const version: number = clusters.version;

    clusters.free(start, 2);
    clusters.flush();

    expect(toCluster(buffers, start)[0]).toEqual([30, 0, 7, 5]);
    expect(clusters.version).toBeGreaterThan(version);
    expect(clusters.used).toBe(0);
    expect(buffers.clusterRanges.updateRanges).toEqual([{ count: 8, start: 0 }]);
  });

  it("grows with what is written kept, reaching as far as its last run", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), { [EStaticPool.CLUSTERS]: 2 });
    const clusters: StaticClusters = new StaticClusters(buffers);

    clusters.write(clusters.allocate(2) as number, 5, CLUSTERS, { count: 2, start: 0 }, createRange(), null);

    expect(clusters.fits(1)).toBe(false);

    clusters.grow(4);

    expect(clusters.allocate(1)).toBe(2);
    expect([clusters.capacity, clusters.used, clusters.extent]).toEqual([4, 3, 3]);
    expect(toCluster(buffers, 1)[0]).toEqual([36, 1, 7, 5]);
  });
});
