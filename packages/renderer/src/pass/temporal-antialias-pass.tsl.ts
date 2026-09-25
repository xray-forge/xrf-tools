import {
  abs,
  clamp,
  exp,
  float,
  floor,
  Fn,
  getViewPosition,
  int,
  ivec2,
  length,
  luminance,
  max,
  mix,
  outputStruct,
  round,
  saturate,
  screenUV,
  select,
  sqrt,
  texture,
  textureLoad,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { DepthTexture, Node, Texture } from "three/webgpu";

import { CameraUniforms } from "#/uniforms/camera-uniforms";
import { MotionUniforms } from "#/uniforms/motion-uniforms";
import { TemporalUniforms } from "#/uniforms/temporal-uniforms";

/** What the resolve reads: this frame at its own size, and the history at the output's. */
export interface ITemporalInputs {
  /** The tonemapped frame, drawn jittered. */
  frame: Texture;
  depth: DepthTexture;
  /** How far each pixel's surface moved since the frame before, in texture coordinates. */
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
/** The 3x3 neighbourhood around a sample. */
const NEIGHBOURS: ReadonlyArray<readonly [number, number]> = [-1, 0, 1].flatMap((y: number) =>
  [-1, 0, 1].map((x: number) => [x, y] as const)
);

/**
 * The temporal resolve, after Karis's "High Quality Temporal Supersampling" and three's `TAAUNode` (MIT): each output
 * pixel reconstructs this frame's colour from the jittered samples around it with a Blackman-Harris window, finds its
 * surface in the history by the motion of the nearest surface around it, rejects the history where that surface stood
 * elsewhere, clips it to the neighbourhood's colours, and blends. The frame and the output are sized apart, so the same
 * resolve upscales as TAAU with the frame drawn smaller: there an output pixel takes this frame by how near the nearest
 * sample landed to its centre, scaled by the area a drawn pixel covers, so the history keeps what earlier samples
 * nearer the centre saw and holds as many frames' worth as it does unscaled.
 *
 * @param inputs - What the resolve reads.
 * @param uniforms - The uniforms it reads.
 * @returns Two outputs: the history, colour and distance, and the frame as shown, colour and coverage.
 */
export function toTemporalResolve(inputs: ITemporalInputs, uniforms: ITemporalUniforms): Node {
  const { camera, motion, temporal } = uniforms;
  const inputSize: Node<"vec2"> = vec2(texture(inputs.frame).size(int(0)) as Node<"uvec2">);
  const outputSize: Node<"vec2"> = vec2(texture(inputs.history).size(int(0)) as Node<"uvec2">);
  const lastTexel: Node<"vec2"> = inputSize.sub(1);
  // Output pixels a drawn one spans across; one unscaled.
  const upscale: Node<"float"> = outputSize.x.div(inputSize.x);

  function toTexel(at: Node<"vec2">): Node<"ivec2"> {
    return ivec2(clamp(at, vec2(0), lastTexel));
  }

  function toDepth(at: Node<"vec2">): Node<"float"> {
    return textureLoad(inputs.depth, toTexel(at)) as unknown as Node<"float">;
  }

  const resolved: Node<"vec4"> = Fn(() => {
    const uv: Node<"vec2"> = screenUV;
    const position: Node<"vec2"> = uv.mul(inputSize);
    // The sample at texel `m` shows the scene at `m + 0.5 + jitter`: the one nearest this pixel's centre.
    const nearest: Node<"vec2"> = round(position.sub(0.5).sub(motion.jitter));

    // The nearest surface around it, whose motion carries an edge's history with the edge rather than its background.
    let closestDepth: Node<"float"> = float(0);
    let closest: Node<"vec2"> = nearest;

    for (const [x, y] of NEIGHBOURS) {
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

    // Nothing drawn is the sky at the far plane, which moves with the camera's turn alone.
    const far: Node<"vec3"> = camera.viewToWorld.mul(vec4(getViewPosition(uv, float(0), camera.projectionInverse), 1))
      .xyz as Node<"vec3">;
    const farClip: Node<"vec4"> = motion.previousViewProjection.mul(vec4(far, 1));
    const farBefore: Node<"vec2"> = farClip.xy.div(farClip.w).mul(vec2(0.5, -0.5)).add(0.5);
    const moved: Node<"vec2"> = select(
      closestDepth.greaterThan(0),
      textureLoad(inputs.motion, toTexel(closest)).xy,
      uv.sub(farBefore)
    );
    const before: Node<"vec2"> = uv.sub(moved);

    // The history stands for this surface where it showed something as far from the camera as this point was then.
    const isInside: Node<"bool"> = before
      .greaterThanEqual(vec2(0))
      .all()
      .and(before.lessThanEqual(vec2(1)).all());
    const historyDistance: Node<"float"> = textureLoad(
      inputs.history,
      ivec2(clamp(floor(before.mul(outputSize)), vec2(0), outputSize.sub(1)))
    ).w;
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

    for (const [x, y] of NEIGHBOURS) {
      const at: Node<"vec2"> = nearest.add(vec2(x, y));
      const color: Node<"vec3"> = max(textureLoad(inputs.frame, toTexel(at)).xyz, vec3(0));
      const offset: Node<"vec2"> = position.sub(at.add(0.5).add(motion.jitter));
      const weight: Node<"float"> = exp(offset.dot(offset).mul(WINDOW));

      sum = sum.add(color.mul(weight));
      weights = weights.add(weight);
      moment = moment.add(color);
      momentSquared = momentSquared.add(color.mul(color));
    }

    const current: Node<"vec3"> = sum.div(max(weights, 1e-5));
    const mean: Node<"vec3"> = moment.div(NEIGHBOURS.length);
    const motionShare: Node<"float"> = saturate(length(moved.mul(outputSize)).div(MAX_MOTION));
    // Wider while still, so a sub-pixel edge keeps its history; tighter while moving, so nothing trails.
    const spread: Node<"vec3"> = sqrt(max(momentSquared.div(NEIGHBOURS.length).sub(mean.mul(mean)), vec3(0))).mul(
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

  const coverage: Node<"float"> = textureLoad(
    inputs.frame,
    toTexel(round(screenUV.mul(inputSize).sub(0.5).sub(motion.jitter)))
  ).w;

  return outputStruct(resolved, vec4(resolved.xyz, coverage));
}

/**
 * The depth the output carries for the helpers drawn over it: the drawn sample's nearest this pixel's centre.
 *
 * @param inputs - What the resolve reads.
 * @param motion - The motion uniforms, which hold this frame's jitter.
 * @returns The depth, reversed like the drawn one.
 */
export function toResolvedDepth(inputs: ITemporalInputs, motion: MotionUniforms): Node<"float"> {
  const inputSize: Node<"vec2"> = vec2(texture(inputs.frame).size(int(0)) as Node<"uvec2">);
  const nearest: Node<"vec2"> = round(screenUV.mul(inputSize).sub(0.5).sub(motion.jitter));

  return textureLoad(inputs.depth, ivec2(clamp(nearest, vec2(0), inputSize.sub(1)))) as unknown as Node<"float">;
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
