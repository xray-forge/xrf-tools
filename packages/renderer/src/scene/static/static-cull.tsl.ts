import { atomicAdd, Fn, If, instanceIndex, storage, uint, uniformArray } from "three/tsl";
import { ComputeNode, Node, Vector4 } from "three/webgpu";

import { STATIC_DRAW_ARGUMENTS, STATIC_DRAW_CAPACITY, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** What a static cull reads besides the buffers: the view's planes, set before each dispatch. */
export interface IStaticCullShader {
  compute: ComputeNode;
  /** Six planes, normals pointing in, `w` the constant. */
  planes: ReadonlyArray<Vector4>;
}

/**
 * Frustum culling on the GPU: every static draw's instance count is one where its sphere reaches into the view, and
 * zero where it does not or where the slot draws nothing. The draws and indices it keeps are counted for the report.
 *
 * @param buffers - The static draw buffers.
 * @returns The compute pass, one invocation a slot, and the planes it reads.
 */
export function createStaticCullShader(buffers: StaticDrawBuffers): IStaticCullShader {
  const planes: Array<Vector4> = Array.from({ length: 6 }, () => new Vector4());
  const planeNodes = uniformArray(planes, "vec4");
  const args = storage(buffers.args, "uint", STATIC_DRAW_CAPACITY * STATIC_DRAW_ARGUMENTS);
  const spheres = storage(buffers.spheres, "vec4", STATIC_DRAW_CAPACITY).toReadOnly();
  const counts = storage(buffers.counts, "uint", 2).toAtomic();

  const compute = Fn(() => {
    const sphere = spheres.element(instanceIndex);
    const isSeen = uint(1).toVar();

    If(sphere.w.lessThan(0), () => {
      isSeen.assign(0);
    });

    for (let plane = 0; plane < planes.length; plane += 1) {
      const it = planeNodes.element(plane) as unknown as Node<"vec4">;

      If(it.xyz.dot(sphere.xyz).add(it.w).lessThan(sphere.w.negate()), () => {
        isSeen.assign(0);
      });
    }

    const at = instanceIndex.mul(STATIC_DRAW_ARGUMENTS);

    args.element(at.add(1)).assign(isSeen);

    If(isSeen.equal(1), () => {
      atomicAdd(counts.element(0), uint(1));
      atomicAdd(counts.element(1), args.element(at));
    });
  })().compute(STATIC_DRAW_CAPACITY);

  return { compute, planes };
}
