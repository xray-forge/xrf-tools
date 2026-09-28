import { describe, expect, it } from "@jest/globals";

import { SceneClusters } from "#/scene/geometry/scene-clusters";

/** Clusters from first index and triangles, of no drawable and a unit sphere. */
function createClusters(runs: ReadonlyArray<readonly [number, number]>): SceneClusters {
  return new SceneClusters({
    ranges: new Uint32Array(runs.flatMap(([first, triangles]) => [first, triangles, 0, 0])),
    spheres: new Float32Array(runs.length * 4).fill(1),
  });
}

describe("SceneClusters", () => {
  // A section of two drawables, then a progressive mesh's two bands, which overlap in the indices.
  it("finds the run a range is cut into", () => {
    const clusters: SceneClusters = createClusters([
      [0, 128],
      [384, 20],
      [444, 10],
      [474, 5],
      [474, 3],
    ]);

    expect(clusters.toRun(0, 474)).toEqual({ count: 3, start: 0 });
    expect(clusters.toRun(444, 30)).toEqual({ count: 1, start: 2 });
    expect(clusters.toRun(474, 15)).toEqual({ count: 1, start: 3 });
  });

  // Drawn statically only as a whole run: a range narrowed to part of one would draw what it left out.
  it("finds none for a range no run is exactly", () => {
    const clusters: SceneClusters = createClusters([
      [0, 128],
      [384, 20],
    ]);

    expect(clusters.toRun(0, 384)).toEqual({ count: 1, start: 0 });
    expect(clusters.toRun(0, 390)).toBeNull();
    expect(clusters.toRun(3, 381)).toBeNull();
    expect(clusters.toRun(0, 0)).toBeNull();
  });
});
