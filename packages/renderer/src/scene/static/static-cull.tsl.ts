import { atomicAdd, Fn, If, instanceIndex, storage, uint, uniformArray } from "three/tsl";
import { ComputeNode, UniformArrayNode, Vector4 } from "three/webgpu";

import { toInFrustum } from "#/scene/static/static-frustum.tsl";
import { createEarlyInstanceCullShader, createLateInstanceCullShader } from "#/scene/static/static-instance-cull.tsl";
import { toOccluded } from "#/scene/static/static-occlusion.tsl";
import {
  EStaticCullState,
  EStaticPool,
  STATIC_CULL_COUNTS,
  STATIC_DRAW_ARGUMENTS,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

/**
 * The culls of the static draws, a frame's worth: the first before anything draws, the second once the first's draws
 * have drawn their depth. Each culls the single draws by slot, then the instanced ones by row. Built over the buffers
 * as they are laid out, and dispatched only as far as their slots and rows are used.
 */
export interface IStaticCullShader {
  early: Array<ComputeNode>;
  late: Array<ComputeNode>;
  /** Six planes, normals pointing in, `w` the constant. */
  planes: ReadonlyArray<Vector4>;
}

/**
 * @param buffers - The static draw buffers, as they are laid out now.
 * @returns Both culls, one invocation a slot or a row, and the planes they read.
 */
export function createStaticCullShader(buffers: StaticDrawBuffers): IStaticCullShader {
  const planes: Array<Vector4> = Array.from({ length: 6 }, () => new Vector4());
  const planeNodes = uniformArray(planes, "vec4");

  return {
    early: [createEarlySlotCullShader(buffers, planeNodes), createEarlyInstanceCullShader(buffers, planeNodes)],
    late: [createLateSlotCullShader(buffers), createLateInstanceCullShader(buffers)],
    planes,
  };
}

/**
 * The first cull of the single draws: a slot draws where its sphere reaches into the view and the last frame's depth
 * does not hide it; one it hides is left for the second cull. Every slot's second instance count starts at none.
 */
function createEarlySlotCullShader(buffers: StaticDrawBuffers, planes: UniformArrayNode<string>): ComputeNode {
  const slots: number = buffers.capacity(EStaticPool.SLOTS);
  const args = storage(buffers.args, "uint", slots * STATIC_DRAW_ARGUMENTS);
  const lateArgs = storage(buffers.lateArgs, "uint", slots * STATIC_DRAW_ARGUMENTS);
  const spheres = storage(buffers.spheres, "vec4", slots).toReadOnly();
  const states = storage(buffers.slotStates, "uint", slots);
  const pyramid = storage(buffers.pyramid, "float", buffers.capacity(EStaticPool.PYRAMID)).toReadOnly();
  const counts = storage(buffers.counts, "uint", STATIC_CULL_COUNTS).toAtomic();

  return Fn(() => {
    const sphere = spheres.element(instanceIndex);
    const state = uint(EStaticCullState.OUTSIDE).toVar();
    const at = instanceIndex.mul(STATIC_DRAW_ARGUMENTS);

    If(toInFrustum(sphere, planes).equal(1), () => {
      state.assign(EStaticCullState.DRAWN);

      If(toOccluded(sphere, buffers.occlusion.previous, pyramid, buffers.occlusion).equal(1), () => {
        state.assign(EStaticCullState.OCCLUDED);
      });
    });

    args.element(at.add(1)).assign(uint(state.equal(EStaticCullState.DRAWN)));
    lateArgs.element(at.add(1)).assign(0);
    states.element(instanceIndex).assign(state);

    If(state.equal(EStaticCullState.DRAWN), () => {
      atomicAdd(counts.element(0), uint(1));
      atomicAdd(counts.element(1), args.element(at));
    });
  })().compute(slots);
}

/**
 * The second cull of the single draws: a slot the first left draws where this frame's depth so far does not hide it,
 * and is counted as occluded where it still does.
 */
function createLateSlotCullShader(buffers: StaticDrawBuffers): ComputeNode {
  const slots: number = buffers.capacity(EStaticPool.SLOTS);
  const lateArgs = storage(buffers.lateArgs, "uint", slots * STATIC_DRAW_ARGUMENTS);
  const spheres = storage(buffers.spheres, "vec4", slots).toReadOnly();
  const states = storage(buffers.slotStates, "uint", slots).toReadOnly();
  const pyramid = storage(buffers.pyramid, "float", buffers.capacity(EStaticPool.PYRAMID)).toReadOnly();
  const counts = storage(buffers.counts, "uint", STATIC_CULL_COUNTS).toAtomic();

  return Fn(() => {
    If(states.element(instanceIndex).equal(EStaticCullState.OCCLUDED), () => {
      const sphere = spheres.element(instanceIndex);
      const at = instanceIndex.mul(STATIC_DRAW_ARGUMENTS);

      If(toOccluded(sphere, buffers.occlusion.current, pyramid, buffers.occlusion).equal(0), () => {
        lateArgs.element(at.add(1)).assign(1);
        atomicAdd(counts.element(0), uint(1));
        atomicAdd(counts.element(1), lateArgs.element(at));
      }).Else(() => {
        atomicAdd(counts.element(2), uint(1));
        atomicAdd(counts.element(4), lateArgs.element(at));
      });
    });
  })().compute(slots);
}
