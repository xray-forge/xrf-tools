import {
  abs,
  clamp,
  exp,
  float,
  floor,
  Fn,
  getViewPosition,
  length,
  luminance,
  max,
  mix,
  outputStruct,
  saturate,
  screenUV,
  select,
  sqrt,
  texture,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { DepthTexture, Node, Texture } from "three/webgpu";

import { toNearestDrawnTexel, toUpscaledCoverage } from "#/shader/drawn-sample.tsl";
import { loadClamped, loadDepth, NEIGHBOURHOOD, toTextureSize } from "#/shader/texel.tsl";
import { CameraUniforms } from "#/uniforms/camera-uniforms";
import { MotionUniforms } from "#/uniforms/motion-uniforms";
import { TemporalUniforms } from "#/uniforms/temporal-uniforms";

/** What the resolve reads: this frame at its own size, and the history at the output's. */
export interface ITemporalInputs {
  /** The tonemapped frame, drawn jittered. */
  frame: Texture;
  depth: DepthTexture;
  /** How far each pixel's surface moved since the frame before, in texture coordinates, the sky's included. */
  motion: Texture;
  /** The resolved frame before: colour, and the distance along the view of what it showed. */
  history: Texture;
}

/** The uniforms the resolve reads. */
export interface ITemporalUniforms {
  camera: CameraUniforms;
  motion: MotionUniforms;
  temporal: TemporalUniforms;
}

/** Share of a point's distance its history's may differ by and still be taken as the same surface. */
const DISTANCE_TOLERANCE: number = 0.1;
/** Output pixels of motion at which the history counts for nothing more than the neighbourhood lets it. */
const MAX_MOTION: number = 128;
/** `exp(-2.29 x²)`'s factor: Karis's fit of a Blackman-Harris window a pixel wide. */
const WINDOW: number = -2.29;

/**
 * The temporal resolve, after Karis's "High Quality Temporal Supersampling" and three's `TAAUNode` (MIT), upscaling as
 * TAAU where the frame is drawn smaller than the output.
 *
 * @param inputs - What the resolve reads.
 * @param uniforms - The uniforms it reads.
 * @returns Two outputs: the history, colour and distance, and the frame as shown, colour and coverage.
 */
export function toTemporalResolve(inputs: ITemporalInputs, uniforms: ITemporalUniforms): Node {
  const { camera, motion, temporal } = uniforms;
  const inputSize: Node<"vec2"> = toTextureSize(inputs.frame);
  const outputSize: Node<"vec2"> = toTextureSize(inputs.history);
  // Output pixels a drawn one spans across; one unscaled.
  const upscale: Node<"float"> = outputSize.x.div(inputSize.x);

  function toDepth(at: Node<"vec2">): Node<"float"> {
    return loadDepth(inputs.depth, at, inputSize);
  }

  const resolved: Node<"vec4"> = Fn(() => {
    const uv: Node<"vec2"> = screenUV;
    const position: Node<"vec2"> = uv.mul(inputSize);
    const nearest: Node<"vec2"> = toNearestDrawnTexel(inputSize, uv, motion.jitter);

    // The nearest surface around it, whose motion carries an edge's history with the edge rather than its background.
    let closestDepth: Node<"float"> = float(0);
    let closest: Node<"vec2"> = nearest;

    for (const [x, y] of NEIGHBOURHOOD) {
      const at: Node<"vec2"> = nearest.add(vec2(x, y));
      const depth: Node<"float"> = toDepth(at);
      const isCloser: Node<"bool"> = depth.greaterThan(closestDepth);

      closestDepth = select(isCloser, depth, closestDepth);
      closest = select(isCloser, at, closest);
    }

    const depth: Node<"float"> = toDepth(nearest);
    const isDrawn: Node<"bool"> = depth.greaterThan(0);
    const view: Node<"vec3"> = getViewPosition(nearest.add(0.5).div(inputSize), depth, camera.projectionInverse);
    const world: Node<"vec3"> = camera.viewToWorld.mul(vec4(view, 1)).xyz;
    const distance: Node<"float"> = view.z.negate();
    const moved: Node<"vec2"> = loadClamped(inputs.motion, closest, inputSize).xy;
    const before: Node<"vec2"> = uv.sub(moved);

    // The history stands for this surface where it showed something as far from the camera as this point was then.
    const isInside: Node<"bool"> = before
      .greaterThanEqual(vec2(0))
      .all()
      .and(before.lessThanEqual(vec2(1)).all());
    const historyDistance: Node<"float"> = loadClamped(inputs.history, floor(before.mul(outputSize)), outputSize).w;
    const expected: Node<"float"> = motion.previousView.mul(vec4(world, 1)).z.negate();
    const isSameSurface: Node<"bool"> = select(
      isDrawn,
      abs(historyDistance.sub(expected)).lessThanEqual(expected.mul(DISTANCE_TOLERANCE)),
      historyDistance.lessThanEqual(0)
    );
    const isValid: Node<"bool"> = temporal.isHistoryValid.greaterThan(0.5).and(isInside).and(isSameSurface);

    // This frame's colour where the pixel's centre is, and the spread of colours around it.
    let sum: Node<"vec3"> = vec3(0);
    let weights: Node<"float"> = float(0);
    let moment: Node<"vec3"> = vec3(0);
    let momentSquared: Node<"vec3"> = vec3(0);

    for (const [x, y] of NEIGHBOURHOOD) {
      const at: Node<"vec2"> = nearest.add(vec2(x, y));
      const color: Node<"vec3"> = max(loadClamped(inputs.frame, at, inputSize).xyz, vec3(0));
      const offset: Node<"vec2"> = position.sub(at.add(0.5).add(motion.jitter));
      const weight: Node<"float"> = exp(offset.dot(offset).mul(WINDOW));

      sum = sum.add(color.mul(weight));
      weights = weights.add(weight);
      moment = moment.add(color);
      momentSquared = momentSquared.add(color.mul(color));
    }

    const current: Node<"vec3"> = sum.div(max(weights, 1e-5));
    const mean: Node<"vec3"> = moment.div(NEIGHBOURHOOD.length);
    const motionShare: Node<"float"> = saturate(length(moved.mul(outputSize)).div(MAX_MOTION));
    // Wider while still, so a sub-pixel edge keeps its history; tighter while moving, so nothing trails.
    const spread: Node<"vec3"> = sqrt(max(momentSquared.div(NEIGHBOURHOOD.length).sub(mean.mul(mean)), vec3(0))).mul(
      mix(float(0.5), float(1), motionShare.oneMinus().mul(motionShare.oneMinus()))
    );
    const history: Node<"vec3"> = toClipped(
      toCatmullRom(inputs.history, before, outputSize),
      mean,
      mean.sub(spread),
      mean.add(spread)
    );
    // The nearest sample's distance from this pixel's centre, in output pixels, weighs this frame; unscaled, as is.
    const landed: Node<"vec2"> = position.sub(nearest.add(0.5).add(motion.jitter)).mul(upscale);
    const confidence: Node<"float"> = select(
      upscale.greaterThan(1.001),
      exp(landed.dot(landed).mul(WINDOW)).mul(upscale.mul(upscale)),
      float(1)
    );
    const weight: Node<"float"> = select(
      isValid,
      saturate(temporal.currentWeight.mul(confidence).add(motionShare)),
      float(1)
    );

    return vec4(toBlended(current, history, weight), select(isDrawn, distance, float(0)));
  })();

  return outputStruct(resolved, vec4(resolved.xyz, toUpscaledCoverage(inputs.frame, motion.jitter)));
}

/**
 * The history clipped towards the neighbourhood's mean onto the box of its colours (Playdead's `clip_aabb`), so a
 * colour the neighbourhood no longer has fades out rather than trails.
 */
function toClipped(
  history: Node<"vec3">,
  mean: Node<"vec3">,
  minimum: Node<"vec3">,
  maximum: Node<"vec3">
): Node<"vec3"> {
  const center: Node<"vec3"> = clamp(mean, minimum, maximum);
  const extent: Node<"vec3"> = maximum.sub(minimum).mul(0.5).add(1e-7);
  const offset: Node<"vec3"> = history.sub(center);
  const units: Node<"vec3"> = abs(offset.div(extent));
  const reach: Node<"float"> = max(units.x, max(units.y, units.z));

  return select(reach.greaterThan(1), center.add(offset.div(reach)), history);
}

/** This frame and the history blended, each weighed down by its brightness so a bright sample does not flicker. */
function toBlended(current: Node<"vec3">, history: Node<"vec3">, weight: Node<"float">): Node<"vec3"> {
  const currentWeight: Node<"float"> = weight.div(luminance(current).add(1));
  const historyWeight: Node<"float"> = weight.oneMinus().div(luminance(history).add(1));

  return current
    .mul(currentWeight)
    .add(history.mul(historyWeight))
    .div(max(currentWeight.add(historyWeight), 1e-5));
}

/**
 * The history at a point, Catmull-Rom filtered in five bilinear fetches: a bilinear one blurs a moving view a little
 * more every frame it is resampled.
 */
function toCatmullRom(history: Texture, at: Node<"vec2">, size: Node<"vec2">): Node<"vec3"> {
  const position: Node<"vec2"> = at.mul(size);
  const center: Node<"vec2"> = floor(position.sub(0.5)).add(0.5);
  const f: Node<"vec2"> = position.sub(center);
  const w0: Node<"vec2"> = f.mul(f.mul(f.mul(-0.5).add(1)).sub(0.5));
  const w1: Node<"vec2"> = f.mul(f).mul(f.mul(1.5).sub(2.5)).add(1);
  const w2: Node<"vec2"> = f.mul(f.mul(f.mul(-1.5).add(2)).add(0.5));
  const w3: Node<"vec2"> = f.mul(f).mul(f.mul(0.5).sub(0.5));
  const w12: Node<"vec2"> = w1.add(w2);
  const at0: Node<"vec2"> = center.sub(1).div(size);
  const at3: Node<"vec2"> = center.add(2).div(size);
  const at12: Node<"vec2"> = center.add(w2.div(w12)).div(size);

  function toSample(x: Node<"float">, y: Node<"float">, weight: Node<"float">): Node<"vec4"> {
    return vec4(texture(history, vec2(x, y)).xyz.mul(weight), weight);
  }

  const total: Node<"vec4"> = toSample(at12.x, at0.y, w12.x.mul(w0.y))
    .add(toSample(at0.x, at12.y, w0.x.mul(w12.y)))
    .add(toSample(at12.x, at12.y, w12.x.mul(w12.y)))
    .add(toSample(at3.x, at12.y, w3.x.mul(w12.y)))
    .add(toSample(at12.x, at3.y, w12.x.mul(w3.y)));

  return max(total.xyz.div(max(total.w, 1e-5)), vec3(0));
}
