import {
  abs,
  acos,
  clamp,
  cos,
  cross,
  dot,
  float,
  floor,
  Fn,
  fract,
  getViewPosition,
  int,
  ivec2,
  length,
  max,
  min,
  mix,
  mod,
  normalize,
  PI,
  pow,
  round,
  saturate,
  screenCoordinate,
  select,
  sign,
  sin,
  texture,
  textureLoad,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { decodeOctahedral } from "#/shader/octahedral-normal.tsl";
import { AmbientOcclusionUniforms } from "#/uniforms/ambient-occlusion-uniforms";
import { CameraUniforms } from "#/uniforms/camera-uniforms";

/** How a quality searches: directions around the view, and steps each way along each. */
export interface IAmbientOcclusionSearch {
  slices: number;
  steps: number;
}

/** XeGTAO's `EffectFalloffRange`: the outer share of the radius over which an occluder fades out. */
const FALLOFF_RANGE: number = 0.615;
/** XeGTAO's `SampleDistributionPower`: steps crowd towards the point, where occluders matter most. */
const DISTRIBUTION_POWER: number = 2;
/** XeGTAO's `PixelTooCloseThreshold`: the first step lands this many pixels out, off the point's own texel. */
const TOO_CLOSE: number = 1.3;
/** Share of a point's distance that a neighbour's may differ by and still be denoised with it. */
const DENOISE_TOLERANCE: number = 0.02;

/** One point of the frame, from the G-buffer's depth at a texel. */
interface IOcclusionPoint {
  depth: Node<"float">;
  position: Node<"vec3">;
}

/** The point searched around, and how an occluder fades with its distance from it. */
interface IOcclusionOrigin {
  position: Node<"vec3">;
  toCamera: Node<"vec3">;
  falloffMul: Node<"float">;
  falloffAdd: Node<"float">;
}

/**
 * XeGTAO's main pass (`XeGTAO_MainPass`, MIT) on a target half the frame's size, a pixel per two by two of the frame:
 * the horizons each way along a few directions around the view, and the cosine weighted visibility between them.
 * The noise that turns the directions is tiled four by four and does not change from frame to frame, so the denoise
 * takes it out and a still view stays still.
 *
 * @param gbuffer - The G-buffer searched.
 * @param camera - The drawing camera's uniforms, which positions are rebuilt with.
 * @param uniforms - What the search is set to.
 * @param search - Directions and steps.
 * @returns Visibility in red, from none to all, and the point's distance along the view in green; none drawn there
 *   is all visible at no distance.
 */
export function toAmbientOcclusionSearch(
  gbuffer: IGBufferTextures,
  camera: CameraUniforms,
  uniforms: AmbientOcclusionUniforms,
  search: IAmbientOcclusionSearch
): Node<"vec4"> {
  const { slices, steps } = search;

  return Fn(() => {
    const frameSize: Node<"vec2"> = vec2(texture(gbuffer.depth).size(int(0)) as Node<"uvec2">);
    const lastPixel: Node<"vec2"> = floor(frameSize.sub(1).div(2));
    const pixel: Node<"vec2"> = floor(screenCoordinate.xy);

    function toPoint(at: Node<"vec2">): IOcclusionPoint {
      const texel: Node<"vec2"> = clamp(at, vec2(0), lastPixel).mul(2);
      const depth: Node<"float"> = textureLoad(gbuffer.depth, ivec2(texel)) as unknown as Node<"float">;

      return {
        depth,
        position: getViewPosition(texel.add(0.5).div(frameSize), depth, camera.projectionInverse),
      };
    }

    const center: IOcclusionPoint = toPoint(pixel);
    // Towards the camera a little, as XeGTAO moves it, against the depth's own imprecision.
    const position: Node<"vec3"> = center.position.mul(0.99999);
    const distance: Node<"float"> = position.z.negate();
    const normal: Node<"vec3"> = decodeOctahedral(textureLoad(gbuffer.normal, ivec2(pixel.mul(2))).xy);
    const toCamera: Node<"vec3"> = normalize(position.negate());

    const falloffRange: Node<"float"> = uniforms.radius.mul(FALLOFF_RANGE);
    const origin: IOcclusionOrigin = {
      falloffAdd: uniforms.radius
        .mul(1 - FALLOFF_RANGE)
        .div(falloffRange)
        .add(1),
      falloffMul: float(-1).div(falloffRange),
      position,
      toCamera,
    };
    const screenRadius: Node<"float"> = min(uniforms.radius.div(distance.mul(uniforms.spread)), uniforms.reach);
    const minStep: Node<"float"> = float(TOO_CLOSE).div(screenRadius);
    const sliceNoise: Node<"float"> = toTileNoise(pixel.x, pixel.y).add(0.5).div(16);
    const stepNoise: Node<"float"> = toTileNoise(pixel.y, pixel.x).div(16);
    const halfPi: Node<"float"> = PI.mul(0.5);

    let visibility: Node<"float"> = saturate(float(10).sub(screenRadius).div(100)).mul(0.5);

    for (let slice = 0; slice < slices; slice++) {
      const phi: Node<"float"> = sliceNoise.add(slice).div(slices).mul(PI);
      // Down the target is up the view.
      const omega: Node<"vec2"> = vec2(cos(phi), sin(phi).negate()).mul(screenRadius);
      const direction: Node<"vec3"> = vec3(cos(phi), sin(phi), 0);
      const orthoDirection: Node<"vec3"> = direction.sub(toCamera.mul(dot(direction, toCamera)));
      const axis: Node<"vec3"> = normalize(cross(orthoDirection, toCamera));
      const projectedNormal: Node<"vec3"> = normal.sub(axis.mul(dot(normal, axis)));
      const projectedLength: Node<"float"> = length(projectedNormal);
      const cosNormal: Node<"float"> = saturate(dot(projectedNormal, toCamera).div(max(projectedLength, 1e-4)));
      const n: Node<"float"> = sign(dot(orthoDirection, projectedNormal)).mul(acos(cosNormal));
      // The tangent plane each way: no horizon found lies below it.
      const lowCos0: Node<"float"> = cos(n.add(halfPi));
      const lowCos1: Node<"float"> = cos(n.sub(halfPi));

      let horizonCos0: Node<"float"> = lowCos0;
      let horizonCos1: Node<"float"> = lowCos1;

      for (let step = 0; step < steps; step++) {
        const noise: Node<"float"> = fract(stepNoise.add((slice + step * steps) * 0.6180339887));
        const s: Node<"float"> = pow(noise.add(step).div(steps), DISTRIBUTION_POWER).add(minStep);
        const offset: Node<"vec2"> = round(omega.mul(s));

        horizonCos0 = toHorizonCos(toPoint(pixel.add(offset)), origin, lowCos0, horizonCos0);
        horizonCos1 = toHorizonCos(toPoint(pixel.sub(offset)), origin, lowCos1, horizonCos1);
      }

      const h0: Node<"float"> = n.add(
        clamp(
          acos(clamp(horizonCos1, -1, 1))
            .negate()
            .sub(n),
          halfPi.negate(),
          halfPi
        )
      );
      const h1: Node<"float"> = n.add(clamp(acos(clamp(horizonCos0, -1, 1)).sub(n), halfPi.negate(), halfPi));
      const arc0: Node<"float"> = cosNormal
        .add(h0.mul(2).mul(sin(n)))
        .sub(cos(h0.mul(2).sub(n)))
        .div(4);
      const arc1: Node<"float"> = cosNormal
        .add(h1.mul(2).mul(sin(n)))
        .sub(cos(h1.mul(2).sub(n)))
        .div(4);

      visibility = visibility.add(mix(projectedLength, 1, 0.05).mul(arc0.add(arc1)));
    }

    const occluded: Node<"float"> = max(pow(max(visibility.div(slices), 1e-4), uniforms.power), 0.03);
    const isDrawn: Node<"bool"> = center.depth.greaterThan(0);
    // A point whose radius is less than a pixel across has nothing to search.
    const isSearched: Node<"bool"> = isDrawn.and(screenRadius.greaterThanEqual(1));

    return vec4(select(isSearched, occluded, float(1)), select(isDrawn, distance, float(0)), 0, 1);
  })();
}

/**
 * One way of XeGTAO's denoise, separable: the visibility averaged four pixels wide, each weighed by how
 * near its distance lies to the one a surface through the centre would have there, so an edge keeps its side.
 *
 * @param source - What the search or the other way wrote.
 * @param across - The pixel step: one along x or along y.
 * @returns The visibility denoised, and the distance as it was.
 */
export function toAmbientOcclusionDenoise(source: Texture, across: [number, number]): Node<"vec4"> {
  return Fn(() => {
    const lastPixel: Node<"vec2"> = vec2(texture(source).size(int(0)) as Node<"uvec2">).sub(1);
    const pixel: Node<"vec2"> = floor(screenCoordinate.xy);
    const step: Node<"vec2"> = vec2(across[0], across[1]);

    function toTexel(offset: number): Node<"vec4"> {
      return textureLoad(source, ivec2(clamp(pixel.add(step.mul(offset)), vec2(0), lastPixel)));
    }

    const center: Node<"vec4"> = toTexel(0);
    const distance: Node<"float"> = center.y;
    // The slope through the centre, from its nearest neighbours, so a floor seen edge on is not an edge.
    const slope: Node<"float"> = toTexel(1).y.sub(toTexel(-1).y).mul(0.5);
    const tolerance: Node<"float"> = max(distance.mul(DENOISE_TOLERANCE), 1e-3);

    let sum: Node<"float"> = center.x;
    let weights: Node<"float"> = float(1);

    // Four pixels wide, the ends at half weight: every phase of the noise's four pixel tile counts once.
    for (const [offset, share] of [
      [-2, 0.5],
      [-1, 1],
      [1, 1],
      [2, 0.5],
    ]) {
      const texel: Node<"vec4"> = toTexel(offset);
      const delta: Node<"float"> = texel.y.sub(distance);
      const error: Node<"float"> = min(abs(delta), abs(delta.sub(slope.mul(offset))));
      const weight: Node<"float"> = saturate(float(1.25).sub(error.div(tolerance))).mul(share);

      sum = sum.add(texel.x.mul(weight));
      weights = weights.add(weight);
    }

    return vec4(sum.div(weights), distance, 0, 1);
  })();
}

/**
 * How far one sample raises a horizon: up to it by how near it stands, one past the radius not at all.
 *
 * @param sample - The point sampled.
 * @param origin - The point searched around.
 * @param low - The horizon's cosine along the tangent plane, the lowest it can be.
 * @param horizon - Its cosine so far.
 * @returns Its cosine with the sample.
 */
function toHorizonCos(
  sample: IOcclusionPoint,
  origin: IOcclusionOrigin,
  low: Node<"float">,
  horizon: Node<"float">
): Node<"float"> {
  const delta: Node<"vec3"> = sample.position.sub(origin.position);
  const distance: Node<"float"> = max(length(delta), 1e-4);
  const weight: Node<"float"> = saturate(distance.mul(origin.falloffMul).add(origin.falloffAdd));
  const cosine: Node<"float"> = mix(low, dot(delta.div(distance), origin.toCamera), weight);

  // Nothing drawn there hides nothing.
  return max(horizon, select(sample.depth.greaterThan(0), cosine, low));
}

/**
 * A four by four ordered dither, each of sixteen values once in every tile.
 *
 * @param x - A pixel's column.
 * @param y - And its row.
 * @returns Its value in the tile, zero to fifteen.
 */
function toTileNoise(x: Node<"float">, y: Node<"float">): Node<"float"> {
  // Two by two `[[0, 2], [3, 1]]`: twice the bits' difference, plus the row's.
  function toPair(a: Node<"float">, b: Node<"float">): Node<"float"> {
    return abs(a.sub(b)).mul(2).add(b);
  }

  return toPair(mod(x, 2), mod(y, 2))
    .mul(4)
    .add(toPair(mod(floor(x.div(2)), 2), mod(floor(y.div(2)), 2)));
}
