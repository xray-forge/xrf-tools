import { dot, float, Fn, If, instanceIndex, max, min, pow, storage, uint, vec3 } from "three/tsl";
import { ComputeNode, Node, StorageBufferAttribute } from "three/webgpu";

import { loopNamed } from "#/shader/named-loop.tsl";
import {
  LIGHT_CLUSTER_CAPACITY,
  LIGHT_CLUSTERS,
  LIGHT_CLUSTERS_X,
  LIGHT_CLUSTERS_Y,
  LIGHT_CLUSTERS_Z,
  LightsUniforms,
} from "#/uniforms/lights-uniforms";

/** What the lights are binned from and into. */
export interface ILightBinningBuffers {
  records: StorageBufferAttribute;
  counts: StorageBufferAttribute;
  items: StorageBufferAttribute;
}

/**
 * @param uniforms - What the lights are binned by.
 * @param slice - A slice of the view's depth, from zero.
 * @returns How far along the view the slice starts: exponentially from the near plane to the far one.
 */
export function toLightSliceDepth(uniforms: LightsUniforms, slice: Node<"float">): Node<"float"> {
  return uniforms.near.mul(pow(uniforms.far.div(uniforms.near), slice.div(LIGHT_CLUSTERS_Z)));
}

/** A device coordinate across at a depth, in view space: `(ndc + offset) * depth / scale`. */
function toViewX(projection: Node<"vec4">, ndc: Node<"float">, depth: Node<"float">): Node<"float"> {
  return ndc.add(projection.z).mul(depth).div(projection.x);
}

/** A device coordinate up at a depth, in view space. */
function toViewY(projection: Node<"vec4">, ndc: Node<"float">, depth: Node<"float">): Node<"float"> {
  return ndc.add(projection.w).mul(depth).div(projection.y);
}

/**
 * Bins the lights standing in view into the clusters of the view, a thread a cluster: each keeps the lights whose
 * sphere reaches its box, up to its capacity.
 *
 * @param buffers - The lights, and the clusters' counts and lists.
 * @param uniforms - What the view is cut by.
 * @param vectors - Vectors of four floats a light record takes.
 * @param capacity - Lights the records hold.
 * @returns The compute pass.
 */
export function createLightBinning(
  buffers: ILightBinningBuffers,
  uniforms: LightsUniforms,
  vectors: number,
  capacity: number
): ComputeNode {
  const records = storage(buffers.records, "vec4", capacity * vectors).toReadOnly();
  const counts = storage(buffers.counts, "uint", LIGHT_CLUSTERS);
  const items = storage(buffers.items, "uint", LIGHT_CLUSTERS * LIGHT_CLUSTER_CAPACITY);

  return Fn(() => {
    const cluster = instanceIndex;
    const x = float(cluster.mod(LIGHT_CLUSTERS_X));
    const y = float(cluster.div(LIGHT_CLUSTERS_X).mod(LIGHT_CLUSTERS_Y));
    const z = float(cluster.div(LIGHT_CLUSTERS_X * LIGHT_CLUSTERS_Y));
    const near = toLightSliceDepth(uniforms, z);
    const far = toLightSliceDepth(uniforms, z.add(1));
    // The tile's edges in device coordinates, `y` up from the bottom as clip space runs, the first row the top one.
    const left = x.mul(2 / LIGHT_CLUSTERS_X).sub(1);
    const right = left.add(2 / LIGHT_CLUSTERS_X);
    const top = float(1).sub(y.mul(2 / LIGHT_CLUSTERS_Y));
    const bottom = top.sub(2 / LIGHT_CLUSTERS_Y);
    const { projection } = uniforms;
    const least = vec3(
      min(
        min(toViewX(projection, left, near), toViewX(projection, left, far)),
        min(toViewX(projection, right, near), toViewX(projection, right, far))
      ),
      min(
        min(toViewY(projection, bottom, near), toViewY(projection, bottom, far)),
        min(toViewY(projection, top, near), toViewY(projection, top, far))
      ),
      far.negate()
    ).toVar();
    const most = vec3(
      max(
        max(toViewX(projection, left, near), toViewX(projection, left, far)),
        max(toViewX(projection, right, near), toViewX(projection, right, far))
      ),
      max(
        max(toViewY(projection, bottom, near), toViewY(projection, bottom, far)),
        max(toViewY(projection, top, near), toViewY(projection, top, far))
      ),
      near.negate()
    ).toVar();
    const kept = uint(0).toVar();

    loopNamed({ end: uniforms.count, name: "light", start: uint(0), type: "uint" }, (light) => {
      const sphere = records.element(light.mul(vectors).add(vectors - 1));
      const nearest = sphere.xyz.clamp(least, most);
      const offset = sphere.xyz.sub(nearest);

      If(dot(offset, offset).lessThanEqual(sphere.w.mul(sphere.w)).and(kept.lessThan(LIGHT_CLUSTER_CAPACITY)), () => {
        items.element(cluster.mul(LIGHT_CLUSTER_CAPACITY).add(kept)).assign(light);
        kept.addAssign(1);
      });
    });

    counts.element(cluster).assign(kept);
  })().compute(LIGHT_CLUSTERS);
}
