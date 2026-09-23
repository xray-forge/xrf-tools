import { atomicAdd, Fn, If, instanceIndex, storage, uint } from "three/tsl";
import { ComputeNode, UniformArrayNode } from "three/webgpu";

import { toInFrustum } from "#/scene/static/static-frustum.tsl";
import { toOccluded } from "#/scene/static/static-occlusion.tsl";
import {
  EStaticCullState,
  STATIC_DRAW_ARGUMENTS,
  STATIC_DRAW_CAPACITY,
  STATIC_PYRAMID_CAPACITY,
  STATIC_ROW_CAPACITY,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

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
  const args = storage(buffers.args, "uint", STATIC_DRAW_CAPACITY * STATIC_DRAW_ARGUMENTS).toAtomic();
  const spheres = storage(buffers.rowSpheres, "vec4", STATIC_ROW_CAPACITY).toReadOnly();
  const targets = storage(buffers.rowTargets, "uvec4", STATIC_ROW_CAPACITY).toReadOnly();
  const visible = storage(buffers.visible, "uint", STATIC_ROW_CAPACITY * 2);
  const states = storage(buffers.rowStates, "uint", STATIC_ROW_CAPACITY);
  const pyramid = storage(buffers.pyramid, "float", STATIC_PYRAMID_CAPACITY).toReadOnly();
  const counts = storage(buffers.counts, "uint", 2).toAtomic();

  return Fn(() => {
    const sphere = spheres.element(instanceIndex);
    const state = uint(EStaticCullState.OUTSIDE).toVar();

    If(toInFrustum(sphere, planes).equal(1), () => {
      state.assign(EStaticCullState.DRAWN);

      If(toOccluded(sphere, buffers.occlusion.previous, pyramid, buffers.occlusion).equal(1), () => {
        state.assign(EStaticCullState.OCCLUDED);
      });
    });

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
  })().compute(STATIC_ROW_CAPACITY);
}

/**
 * The second cull of the instanced draws: a row the first left counts its draw's second instance count up where this
 * frame's depth so far does not hide it, listing its place in the second half of the list.
 *
 * @param buffers - The static draw buffers.
 * @returns The compute pass.
 */
export function createLateInstanceCullShader(buffers: StaticDrawBuffers): ComputeNode {
  const lateArgs = storage(buffers.lateArgs, "uint", STATIC_DRAW_CAPACITY * STATIC_DRAW_ARGUMENTS).toAtomic();
  const spheres = storage(buffers.rowSpheres, "vec4", STATIC_ROW_CAPACITY).toReadOnly();
  const targets = storage(buffers.rowTargets, "uvec4", STATIC_ROW_CAPACITY).toReadOnly();
  const visible = storage(buffers.visible, "uint", STATIC_ROW_CAPACITY * 2);
  const states = storage(buffers.rowStates, "uint", STATIC_ROW_CAPACITY).toReadOnly();
  const pyramid = storage(buffers.pyramid, "float", STATIC_PYRAMID_CAPACITY).toReadOnly();
  const counts = storage(buffers.counts, "uint", 2).toAtomic();

  return Fn(() => {
    If(states.element(instanceIndex).equal(EStaticCullState.OCCLUDED), () => {
      const sphere = spheres.element(instanceIndex);

      If(toOccluded(sphere, buffers.occlusion.current, pyramid, buffers.occlusion).equal(0), () => {
        const target = targets.element(instanceIndex);
        const kept = atomicAdd(lateArgs.element(target.y.mul(STATIC_DRAW_ARGUMENTS).add(1)), uint(1));

        visible.element(target.z.add(STATIC_ROW_CAPACITY).add(kept)).assign(target.x);

        If(kept.equal(0), () => {
          atomicAdd(counts.element(0), uint(1));
        });
        atomicAdd(counts.element(1), target.w);
      });
    });
  })().compute(STATIC_ROW_CAPACITY);
}
