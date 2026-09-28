import {
  bool,
  dot,
  float,
  floor,
  Fn,
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
  uint,
  uintBitsToFloat,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node } from "three/webgpu";

import { IFsrClipInputs } from "#/pass/fsr/fsr-clip-inputs";
import {
  FSR2_FP16_MAX,
  isOnScreen,
  loadFsrMotion,
  RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD,
  toClampedUv,
  toMaxDistance,
  toViewDepth,
  toYCoCg,
} from "#/pass/fsr/fsr-common.tsl";
import { IFsrConstants } from "#/pass/fsr/fsr-constants";
import { IFsrInputs } from "#/pass/fsr/fsr-inputs";
import { IBilinearFootprint } from "#/shader/bilinear-footprint";
import { packOutputs, unpackOutputs } from "#/shader/packed-outputs.tsl";
import { loadClamped, NEIGHBOURHOOD, toBilinearFootprint, toClampedTexel } from "#/shader/texel.tsl";

// `ffx_fsr2_depth_clip.h`.

/** `Ksep`: the depth separation a pixel of the view can tell, per unit of distance. */
const DEPTH_SEPARATION: number = 1.37e-5;

/**
 * `DepthClip`.
 *
 * @param inputs - What FSR 2 reads of the frame.
 * @param clip - What it reads besides.
 * @param constants - The frame's FSR constants.
 * @returns Two outputs: the prepared colour in YCoCg with its depth clip, and the reactive and accumulation masks.
 */
export function toFsrDepthClip(inputs: IFsrInputs, clip: IFsrClipInputs, constants: IFsrConstants): Node {
  const reconstructed = storage(clip.reconstructed, "uint", clip.capacity).toReadOnly();
  const packed: Node<"mat4"> = Fn(() => {
    const { renderSize, displaySize } = constants;
    const position: Node<"vec2"> = screenCoordinate.xy.floor();
    const width: Node<"uint"> = uint(renderSize.x);

    function loadReconstructed(at: Node<"vec2">): Node<"float"> {
      const texel: Node<"ivec2"> = toClampedTexel(at, renderSize);

      return uintBitsToFloat(
        reconstructed.element(uint(texel.y).mul(width).add(uint(texel.x)))
      ) as unknown as Node<"float">;
    }

    function dilatedDepth(at: Node<"vec2">): Node<"float"> {
      return loadClamped(clip.dilatedDepth, at, renderSize).x;
    }

    // `GetViewSpacePosition`.
    function toViewPosition(at: Node<"vec2">, depth: Node<"float">): Node<"vec3"> {
      const z: Node<"float"> = toViewDepth(depth, constants);
      const ndc: Node<"vec2"> = at.div(renderSize).mul(vec2(2, -2)).add(vec2(-1, 1));

      return vec3(constants.deviceToView.z.mul(ndc.x).mul(z), constants.deviceToView.w.mul(ndc.y).mul(z), z);
    }

    const motion: Node<"vec2"> = loadClamped(clip.dilatedMotion, position, renderSize).xy.toVar();
    // Motion under a hundredth of a display pixel is taken for none.
    const moved: Node<"vec2"> = motion.mul(float(length(motion.mul(displaySize)).greaterThan(0.01)));
    const current: Node<"float"> = dilatedDepth(position).toVar();

    // `ComputeDepthClip`: how far the surface stands behind what stood there the frame before, against the separation
    // the view's resolution and field can tell.
    const currentView: Node<"float"> = toViewDepth(current, constants).toVar();
    const footprint: IBilinearFootprint = toBilinearFootprint(position.add(0.5).div(renderSize).add(moved), renderSize);
    const halfViewport: Node<"float"> = length(renderSize);
    const power: Node<"float"> = mix(float(1), float(3), saturate(halfViewport.div(Math.hypot(1920, 1080))));
    let clipped: Node<"float"> = float(0);
    let weights: Node<"float"> = float(0);

    for (const { offset, weight } of footprint.corners) {
      const at: Node<"vec2"> = footprint.base.add(vec2(...offset));
      const previous: Node<"float"> = loadReconstructed(at).toVar();
      const previousView: Node<"float"> = toViewDepth(previous, constants).toVar();
      const difference: Node<"float"> = currentView.sub(previousView);
      const isCounted: Node<"bool"> = isOnScreen(at, renderSize)
        .and(weight.greaterThan(RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD))
        .and(difference.greaterThan(0));
      const plane: Node<"float"> = min(previous, current);
      const centre: Node<"vec3"> = toViewPosition(floor(renderSize.mul(0.5)), plane);
      const corner: Node<"vec3"> = toViewPosition(vec2(0), plane);
      const required: Node<"float"> = float(DEPTH_SEPARATION)
        .mul(length(corner).div(length(centre)))
        .mul(halfViewport)
        .mul(max(currentView, previousView));
      const share: Node<"float"> = pow(saturate(required.div(max(difference, 1e-10))), power).mul(weight);

      clipped = clipped.add(select(isCounted, share, float(0)));
      weights = weights.add(select(isCounted, weight, float(0)));
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
    const nucleus: Node<"vec2"> = loadFsrMotion(inputs, position, constants).toVar();
    let maxVelocity: Node<"float"> = length(nucleus).toVar();
    let convergence: Node<"float"> = float(1);

    for (const [x, y] of NEIGHBOURHOOD) {
      const around: Node<"vec2"> = loadFsrMotion(inputs, position.add(vec2(x, y)), constants).toVar();
      const velocity: Node<"float"> = length(around);

      maxVelocity = max(velocity, maxVelocity).toVar();

      const scale: Node<"float"> = max(max(velocity, maxVelocity), 1e-10);

      convergence = min(convergence, dot(around.div(scale), nucleus.div(scale)));
    }

    const motionDivergence: Node<"float"> = select(
      length(nucleus.mul(renderSize)).greaterThan(0.01),
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
