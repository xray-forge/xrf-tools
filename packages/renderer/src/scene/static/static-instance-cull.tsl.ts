import { atomicAdd, clamp, float, floor, Fn, If, instanceIndex, select, sqrt, storage, uint } from "three/tsl";
import { ComputeNode, Node, StorageBufferNode, UniformArrayNode } from "three/webgpu";

import { toInFrustum } from "#/scene/static/static-frustum.tsl";
import { toOccluded } from "#/scene/static/static-occlusion.tsl";
import { LodUniforms } from "#/uniforms/lod-uniforms";
import {
  EStaticCullState,
  EStaticLodState,
  EStaticPool,
  STATIC_CULL_COUNTS,
  STATIC_DRAW_ARGUMENTS,
  STATIC_LOD_IMPOSTOR_ROW,
  STATIC_NO_BAND,
  STATIC_NO_LOD,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

/** `EPS`, what `CalcSSA` adds to a squared distance so a camera standing at a centre divides by something. */
const DISTANCE_EPSILON: number = 0.00001;

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
 * Whether a row's band is the one its place draws at (`calcLOD`, `FTreeVisual_PM::Render`): the place's screen area
 * against the progressive thresholds gives its detail, the detail a window of the engine's table, and the window the
 * band it falls in. A row of a draw of one detail always does.
 *
 * @param word - The row's band word, `STATIC_NO_BAND` for none.
 * @param sphere - The place's sphere.
 * @param lod - The thresholds and the camera they are measured from.
 */
function toBandDrawn(word: Node<"uint">, sphere: Node<"vec4">, lod: LodUniforms): Node<"bool"> {
  const band: Node<"uint"> = word.bitAnd(255);
  const bands: Node<"uint"> = word.shiftRight(8).bitAnd(255);
  const windows: Node<"uint"> = word.shiftRight(16);
  const offset: Node<"vec3"> = sphere.xyz.sub(lod.camera);
  const ssa: Node<"float"> = sphere.w.div(offset.dot(offset).add(DISTANCE_EPSILON));
  const detail: Node<"float"> = sqrt(clamp(ssa.sub(lod.glodEnd).div(lod.glodStart.sub(lod.glodEnd)), 0, 1));
  const window: Node<"uint"> = floor(
    float(1)
      .sub(detail)
      .mul(float(windows.sub(1)))
      .add(0.5)
  ).toUint();

  // The window is at most `windows - 1`, so the band it falls in is always one of the draw's.
  return word.equal(STATIC_NO_BAND).or(window.mul(bands).div(windows).equal(band));
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
  const rowLods = storage(buffers.rowLods, "uvec2", rows).toReadOnly();
  const lodTerms = storage(buffers.lodTerms, "uvec4", buffers.capacity(EStaticPool.LODS)).toReadOnly();
  const spheres = storage(buffers.rowSpheres, "vec4", rows).toReadOnly();
  const targets = storage(buffers.rowTargets, "uvec4", rows).toReadOnly();
  const visible = storage(buffers.visible, "uint", rows * 2);
  const states = storage(buffers.rowStates, "uint", rows);
  const pyramid = storage(buffers.pyramid, "float", buffers.capacity(EStaticPool.PYRAMID)).toReadOnly();
  const counts = storage(buffers.counts, "uint", STATIC_CULL_COUNTS).toAtomic();

  return Fn(() => {
    const sphere = spheres.element(instanceIndex);
    const words = rowLods.element(instanceIndex) as unknown as Node<"uvec2">;
    const state = uint(EStaticCullState.OUTSIDE).toVar();

    If(
      toLodDrawn(words.x, lodTerms)
        .and(toBandDrawn(words.y, sphere as unknown as Node<"vec4">, buffers.lod))
        .and(toInFrustum(sphere, planes).equal(1)),
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
