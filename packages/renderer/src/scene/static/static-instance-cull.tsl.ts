import { atomicAdd, Fn, If, instanceIndex, select, storage, uint } from "three/tsl";
import { ComputeNode, Node, StorageBufferNode, UniformArrayNode } from "three/webgpu";

import { toInFrustum } from "#/scene/static/static-frustum.tsl";
import { toOccluded } from "#/scene/static/static-occlusion.tsl";
import {
  EStaticCullState,
  EStaticLodState,
  EStaticPool,
  STATIC_CULL_COUNTS,
  STATIC_DRAW_ARGUMENTS,
  STATIC_LOD_IMPOSTOR_ROW,
  STATIC_NO_LOD,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

/**
 * Whether a row's clump draws what the row is: a tree while its trees are near enough, the impostor's own draw while it
 * is far enough; a row no impostor stands in for always.
 *
 * @param row - The row's impostor word, `STATIC_NO_LOD` for none.
 * @param terms - What the LOD cull decided, an impostor each.
 */
function toLodDrawn(row: Node<"uint">, terms: StorageBufferNode<"uvec4">): Node<"bool"> {
  const lod: Node<"uint"> = row.bitAnd(STATIC_LOD_IMPOSTOR_ROW - 1);
  const wanted: Node<"uint"> = select(
    row.bitAnd(STATIC_LOD_IMPOSTOR_ROW).notEqual(0),
    uint(EStaticLodState.IMPOSTOR),
    uint(EStaticLodState.TREES)
  );

  return row.equal(STATIC_NO_LOD).or((terms.element(lod) as unknown as Node<"uvec4">).w.bitAnd(wanted).notEqual(0));
}

/**
 * The first cull of the instanced draws, one invocation a row: a row whose sphere reaches into the view and which the
 * last frame's depth does not hide counts its draw's instance count up by one and writes its place into the draw's
 * list at the index that took; one the depth hides is left for the second cull. Runs after the first slot cull, which
 * leaves every instanced draw's instance counts at none.
 *
 * @param buffers - The static draw buffers.
 * @param planes - The view's six planes, the slot cull's own.
 * @returns The compute pass.
 */
export function createEarlyInstanceCullShader(
  buffers: StaticDrawBuffers,
  planes: UniformArrayNode<string>
): ComputeNode {
  const rows: number = buffers.capacity(EStaticPool.ROWS);
  const args = storage(buffers.args, "uint", buffers.capacity(EStaticPool.SLOTS) * STATIC_DRAW_ARGUMENTS).toAtomic();
  const rowLods = storage(buffers.rowLods, "uint", rows).toReadOnly();
  const lodTerms = storage(buffers.lodTerms, "uvec4", buffers.capacity(EStaticPool.LODS)).toReadOnly();
  const spheres = storage(buffers.rowSpheres, "vec4", rows).toReadOnly();
  const targets = storage(buffers.rowTargets, "uvec4", rows).toReadOnly();
  const visible = storage(buffers.visible, "uint", rows * 2);
  const states = storage(buffers.rowStates, "uint", rows);
  const pyramid = storage(buffers.pyramid, "float", buffers.capacity(EStaticPool.PYRAMID)).toReadOnly();
  const counts = storage(buffers.counts, "uint", STATIC_CULL_COUNTS).toAtomic();

  return Fn(() => {
    const sphere = spheres.element(instanceIndex);
    const state = uint(EStaticCullState.OUTSIDE).toVar();

    If(
      toLodDrawn(rowLods.element(instanceIndex) as unknown as Node<"uint">, lodTerms).and(
        toInFrustum(sphere, planes).equal(1)
      ),
      () => {
        state.assign(EStaticCullState.DRAWN);

        If(toOccluded(sphere, buffers.occlusion.previous, pyramid, buffers.occlusion).equal(1), () => {
          state.assign(EStaticCullState.OCCLUDED);
        });
      }
    );

    states.element(instanceIndex).assign(state);

    If(state.equal(EStaticCullState.DRAWN), () => {
      const target = targets.element(instanceIndex);
      const kept = atomicAdd(args.element(target.y.mul(STATIC_DRAW_ARGUMENTS).add(1)), uint(1));

      visible.element(target.z.add(kept)).assign(target.x);

      // A draw counts once, on its first kept instance; every instance counts its indices.
      If(kept.equal(0), () => {
        atomicAdd(counts.element(0), uint(1));
      });
      atomicAdd(counts.element(1), target.w);
    });
  })().compute(rows);
}

/**
 * The second cull of the instanced draws: a row the first left counts its draw's second instance count up where this
 * frame's depth so far does not hide it, listing its place in the second half of the list, a row capacity on; one it
 * still hides is counted as occluded.
 *
 * @param buffers - The static draw buffers.
 * @returns The compute pass.
 */
export function createLateInstanceCullShader(buffers: StaticDrawBuffers): ComputeNode {
  const rows: number = buffers.capacity(EStaticPool.ROWS);
  const lateArgs = storage(
    buffers.lateArgs,
    "uint",
    buffers.capacity(EStaticPool.SLOTS) * STATIC_DRAW_ARGUMENTS
  ).toAtomic();
  const spheres = storage(buffers.rowSpheres, "vec4", rows).toReadOnly();
  const targets = storage(buffers.rowTargets, "uvec4", rows).toReadOnly();
  const visible = storage(buffers.visible, "uint", rows * 2);
  const states = storage(buffers.rowStates, "uint", rows).toReadOnly();
  const pyramid = storage(buffers.pyramid, "float", buffers.capacity(EStaticPool.PYRAMID)).toReadOnly();
  const counts = storage(buffers.counts, "uint", STATIC_CULL_COUNTS).toAtomic();

  return Fn(() => {
    If(states.element(instanceIndex).equal(EStaticCullState.OCCLUDED), () => {
      const sphere = spheres.element(instanceIndex);

      If(toOccluded(sphere, buffers.occlusion.current, pyramid, buffers.occlusion).equal(0), () => {
        const target = targets.element(instanceIndex);
        const kept = atomicAdd(lateArgs.element(target.y.mul(STATIC_DRAW_ARGUMENTS).add(1)), uint(1));

        visible.element(target.z.add(rows).add(kept)).assign(target.x);

        If(kept.equal(0), () => {
          atomicAdd(counts.element(0), uint(1));
        });
        atomicAdd(counts.element(1), target.w);
      }).Else(() => {
        atomicAdd(counts.element(3), uint(1));
        atomicAdd(counts.element(4), targets.element(instanceIndex).w);
      });
    });
  })().compute(rows);
}
