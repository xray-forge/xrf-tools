import { clamp, float, floor, Fn, If, instanceIndex, max, normalize, storage, uint, uvec4 } from "three/tsl";
import { ComputeNode, Node } from "three/webgpu";

import { RENDERER_IMPOSTOR_FACETS } from "#/contract/scene/renderer-impostors";
import { EStaticLodState, EStaticPool, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** `EPS`, what `CalcSSA` adds to a squared distance so a camera standing at a centre divides by something. */
const DISTANCE_EPSILON: number = 0.00001;

/** `EPS_S`, the least the fade's range is taken as. */
const RANGE_EPSILON: number = 0.0000001;

/**
 * `add_leafs_static`'s `MT_LOD` case and `render_lods`'s terms, one invocation an impostor. A clump's screen area is
 * its sphere's radius over its squared distance, times its `lod_factor`: above `r_ssaLOD_B` its trees draw, below
 * `r_ssaLOD_A` its impostor does, both between, neither below `r_ssaDISCARD`. For the impostor it picks the two facets
 * facing the camera best, how far to blend between them, and how far it has faded in, as the bytes the engine writes
 * into its vertices' `sun_af`.
 *
 * @param buffers - The static draw buffers, as they are laid out now.
 * @returns The compute pass.
 */
export function createLodCullShader(buffers: StaticDrawBuffers): ComputeNode {
  const lods: number = buffers.capacity(EStaticPool.LODS);
  const spheres = storage(buffers.lodSpheres, "vec4", lods).toReadOnly();
  const factors = storage(buffers.lodFactors, "float", lods).toReadOnly();
  const normals = storage(buffers.lodNormals, "vec4", lods * RENDERER_IMPOSTOR_FACETS).toReadOnly();
  const terms = storage(buffers.lodTerms, "uvec4", lods);
  const { lod } = buffers;

  return Fn(() => {
    const sphere = spheres.element(instanceIndex);
    const state = uint(0).toVar();
    const best = uint(0).toVar();
    const next = uint(0).toVar();
    const bytes = uint(0).toVar();

    If(sphere.w.greaterThan(0), () => {
      const offset: Node<"vec3"> = sphere.xyz.sub(lod.camera);
      const ssa: Node<"float"> = sphere.w
        .div(offset.dot(offset).add(DISTANCE_EPSILON))
        .mul(factors.element(instanceIndex))
        .toVar();

      If(lod.isEnabled.lessThan(0.5).or(ssa.greaterThan(lod.lodB)), () => {
        state.assign(EStaticLodState.TREES);
      });

      If(lod.isEnabled.greaterThan(0.5).and(ssa.lessThan(lod.lodA)).and(ssa.greaterThanEqual(lod.discard)), () => {
        state.assign(state.bitOr(EStaticLodState.IMPOSTOR));

        // The three facets facing the camera best, as `render_lods` sorts them.
        const direction: Node<"vec3"> = normalize(offset);
        const first = float(-2).toVar();
        const second = float(-2).toVar();
        const third = float(-2).toVar();

        for (let facet = 0; facet < RENDERER_IMPOSTOR_FACETS; facet += 1) {
          const facing: Node<"float"> = direction.dot(
            normals.element(instanceIndex.mul(RENDERER_IMPOSTOR_FACETS).add(facet)).xyz
          );

          If(facing.greaterThan(first), () => {
            third.assign(second);
            second.assign(first);
            next.assign(best);
            first.assign(facing);
            best.assign(facet);
          })
            .ElseIf(facing.greaterThan(second), () => {
              third.assign(second);
              second.assign(facing);
              next.assign(facet);
            })
            .ElseIf(facing.greaterThan(third), () => {
              third.assign(facing);
            });
        }

        const fade: Node<"float"> = float(1).sub(ssa.sub(lod.lodB).div(max(lod.lodA.sub(lod.lodB), RANGE_EPSILON)));
        const blend: Node<"float"> = float(0.5).add(
          float(0.5).mul(float(1).sub(second.sub(third).div(max(first.sub(third), RANGE_EPSILON))))
        );
        const alpha: Node<"uint"> = clamp(floor(fade.mul(255)), 0, 255).toUint();
        const factor: Node<"uint"> = clamp(floor(blend.mul(255.5)), 0, 255).toUint();

        bytes.assign(alpha.bitOr(factor.shiftLeft(8)));
      });
    });

    terms.element(instanceIndex).assign(uvec4(best, next, bytes, state));
  })().compute(lods);
}
