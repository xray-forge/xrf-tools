import { Nullable } from "@xrf/types";

import { IRendererClusters, RENDERER_CLUSTER_TRIANGLES } from "#/contract/scene/renderer-clusters";
import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererProgressive } from "#/contract/scene/renderer-progressive";
import { ISceneSection } from "#/scene/geometry/scene-section";

/**
 * Cuts a geometry that came without clusters as the packer cuts one (`VisualClusterTable`): every range its sections
 * draw, and every band of a progressive one, into runs of up to `RENDERER_CLUSTER_TRIANGLES` consecutive triangles,
 * each bounded by the smaller of the sphere about its box's centre and Ritter's. What a consumer builds itself, a
 * model posed out of its bind or an impostor's quad, is small; a level's geometry comes cut by the packer.
 *
 * @param geometry - The geometry, as it crossed.
 * @param sections - Its sections.
 * @returns Its clusters.
 */
export function cutSceneClusters(
  geometry: IRendererGeometry,
  sections: ReadonlyArray<ISceneSection>
): IRendererClusters {
  const ranges: Array<number> = [];
  const spheres: Array<number> = [];
  const cut: Set<string> = new Set();
  const runs: Array<readonly [number, number]> = sections.flatMap((section: ISceneSection) => [
    [section.start, section.count] as const,
    ...(section.progressive?.bands.map(
      (band: IRendererProgressive["bands"][number]) => [band.start, band.count] as const
    ) ?? []),
  ]);

  for (const [start, count] of runs) {
    const key: string = `${start}:${count}`;

    if (cut.has(key)) {
      continue;
    }

    cut.add(key);

    for (let at = start; at < start + count; at += RENDERER_CLUSTER_TRIANGLES * 3) {
      const length: number = Math.min(RENDERER_CLUSTER_TRIANGLES * 3, start + count - at);

      ranges.push(at, Math.floor(length / 3), 0xffffffff, 0);
      spheres.push(...toSphere(geometry, at, length));
    }
  }

  return { ranges: new Uint32Array(ranges), spheres: new Float32Array(spheres) };
}

/** The vertex an index of a geometry names: the index's own number for one drawn without indices. */
function toVertex(geometry: IRendererGeometry, at: number): number {
  return geometry.index ? geometry.index[at] : at;
}

function toDistance(positions: Float32Array, vertex: number, centre: ReadonlyArray<number>): number {
  return Math.hypot(
    positions[vertex * 3] - centre[0],
    positions[vertex * 3 + 1] - centre[1],
    positions[vertex * 3 + 2] - centre[2]
  );
}

/** How far the farthest vertex of a run stands from a centre. */
function toReach(geometry: IRendererGeometry, start: number, length: number, centre: ReadonlyArray<number>): number {
  let reach: number = 0;

  for (let at = start; at < start + length; at += 1) {
    reach = Math.max(reach, toDistance(geometry.position, toVertex(geometry, at), centre));
  }

  return reach;
}

/** The vertex of a run farthest from a point. */
function toFarthest(geometry: IRendererGeometry, start: number, length: number, from: ReadonlyArray<number>): number {
  let farthest: Nullable<number> = null;
  let distance: number = -1;

  for (let at = start; at < start + length; at += 1) {
    const vertex: number = toVertex(geometry, at);
    const it: number = toDistance(geometry.position, vertex, from);

    if (it > distance) {
      distance = it;
      farthest = vertex;
    }
  }

  return farthest ?? toVertex(geometry, start);
}

/** The smaller of the sphere about a run's box's centre and Ritter's, as four floats; nought for an empty run. */
function toSphere(geometry: IRendererGeometry, start: number, length: number): [number, number, number, number] {
  if (!length) {
    return [0, 0, 0, 0];
  }

  const positions: Float32Array = geometry.position;
  const low: Array<number> = [Infinity, Infinity, Infinity];
  const high: Array<number> = [-Infinity, -Infinity, -Infinity];

  for (let at = start; at < start + length; at += 1) {
    const vertex: number = toVertex(geometry, at);

    for (let axis = 0; axis < 3; axis += 1) {
      low[axis] = Math.min(low[axis], positions[vertex * 3 + axis]);
      high[axis] = Math.max(high[axis], positions[vertex * 3 + axis]);
    }
  }

  const boxed: Array<number> = low.map((value: number, axis: number) => (value + high[axis]) / 2);
  const boxedRadius: number = toReach(geometry, start, length, boxed);
  const first: Array<number> = Array.from(
    positions.subarray(toVertex(geometry, start) * 3, toVertex(geometry, start) * 3 + 3)
  );
  const far: number = toFarthest(geometry, start, length, first);
  const farPoint: Array<number> = Array.from(positions.subarray(far * 3, far * 3 + 3));
  const other: number = toFarthest(geometry, start, length, farPoint);
  const centre: Array<number> = farPoint.map(
    (value: number, axis: number) => (value + positions[other * 3 + axis]) / 2
  );
  let radius: number = toDistance(positions, other, centre);

  for (let at = start; at < start + length; at += 1) {
    const vertex: number = toVertex(geometry, at);
    const distance: number = toDistance(positions, vertex, centre);

    if (distance > radius) {
      const grown: number = (radius + distance) / 2;
      const shift: number = (grown - radius) / distance;

      for (let axis = 0; axis < 3; axis += 1) {
        centre[axis] += (positions[vertex * 3 + axis] - centre[axis]) * shift;
      }

      radius = grown;
    }
  }

  const ritterRadius: number = Math.max(radius, toReach(geometry, start, length, centre));

  return ritterRadius < boxedRadius
    ? [centre[0], centre[1], centre[2], ritterRadius]
    : [boxed[0], boxed[1], boxed[2], boxedRadius];
}
