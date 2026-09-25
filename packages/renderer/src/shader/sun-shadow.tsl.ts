import { abs, float, Fn, If, int, min, mix, saturate, select, sqrt, texture, vec2, vec4 } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { RENDERER_MAX_SHADOW_CASCADES } from "#/contract/renderer-features";
import { loopNamed } from "#/shader/named-loop.tsl";
import { ShadowUniforms } from "#/uniforms/shadow-uniforms";

/** A cascade's component of a per-cascade vector. */
const COMPONENTS = ["x", "y", "z", "w"] as const;

/** The share of a map's edge a point is kept off, so the filter never reads past it into the next cascade's edge. */
const EDGE: number = 0.02;

/** How far from its centre, in shares of the map, the last cascade starts to fade out: the engine's `border`. */
const BORDER: number = 0.4;

/** Where the cascades stand for a point: still looking, blending the one found into the next, or done. */
const LOOKING: number = 0;
const BLENDING: number = 1;
const DONE: number = 2;

/**
 * How much of the sun reaches a point: the first cascade whose map holds it, compared over a square of texels the
 * filter reaches, one where it is lit and nothing where every texel stands nearer the sun. Within `blend` of that
 * cascade's edge, as a share of its width, the next cascade is sampled too and the two are mixed by how far in the
 * point stands, so the switch to a coarser map is never a line. The last cascade fades out over its outer fifth on the
 * side the camera looks towards, as the engine's far pass does (`accum_sun_far.ps`), so its square never shows; past
 * every cascade the sun reaches it whole. Depth is reversed, so a texel nearer the sun holds the larger value.
 *
 * @param position - The point, in world space.
 * @param normal - Its normal, in world space, which it is moved along first so a lit surface never shadows itself.
 * @param facing - How squarely it faces the sun, the cosine of the angle between its normal and the sun: a surface
 *   the light grazes is moved up to twice as far, since one texel of the map spans more of its depth there.
 * @param shadows - The cascades and what sampling takes.
 * @param maps - Each cascade's depth, one a cascade there can be.
 * @returns The sun's share, from nothing to one.
 */
export function toSunShadow(
  position: Node<"vec3">,
  normal: Node<"vec3">,
  facing: Node<"float">,
  shadows: ShadowUniforms,
  maps: ReadonlyArray<Texture>
): Node<"float"> {
  return Fn(() => {
    const lit = float(1).toVar();
    const stage = int(LOOKING).toVar();
    // How far in from its edge the point stands in the cascade that holds it, past the margin, in shares of its width.
    const inset = float(0).toVar();
    const lean = float(1)
      .add(sqrt(float(1).sub(saturate(facing).mul(saturate(facing)))))
      .toVar();

    for (let view = 0; view < RENDERER_MAX_SHADOW_CASCADES; view += 1) {
      If(stage.notEqual(DONE).and(shadows.count.greaterThan(view)), () => {
        const texel: Node<"float"> = shadows.texels[COMPONENTS[view]];
        const moved: Node<"vec3"> = position.add(normal.mul(shadows.bias.mul(texel).mul(lean)));
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
          const viewLit = toCascadeLit(view, clip, uv, shadows, maps);

          If(stage.equal(LOOKING), () => {
            lit.assign(viewLit);
            inset.assign(min(min(uv.x, uv.y), min(float(1).sub(uv.x), float(1).sub(uv.y))).sub(EDGE));
            // Blended into the next only inside the band, and only where there is a next.
            stage.assign(
              select(inset.lessThan(shadows.blend).and(shadows.count.greaterThan(view + 1)), int(BLENDING), int(DONE))
            );
          }).Else(() => {
            // At the edge the next cascade's, at the band's inner side the one that holds it.
            lit.assign(mix(viewLit, lit, saturate(inset.div(shadows.blend))));
            stage.assign(DONE);
          });
        }).Else(() => {
          // The next does not hold it either: the one that does alone.
          If(stage.equal(BLENDING), () => {
            stage.assign(DONE);
          });
        });
      });
    }

    return lit;
  })();
}

/**
 * @param view - A cascade that holds the point.
 * @param clip - The point in the cascade's clip space.
 * @param uv - Where it falls on the cascade's map.
 * @param shadows - The cascades and what sampling takes.
 * @param maps - Each cascade's depth.
 * @returns The sun's share at the point by that cascade alone, the last one faded out towards its far edge.
 */
function toCascadeLit(
  view: number,
  clip: Node<"vec4">,
  uv: Node<"vec2">,
  shadows: ShadowUniforms,
  maps: ReadonlyArray<Texture>
): Node<"float"> {
  const step = float(1).div(shadows.resolution).toVar();
  const total = float(0).toVar();
  const taps = float(0).toVar();

  // Each loop names its own counter: three names every loop's `i`, so the inner one would shadow the outer and the
  // filter would sample its diagonal alone.
  const filter = int(shadows.filter);

  loopNamed({ condition: "<=", end: filter, name: "tapX", start: filter.negate(), type: "int" }, (x: Node<"int">) => {
    loopNamed({ condition: "<=", end: filter, name: "tapY", start: filter.negate(), type: "int" }, (y: Node<"int">) => {
      // At level zero: a sample inside a branch takes no derivatives.
      const stored = texture(maps[view], uv.add(vec2(float(x), float(y)).mul(step))).level(int(0)).x;

      total.addAssign(select(clip.z.greaterThanEqual(stored), float(1), float(0)));
      taps.addAssign(1);
    });
  });

  const lit = total.div(taps).toVar();

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

  return lit;
}
