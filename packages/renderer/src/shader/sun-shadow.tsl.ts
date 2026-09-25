import { abs, float, Fn, If, int, Loop, saturate, select, texture, vec2, vec4 } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { RENDERER_MAX_SHADOW_CASCADES } from "#/contract/renderer-features";
import { ShadowUniforms } from "#/uniforms/shadow-uniforms";

/** A cascade's component of a per-cascade vector. */
const COMPONENTS = ["x", "y", "z", "w"] as const;

/** The share of a map's edge a point is kept off, so the filter never reads past it into the next cascade's edge. */
const EDGE: number = 0.02;

/** How far from its centre, in shares of the map, the last cascade starts to fade out: the engine's `border`. */
const BORDER: number = 0.4;

/**
 * How much of the sun reaches a point: the first cascade whose map holds it, compared over a square of texels the
 * filter reaches, one where it is lit and nothing where every texel stands nearer the sun. The last cascade fades out
 * over its outer fifth on the side the camera looks towards, as the engine's far pass does (`accum_sun_far.ps`), so
 * its square never shows; past every cascade the sun reaches it whole. Depth is reversed, so a texel nearer the sun
 * holds the larger value.
 *
 * @param position - The point, in world space.
 * @param normal - Its normal, in world space, which it is moved along first so a lit surface never shadows itself.
 * @param shadows - The cascades and what sampling takes.
 * @param maps - Each cascade's depth, one a cascade there can be.
 * @returns The sun's share, from nothing to one.
 */
export function toSunShadow(
  position: Node<"vec3">,
  normal: Node<"vec3">,
  shadows: ShadowUniforms,
  maps: ReadonlyArray<Texture>
): Node<"float"> {
  return Fn(() => {
    const lit = float(1).toVar();
    const isFound = int(0).toVar();

    for (let view = 0; view < RENDERER_MAX_SHADOW_CASCADES; view += 1) {
      If(isFound.equal(0).and(shadows.count.greaterThan(view)), () => {
        const texel: Node<"float"> = shadows.texels[COMPONENTS[view]];
        const moved: Node<"vec3"> = position.add(normal.mul(shadows.bias.mul(texel)));
        const clip = shadows.matrices[view].mul(vec4(moved, 1)).toVar();
        const uv = vec2(clip.x.mul(0.5).add(0.5), clip.y.mul(-0.5).add(0.5)).toVar();
        const isInside: Node<"bool"> = uv.x
          .greaterThan(EDGE)
          .and(uv.x.lessThan(1 - EDGE))
          .and(uv.y.greaterThan(EDGE))
          .and(uv.y.lessThan(1 - EDGE))
          .and(clip.z.greaterThan(0))
          .and(clip.z.lessThan(1));

        If(isInside, () => {
          const step = float(1).div(shadows.resolution).toVar();
          const total = float(0).toVar();
          const taps = float(0).toVar();

          isFound.assign(1);

          Loop({ condition: "<=", end: int(shadows.filter), start: int(shadows.filter).negate() }, ({ i: x }) => {
            Loop({ condition: "<=", end: int(shadows.filter), start: int(shadows.filter).negate() }, ({ i: y }) => {
              // At level zero: a sample inside a branch takes no derivatives.
              const stored = texture(maps[view], uv.add(vec2(float(x), float(y)).mul(step))).level(int(0)).x;

              total.addAssign(select(clip.z.greaterThanEqual(stored), float(1), float(0)));
              taps.addAssign(1);
            });
          });

          lit.assign(total.div(taps));

          If(shadows.count.equal(view + 1), () => {
            // The view's direction in the map, and how far out the point stands from its centre, on that side only.
            const ahead = shadows.matrices[view].mul(vec4(shadows.forward, 0));
            const offset = uv.sub(0.5).toVar();
            const isAhead = offset.dot(vec2(ahead.x, ahead.y.negate())).greaterThanEqual(0);
            const out = select(isAhead, abs(offset), vec2(0));
            const kept = float(1)
              .sub(saturate(out.x.sub(BORDER).div(0.5 - BORDER)))
              .mul(float(1).sub(saturate(out.y.sub(BORDER).div(0.5 - BORDER))));

            lit.assign(lit.add(float(1).sub(lit).mul(float(1).sub(kept))));
          });
        });
      });
    }

    return lit;
  })();
}
