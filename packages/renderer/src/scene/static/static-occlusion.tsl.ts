import {
  ceil,
  clamp,
  float,
  floor,
  Fn,
  If,
  instanceIndex,
  ivec2,
  log2,
  max,
  min,
  select,
  storage,
  textureLoad,
  uint,
  uniform,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { ComputeNode, Node, StorageBufferNode, Texture, UniformNode } from "three/webgpu";

import { OcclusionUniforms } from "#/uniforms/occlusion-uniforms";
import { OcclusionView } from "#/uniforms/occlusion-view";
import { STATIC_PYRAMID_CAPACITY, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** Texels of the level below, or pixels of the depth, a pyramid texel takes the farthest of, per axis. */
export const PYRAMID_REDUCTION: number = 4;

/** A uniform the pyramid's builder sets per build. */
type TLevelUniform = UniformNode<"uint", number>;

/** One level of a depth pyramid being built, and the uniforms saying where it reads and writes. */
export interface IPyramidLevelShader {
  compute: ComputeNode;
  /** Where the level it reduces starts, and its width and height: the depth texture's size for the first level. */
  source: { offset: TLevelUniform; width: TLevelUniform; height: TLevelUniform };
  /** Where this level starts, and its width. */
  target: { offset: TLevelUniform; width: TLevelUniform };
}

/** The lesser of two unsigned integers: a texel clamped to the last of its row or column. */
function toLesser(a: Node<"uint">, b: Node<"uint">): Node<"uint"> {
  return select(a.lessThan(b), a, b) as unknown as Node<"uint">;
}

/** The depth a pyramid holds at one texel of a level. */
function toPyramidDepth(
  pyramid: StorageBufferNode<"float">,
  offset: Node<"uint">,
  width: Node<"uint">,
  x: Node<"uint">,
  y: Node<"uint">
): Node<"float"> {
  return pyramid.element(offset.add(y.mul(width)).add(x)) as unknown as Node<"float">;
}

/**
 * @param sphere - A sphere in renderer space.
 * @param view - The view the pyramid holds the depth of.
 * @param pyramid - The pyramid, read only.
 * @param occlusion - Its layout.
 * @returns Whether everything the view sees of the sphere lies behind what the pyramid holds there. Conservative: a
 *   sphere reaching past the near plane, too large for the pyramid, or tested against no view is never occluded.
 */
export function toOccluded(
  sphere: Node<"vec4">,
  view: OcclusionView,
  pyramid: StorageBufferNode<"float">,
  occlusion: OcclusionUniforms
): Node<"uint"> {
  return Fn(() => {
    const isOccluded = uint(0).toVar();
    const center = view.view.mul(vec4(sphere.xyz, 1)).xyz;
    const radius = sphere.w;

    // Wholly in front of the near plane, which looks down -z.
    If(view.isTaken.greaterThan(0.5).and(center.z.add(radius).lessThan(view.near.negate())), () => {
      const low = vec2(1e9).toVar();
      const high = vec2(-1e9).toVar();

      for (const x of [-1, 1]) {
        for (const y of [-1, 1]) {
          for (const z of [-1, 1]) {
            const clip = view.projection.mul(vec4(center.add(vec3(x, y, z).mul(radius)), 1));
            const ndc = clip.xy.div(clip.w);

            low.assign(min(low, ndc));
            high.assign(max(high, ndc));
          }
        }
      }

      const nearest = view.projection.mul(vec4(0, 0, center.z.add(radius), 1));
      const depth = nearest.z.div(nearest.w);
      const size = occlusion.size;
      // Pixels run down from the top, where normalised device y runs up.
      const first = clamp(vec2(low.x, high.y.negate()).mul(0.5).add(0.5).mul(size), vec2(0), size.sub(1));
      const last = clamp(vec2(high.x, low.y.negate()).mul(0.5).add(0.5).mul(size), vec2(0), size.sub(1));
      const extent = max(max(last.x.sub(first.x), last.y.sub(first.y)), 1);
      // The level a texel of which is at least as wide as the rectangle, so it spans two texels an axis at most.
      const level = max(ceil(log2(extent).div(2)).sub(1), 0).toInt();

      If(level.lessThan(occlusion.levelCount.toInt()), () => {
        const {
          x: offset,
          y: width,
          z: height,
          w: span,
        } = occlusion.levelNodes.element(level) as unknown as Record<"x" | "y" | "z" | "w", Node<"float">>;
        const x0: Node<"uint"> = min(floor(first.x.div(span)), width.sub(1)).toUint();
        const x1: Node<"uint"> = min(floor(last.x.div(span)), width.sub(1)).toUint();
        const y0: Node<"uint"> = min(floor(first.y.div(span)), height.sub(1)).toUint();
        const y1: Node<"uint"> = min(floor(last.y.div(span)), height.sub(1)).toUint();
        const start: Node<"uint"> = offset.toUint();
        const stride: Node<"uint"> = width.toUint();
        const farthest = max(
          max(toPyramidDepth(pyramid, start, stride, x0, y0), toPyramidDepth(pyramid, start, stride, x1, y0)),
          max(toPyramidDepth(pyramid, start, stride, x0, y1), toPyramidDepth(pyramid, start, stride, x1, y1))
        );

        If(depth.greaterThan(farthest), () => {
          isOccluded.assign(1);
        });
      });
    });

    return isOccluded;
  })();
}

/**
 * @param buffers - The static draw buffers, whose pyramid the levels write.
 * @param depth - The depth texture the first level reduces.
 * @returns A shader a level, the first reducing the depth texture and every other the level before it, each the
 *   farthest depth of a four by four block. Their counts are set per build, to the size of their level.
 */
export function createPyramidShaders(buffers: StaticDrawBuffers, depth: Texture): Array<IPyramidLevelShader> {
  const pyramid = storage(buffers.pyramid, "float", STATIC_PYRAMID_CAPACITY);

  return Array.from({ length: buffers.occlusion.levels.length }, (_, index: number) => {
    const source = { height: uniform(1, "uint"), offset: uniform(0, "uint"), width: uniform(1, "uint") };
    const target = { offset: uniform(0, "uint"), width: uniform(1, "uint") };

    const compute = Fn(() => {
      const x = instanceIndex.mod(target.width);
      const y = instanceIndex.div(target.width);
      const farthest = float(0).toVar();

      for (let dy = 0; dy < PYRAMID_REDUCTION; dy += 1) {
        for (let dx = 0; dx < PYRAMID_REDUCTION; dx += 1) {
          const sx: Node<"uint"> = toLesser(x.mul(PYRAMID_REDUCTION).add(dx), source.width.sub(1));
          const sy: Node<"uint"> = toLesser(y.mul(PYRAMID_REDUCTION).add(dy), source.height.sub(1));
          const value: Node<"float"> = index
            ? toPyramidDepth(pyramid, source.offset, source.width, sx, sy)
            : (textureLoad(depth, ivec2(sx.toInt(), sy.toInt())) as unknown as Node<"float">);

          farthest.assign(max(farthest, value));
        }
      }

      pyramid.element(target.offset.add(instanceIndex)).assign(farthest);
    })().compute(1);

    return { compute, source, target };
  });
}
