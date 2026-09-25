import {
  abs,
  atomicMax,
  atomicStore,
  bool,
  dot,
  float,
  floatBitsToUint,
  floor,
  Fn,
  If,
  instanceIndex,
  int,
  length,
  max,
  min,
  mix,
  pow,
  saturate,
  screenCoordinate,
  select,
  storage,
  texture,
  textureLoad,
  uint,
  uintBitsToFloat,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { ComputeNode, DepthTexture, Node, StorageBufferAttribute, Texture } from "three/webgpu";

import {
  FSR2_FP16_MAX,
  IFsrConstants,
  isOnScreen,
  loadClamped,
  NEIGHBOURHOOD,
  packOutputs,
  RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD,
  toClampedTexel,
  toClampedUv,
  toLogLuma,
  toMaxDistance,
  toPerceivedLuma,
  toScalarTexel,
  toViewDepth,
  toYCoCg,
  unpackOutputs,
} from "#/pass/fsr/fsr-common.tsl";

/** What FSR 2 reads of the frame as drawn. */
export interface IFsrInputs {
  /** The tonemapped frame, jittered, with the blended surfaces. */
  color: Texture;
  depth: DepthTexture;
  /** The renderer's motion: how far a pixel's surface moved since the frame before, in texture coordinates. */
  motion: Texture;
}

/** The nine texels around the centre, the centre first: `FindNearestDepth`'s order. */
const NEAREST_ORDER: ReadonlyArray<readonly [number, number]> = [
  [0, 0],
  [1, 0],
  [0, 1],
  [0, -1],
  [-1, 0],
  [-1, 1],
  [1, 1],
  [-1, -1],
  [1, -1],
];

/** `LoadInputMotionVector`: FSR's motion is the renderer's turned, from now to the frame before. */
function loadMotion(inputs: IFsrInputs, position: Node<"vec2">, constants: IFsrConstants): Node<"vec2"> {
  return loadClamped(inputs.motion, position, constants.renderSize).xy.negate();
}

function loadDepth(inputs: IFsrInputs, position: Node<"vec2">, constants: IFsrConstants): Node<"float"> {
  return textureLoad(inputs.depth, toClampedTexel(position, constants.renderSize)) as unknown as Node<"float">;
}

/** `FindNearestDepth`, inverted: the nearest of the nine is the greatest. */
function toNearestDepth(
  inputs: IFsrInputs,
  position: Node<"vec2">,
  constants: IFsrConstants
): { depth: Node<"float">; at: Node<"vec2"> } {
  let depth: Node<"float"> = loadDepth(inputs, position, constants).toVar();
  let at: Node<"vec2"> = position;

  for (const [x, y] of NEAREST_ORDER.slice(1)) {
    const sample: Node<"vec2"> = position.add(vec2(x, y));
    const sampled: Node<"float"> = loadDepth(inputs, sample, constants);
    const isNearer: Node<"bool"> = isOnScreen(sample, constants.renderSize).and(sampled.greaterThan(depth));

    depth = select(isNearer, sampled, depth).toVar();
    at = select(isNearer, sample, at).toVar();
  }

  return { at, depth };
}

/**
 * `SpdLoadSourceImage` and the reduction to the first mip: each texel the mean log luma of the 8x8 drawn texels
 * under it, read where they would stand unjittered; off-screen texels count as nothing, as SPD counts them.
 */
export function toLumaEighth(inputs: IFsrInputs, constants: IFsrConstants): Node<"vec4"> {
  return Fn(() => {
    const base: Node<"vec2"> = screenCoordinate.xy.floor().mul(8);
    let sum: Node<"float"> = float(0);

    for (let y: number = 0; y < 8; y += 1) {
      for (let x: number = 0; x < 8; x += 1) {
        const at: Node<"vec2"> = base.add(vec2(x, y));
        const uv: Node<"vec2"> = toClampedUv(
          at.add(0.5).add(constants.jitter).div(constants.renderSize),
          constants.renderSize
        );
        const luma: Node<"float"> = toLogLuma(texture(inputs.color, uv).level(int(0)).xyz);

        sum = sum.add(select(at.lessThan(constants.renderSize).all(), luma, float(0)));
      }
    }

    return toScalarTexel(sum.div(64));
  })();
}

/** The reduction on to the shading change mip: each texel the mean of the 4x4 first-mip texels under it. */
export function toLumaShadingChange(eighth: Texture): Node<"vec4"> {
  return Fn(() => {
    const size: Node<"vec2"> = vec2(texture(eighth).size(int(0)) as Node<"uvec2">);
    const base: Node<"vec2"> = screenCoordinate.xy.floor().mul(4);
    let sum: Node<"float"> = float(0);

    for (let y: number = 0; y < 4; y += 1) {
      for (let x: number = 0; x < 4; x += 1) {
        const at: Node<"vec2"> = base.add(vec2(x, y));

        sum = sum.add(select(at.lessThan(size).all(), loadClamped(eighth, at, size).x, float(0)));
      }
    }

    return toScalarTexel(sum.div(16));
  })();
}

/**
 * `ClearResourcesForNextFrame`, run before the reconstruction rather than after the lock: every texel of the
 * reconstructed depth at the far plane, zero inverted.
 */
export function createFsrDepthClear(buffer: StorageBufferAttribute, capacity: number): ComputeNode {
  const depths = storage(buffer, "uint", capacity).toAtomic();

  return Fn(() => {
    atomicStore(depths.element(instanceIndex), uint(0));
  })().compute(capacity);
}

/**
 * `ReconstructPrevDepth`: each drawn texel's nearest depth pushed to where its surface stood the frame before, to every
 * texel its bilinear footprint covers there, the nearest kept by an atomic maximum of the depth's bits.
 */
export function createFsrDepthReconstruction(
  inputs: IFsrInputs,
  buffer: StorageBufferAttribute,
  capacity: number,
  constants: IFsrConstants
): ComputeNode {
  const depths = storage(buffer, "uint", capacity).toAtomic();

  return Fn(() => {
    const width: Node<"uint"> = uint(constants.renderSize.x);
    const position: Node<"vec2"> = vec2(float(instanceIndex.mod(width)), float(instanceIndex.div(width))).toVar();
    const nearest = toNearestDepth(inputs, position, constants);
    const motion: Node<"vec2"> = loadMotion(inputs, nearest.at, constants).toVar();
    // Motion under a tenth of a display pixel is taken for none.
    const moved: Node<"vec2"> = motion.mul(float(length(motion.mul(constants.displaySize)).greaterThan(0.1)));
    const sample: Node<"vec2"> = position
      .add(0.5)
      .div(constants.renderSize)
      .add(moved)
      .mul(constants.renderSize)
      .sub(0.5);
    const base: Node<"vec2"> = floor(sample).toVar();
    const fraction: Node<"vec2"> = sample.sub(base).toVar();
    const bits: Node<"uint"> = (floatBitsToUint(nearest.depth) as unknown as Node<"uint">).toVar();
    const corners: ReadonlyArray<readonly [number, number, Node<"float">]> = [
      [0, 0, fraction.x.oneMinus().mul(fraction.y.oneMinus())],
      [1, 0, fraction.x.mul(fraction.y.oneMinus())],
      [0, 1, fraction.x.oneMinus().mul(fraction.y)],
      [1, 1, fraction.x.mul(fraction.y)],
    ];

    for (const [x, y, weight] of corners) {
      const at: Node<"vec2"> = base.add(vec2(x, y));

      If(weight.greaterThan(RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD).and(isOnScreen(at, constants.renderSize)), () => {
        atomicMax(depths.element(uint(at.y).mul(width).add(uint(at.x))), bits);
      });
    }
  })().compute(capacity);
}

/**
 * `ReconstructAndDilate` less the scatter: the nearest depth of the nine, the motion found there, and the luma the locks
 * read.
 *
 * @returns Three outputs: the dilated depth, the dilated motion, and the lock luma.
 */
export function toFsrDilate(inputs: IFsrInputs, constants: IFsrConstants): Node {
  const packed: Node<"mat4"> = Fn(() => {
    const position: Node<"vec2"> = screenCoordinate.xy.floor();
    const nearest = toNearestDepth(inputs, position, constants);
    const motion: Node<"vec2"> = loadMotion(inputs, nearest.at, constants);
    // `ComputeLockInputLuma`: an exposure of one, the colour already in the display's range.
    const color: Node<"vec3"> = max(loadClamped(inputs.color, position, constants.renderSize).xyz, vec3(0));
    const lockLuma: Node<"float"> = pow(max(toPerceivedLuma(color), float(0)), float(1 / 6));

    return packOutputs(toScalarTexel(nearest.depth), vec4(motion, 0, 1), toScalarTexel(lockLuma));
  })() as unknown as Node<"mat4">;

  return unpackOutputs(packed, 3);
}

/**
 * The reactive mask, as FSR 2's generate-reactive pass makes it from the frame before and after the blended surfaces:
 * the greatest change of a channel, both tonemapped, past a threshold a fixed reactivity.
 */
export function toFsrReactive(opaque: Texture, inputs: IFsrInputs, constants: IFsrConstants): Node<"vec4"> {
  return Fn(() => {
    const position: Node<"vec2"> = screenCoordinate.xy.floor();

    function tonemap(rgb: Node<"vec3">): Node<"vec3"> {
      return rgb.div(max(max(rgb.x, max(rgb.y, rgb.z)), float(0)).add(1));
    }

    const before: Node<"vec3"> = tonemap(loadClamped(opaque, position, constants.renderSize).xyz);
    const after: Node<"vec3"> = tonemap(loadClamped(inputs.color, position, constants.renderSize).xyz);
    const delta: Node<"vec3"> = abs(after.sub(before));
    const reactive: Node<"float"> = max(delta.x, max(delta.y, delta.z));

    return toScalarTexel(select(reactive.lessThan(REACTIVE_THRESHOLD), float(0), float(REACTIVE_VALUE)));
  })();
}

/** Where a change counts as reactive, and how reactive it counts: FSR 2's sample's generate-reactive settings. */
const REACTIVE_THRESHOLD: number = 0.2;
const REACTIVE_VALUE: number = 0.9;

/** What the depth clip reads besides the frame. */
export interface IFsrClipInputs {
  /** The reconstructed depth of the frame before. */
  reconstructed: StorageBufferAttribute;
  capacity: number;
  dilatedDepth: Texture;
  dilatedMotion: Texture;
  previousDilatedMotion: Texture;
  reactive: Texture;
}

/**
 * `DepthClip`: the prepared colour, in YCoCg with how far its history is to be distrusted for a surface now uncovered,
 * and the dilated reactive masks, how reactive and how far from its history the motion leaves each texel.
 *
 * @returns Two outputs: the prepared colour and depth clip, and the reactive and accumulation masks.
 */
export function toFsrDepthClip(inputs: IFsrInputs, clip: IFsrClipInputs, constants: IFsrConstants): Node {
  const reconstructed = storage(clip.reconstructed, "uint", clip.capacity).toReadOnly();
  const packed: Node<"mat4"> = Fn(() => {
    const { renderSize, displaySize } = constants;
    const position: Node<"vec2"> = screenCoordinate.xy.floor();
    const width: Node<"uint"> = uint(renderSize.x);

    function loadReconstructed(at: Node<"vec2">): Node<"float"> {
      const texel = toClampedTexel(at, renderSize);

      return uintBitsToFloat(
        reconstructed.element(uint(texel.y).mul(width).add(uint(texel.x)))
      ) as unknown as Node<"float">;
    }

    function dilatedDepth(at: Node<"vec2">): Node<"float"> {
      return loadClamped(clip.dilatedDepth, at, renderSize).x;
    }

    function dilatedMotion(at: Node<"vec2">): Node<"vec2"> {
      return loadClamped(clip.dilatedMotion, at, renderSize).xy;
    }

    // `GetViewSpacePosition`, for the corner and the centre `ComputeDepthClip` compares.
    function toViewPosition(at: Node<"vec2">, depth: Node<"float">): Node<"vec3"> {
      const z: Node<"float"> = toViewDepth(depth, constants);
      const ndc: Node<"vec2"> = at.div(renderSize).mul(vec2(2, -2)).add(vec2(-1, 1));

      return vec3(constants.deviceToView.z.mul(ndc.x).mul(z), constants.deviceToView.w.mul(ndc.y).mul(z), z);
    }

    const motion: Node<"vec2"> = dilatedMotion(position).toVar();
    // Motion under a hundredth of a display pixel is taken for none.
    const moved: Node<"vec2"> = motion.mul(float(length(motion.mul(displaySize)).greaterThan(0.01)));
    const dilatedUv: Node<"vec2"> = position.add(0.5).div(renderSize).add(moved);
    const current: Node<"float"> = dilatedDepth(position).toVar();

    // `ComputeDepthClip`: how far the surface stands behind what stood there the frame before, against the separation
    // the view's resolution and field can tell.
    const currentView: Node<"float"> = toViewDepth(current, constants).toVar();
    const sample: Node<"vec2"> = dilatedUv.mul(renderSize).sub(0.5);
    const base: Node<"vec2"> = floor(sample).toVar();
    const fraction: Node<"vec2"> = sample.sub(base).toVar();
    const halfViewport: Node<"float"> = length(renderSize);
    const resolution: Node<"float"> = saturate(halfViewport.div(Math.hypot(1920, 1080)));
    const power: Node<"float"> = mix(float(1), float(3), resolution);
    let clipped: Node<"float"> = float(0);
    let weights: Node<"float"> = float(0);

    for (const [x, y, weight] of [
      [0, 0, fraction.x.oneMinus().mul(fraction.y.oneMinus())],
      [1, 0, fraction.x.mul(fraction.y.oneMinus())],
      [0, 1, fraction.x.oneMinus().mul(fraction.y)],
      [1, 1, fraction.x.mul(fraction.y)],
    ] as const) {
      const at: Node<"vec2"> = base.add(vec2(x, y));
      const previous: Node<"float"> = loadReconstructed(at).toVar();
      const previousView: Node<"float"> = toViewDepth(previous, constants).toVar();
      const difference: Node<"float"> = currentView.sub(previousView);
      const isCounted: Node<"bool"> = isOnScreen(at, renderSize)
        .and((weight as Node<"float">).greaterThan(RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD))
        .and(difference.greaterThan(0));
      const plane: Node<"float"> = min(previous, current);
      const centre: Node<"vec3"> = toViewPosition(floor(renderSize.mul(0.5)), plane);
      const corner: Node<"vec3"> = toViewPosition(vec2(0), plane);
      const threshold: Node<"float"> = max(currentView, previousView);
      const required: Node<"float"> = float(1.37e-5)
        .mul(length(corner).div(length(centre)))
        .mul(halfViewport)
        .mul(threshold);
      const share: Node<"float"> = pow(saturate(required.div(max(difference, 1e-10))), power).mul(
        weight as Node<"float">
      );

      clipped = clipped.add(select(isCounted, share, float(0)));
      weights = weights.add(select(isCounted, weight as Node<"float">, float(0)));
    }

    const depthClip: Node<"float"> = select(
      weights.greaterThan(0),
      saturate(clipped.div(max(weights, 1e-10)).oneMinus()),
      float(0)
    );
    // `EvaluateSurface`: a surface sloping away down the view counts for nothing.
    const d0: Node<"float"> = toViewDepth(loadReconstructed(position.add(vec2(0, -1))), constants).toVar();
    const d1: Node<"float"> = toViewDepth(loadReconstructed(position), constants).toVar();
    const d2: Node<"float"> = toViewDepth(loadReconstructed(position.add(vec2(0, 1))), constants).toVar();
    const surface: Node<"float"> = float(
      d0
        .sub(d1)
        .greaterThan(d1.mul(0.01))
        .and(d1.sub(d2).greaterThan(d2.mul(0.01)))
    ).oneMinus();

    // `ComputePreparedInputColor`: an exposure of one.
    const rgb: Node<"vec3"> = min(
      max(loadClamped(inputs.color, position, renderSize).xyz, vec3(0)),
      vec3(FSR2_FP16_MAX)
    );
    const prepared: Node<"vec4"> = vec4(toYCoCg(rgb), depthClip.mul(surface));

    // `ComputeMotionDivergence`: how far the motion around turns from this texel's.
    const nucleus: Node<"vec2"> = loadMotion(inputs, position, constants).toVar();
    let maxVelocity: Node<"float"> = length(nucleus).toVar();
    let convergence: Node<"float"> = float(1);

    for (const [x, y] of NEIGHBOURHOOD) {
      const around: Node<"vec2"> = loadMotion(inputs, position.add(vec2(x, y)), constants).toVar();
      const velocity: Node<"float"> = length(around);

      maxVelocity = max(velocity, maxVelocity).toVar();

      const scale: Node<"float"> = max(max(velocity, maxVelocity), 1e-10);

      convergence = min(convergence, dot(around.div(scale), nucleus.div(scale)));
    }

    const isMoving: Node<"bool"> = length(nucleus.mul(renderSize)).greaterThan(0.01);
    const motionDivergence: Node<"float"> = select(
      isMoving,
      saturate(convergence.oneMinus()).mul(saturate(maxVelocity.div(0.01))),
      float(0)
    );

    // `ComputeTemporalMotionDivergence`: how far this motion departs from the motion found where it came from.
    const reprojected: Node<"vec2"> = toClampedUv(position.add(0.5).div(renderSize).add(motion), renderSize);
    const previousMotion: Node<"vec2"> = texture(clip.previousDilatedMotion, reprojected).level(int(0)).xy;
    const distance: Node<"float"> = length(motion.mul(displaySize)).toVar();
    const temporalDivergence: Node<"float"> = select(
      distance.greaterThan(1),
      mix(
        float(0),
        saturate(length(previousMotion).div(max(length(motion), 1e-10))).oneMinus(),
        saturate(pow(distance.div(20), float(3)))
      ),
      float(0)
    );

    // `ComputeDepthDivergence`: how far the depths around spread, none where the sky shows.
    const farthest: Node<"float"> = toMaxDistance(constants).toVar();
    let depthMin: Node<"float"> = farthest;
    let depthMax: Node<"float"> = float(0);
    let isSky: Node<"bool"> = bool(false);

    for (const [x, y] of NEIGHBOURHOOD) {
      const at: Node<"vec2"> = position.add(vec2(x, y));
      const depth: Node<"float"> = toViewDepth(dilatedDepth(at), constants)
        .mul(float(isOnScreen(at, renderSize)))
        .toVar();

      isSky = isSky.or(depth.equal(farthest));
      depthMin = min(depthMin, depth);
      depthMax = max(depthMax, depth);
    }

    const depthDivergence: Node<"float"> = select(isSky, float(0), depthMin.div(max(depthMax, 1e-10)).oneMinus());
    const accumulationMask: Node<"float"> = max(saturate(temporalDivergence.sub(depthDivergence)), motionDivergence);

    // `PreProcessReactiveMasks`: the reactive mask dilated to the similar colours around, the more similar the more.
    const reference: Node<"vec3"> = loadClamped(inputs.color, position, renderSize).xyz.toVar();
    let reactive: Node<"float"> = float(0);

    for (const [x, y] of NEIGHBOURHOOD) {
      const at: Node<"vec2"> = position.add(vec2(x, y));
      const color: Node<"vec3"> = loadClamped(inputs.color, at, renderSize).xyz;
      const mask: Node<"float"> = loadClamped(clip.reactive, at, renderSize).x;
      const similarity: Node<"float"> = dot(reference, color).div(
        max(max(dot(reference, reference), dot(color, color)), 1e-10)
      );
      const sharpness: Node<"float"> = float(1).add(float(6).sub(similarity.mul(6)));

      reactive = max(reactive, pow(max(mask, 0), sharpness));
    }

    return packOutputs(prepared, vec4(reactive, accumulationMask, 0, 1));
  })() as unknown as Node<"mat4">;

  return unpackOutputs(packed, 2);
}

/**
 * `ComputeThinFeatureConfidence`: a texel whose luma stands above or below all it differs from around it, and that no
 * quadrant of similar texels surrounds, is a thin feature to lock.
 */
export function toFsrLock(lockLuma: Texture, constants: IFsrConstants): Node<"vec4"> {
  return Fn(() => {
    const position: Node<"vec2"> = screenCoordinate.xy.floor();
    const nucleus: Node<"float"> = loadClamped(lockLuma, position, constants.renderSize).x.toVar();
    // Row by row, as the rejection masks number them; the centre is similar to itself.
    const similar: Array<Node<"bool">> = [];
    let dissimilarMin: Node<"float"> = float(3.402823466e38);
    let dissimilarMax: Node<"float"> = float(0);

    NEIGHBOURHOOD.forEach(([x, y]) => {
      if (x === 0 && y === 0) {
        similar.push(bool(true));

        return;
      }

      const luma: Node<"float"> = loadClamped(lockLuma, position.add(vec2(x, y)), constants.renderSize).x.toVar();
      const difference: Node<"float"> = max(luma, nucleus).div(min(luma, nucleus));
      const isSimilar: Node<"bool"> = difference.greaterThan(0).and(difference.lessThan(1.05)).toVar();

      similar.push(isSimilar);
      dissimilarMin = select(isSimilar, dissimilarMin, min(dissimilarMin, luma));
      dissimilarMax = select(isSimilar, dissimilarMax, max(dissimilarMax, luma));
    });

    const isRidge: Node<"bool"> = nucleus.greaterThan(dissimilarMax).or(nucleus.lessThan(dissimilarMin));
    const quadrants: ReadonlyArray<ReadonlyArray<number>> = [
      [0, 1, 3, 4],
      [1, 2, 4, 5],
      [3, 4, 6, 7],
      [4, 5, 7, 8],
    ];
    let isSurrounded: Node<"bool"> = bool(false);

    for (const quadrant of quadrants) {
      isSurrounded = isSurrounded.or(
        similar[quadrant[0]].and(similar[quadrant[1]]).and(similar[quadrant[2]]).and(similar[quadrant[3]])
      );
    }

    return toScalarTexel(float(isRidge.and(isSurrounded.not())));
  })();
}
