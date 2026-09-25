import {
  abs,
  bool,
  clamp,
  exp,
  float,
  floor,
  Fn,
  If,
  int,
  length,
  max,
  min,
  mix,
  pow,
  round,
  saturate,
  screenCoordinate,
  select,
  sign,
  sqrt,
  texture,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node, Texture } from "three/webgpu";

import {
  AVERAGE_LANCZOS_WEIGHT_PER_FRAME,
  FSR2_EPSILON,
  FSR2_FP16_MAX,
  IFsrConstants,
  isOnScreen,
  loadClamped,
  MAX_ACCUMULATION_LANCZOS_WEIGHT,
  packOutputs,
  toClampedUv,
  toDisplayPosition,
  toLanczos2ApproxSq,
  toLanczos2Sample,
  toRgb,
  toYCoCg,
  unpackOutputs,
  UPSAMPLE_LANCZOS_WEIGHT_SCALE,
} from "#/pass/fsr/fsr-common.tsl";
import { toUpscaledCoverage } from "#/pass/upscale-depth.tsl";

/** What the accumulation reads: this frame's prepared inputs at the render size, and the history at the display's. */
export interface IFsrAccumulateInputs {
  /** The frame as drawn, whose coverage the output keeps. */
  frame: Texture;
  /** YCoCg, and the depth clip in alpha. */
  prepared: Texture;
  /** The reactive and accumulation masks. */
  reactiveMasks: Texture;
  dilatedMotion: Texture;
  /** Whether each drawn texel is a thin feature to lock. */
  locks: Texture;
  /** The mean log luma a 32nd a side. */
  shadingLuma: Texture;
  /** The frame before, resolved: colour, and the temporal reactivity, negative where it was in motion. */
  history: Texture;
  /** Each pixel's lock: its lifetime remaining, and the shading luma it was locked at. */
  lockStatus: Texture;
  /** Each pixel's four last lumas. */
  lumaHistory: Texture;
}

/** `RectificationBox`. */
interface IRectificationBox {
  centre: Node<"vec3">;
  spread: Node<"vec3">;
  least: Node<"vec3">;
  most: Node<"vec3">;
}

/**
 * `Accumulate` (`ffx_fsr2_accumulate.h`, with `ffx_fsr2_reproject.h`, `ffx_fsr2_upsample.h` and
 * `ffx_fsr2_postprocess_lock_status.h`): each display pixel's history reprojected, this frame's samples around it
 * gathered through a Lanczos kernel sized by how much the history can be trusted, the history rectified to their box
 * unless a lock or the luma's instability holds it, and the two blended by how much this frame adds.
 *
 * @param inputs - What it reads.
 * @param constants - The frame's FSR constants.
 * @param jitter - The renderer's own jitter, for the coverage the output keeps.
 * @returns Four outputs: the history, the lock status, the luma history, and the output with its coverage.
 */
export function toFsrAccumulate(inputs: IFsrAccumulateInputs, constants: IFsrConstants, jitter: Node<"vec2">): Node {
  const packed: Node<"mat4"> = Fn(() => {
    const { renderSize, displaySize, downscale } = constants;
    const position: Node<"vec2"> = screenCoordinate.xy.floor();

    // `InitParams`.
    const uv: Node<"vec2"> = position.add(0.5).div(displaySize).toVar();
    const drawnUv: Node<"vec2"> = toClampedUv(uv.add(constants.jitter.div(renderSize)), renderSize).toVar();
    const motion: Node<"vec2"> = loadClamped(inputs.dilatedMotion, floor(uv.mul(renderSize)), renderSize).xy.toVar();
    const velocity: Node<"float"> = length(motion.mul(displaySize)).toVar();
    const reprojected: Node<"vec2"> = uv.add(motion).toVar();
    const isExisting: Node<"bool"> = reprojected
      .greaterThanEqual(vec2(0))
      .all()
      .and(reprojected.lessThanEqual(vec2(1)).all())
      .toVar();
    const depthClip: Node<"float"> = saturate(texture(inputs.prepared, drawnUv).level(int(0)).w).toVar();
    const masks: Node<"vec4"> = texture(inputs.reactiveMasks, drawnUv).level(int(0)).toVar();
    const dilatedReactive: Node<"float"> = masks.x;
    const accumulationMask: Node<"float"> = masks.y;
    const isReset: Node<"bool"> = constants.frameIndex.lessThan(0.5);
    const isNew: Node<"bool"> = isExisting.not().or(isReset).toVar();

    // `ReprojectHistoryColor` and `ReprojectHistoryLockStatus`.
    const history: Node<"vec3"> = vec3(0).toVar();
    const lockStatus: Node<"vec2"> = vec2(0).toVar();
    const temporalReactive: Node<"float"> = float(0).toVar();
    const wasMoving: Node<"bool"> = bool(false).toVar();
    const isNewLock: Node<"bool"> = bool(false).toVar();

    If(isExisting.and(isReset.not()), () => {
      const previous: Node<"vec4"> = toLanczos2Sample(inputs.history, reprojected, displaySize).toVar();

      history.assign(toYCoCg(clamp(previous.xyz, vec3(0), vec3(FSR2_FP16_MAX))));
      temporalReactive.assign(saturate(abs(previous.w)));
      wasMoving.assign(previous.w.lessThan(0));
      isNewLock.assign(toNewLock(inputs.locks, position, constants).greaterThan(127 / 255));
      lockStatus.assign(texture(inputs.lockStatus, reprojected).level(int(0)).xy);
    });

    const reactive: Node<"float"> = max(dilatedReactive, temporalReactive).toVar();

    // `UpdateLockStatus`.
    const mipSize: Node<"vec2"> = floor(renderSize.div(2 << 4));
    const shadingUv: Node<"vec2"> = toClampedUv(uv, mipSize).mul(mipSize).div(constants.lumaMipSize);
    const shading: Node<"float"> = pow(
      exp(texture(inputs.shadingLuma, shadingUv).level(int(0)).x),
      float(1 / 6)
    ).toVar();

    lockStatus.y.assign(select(lockStatus.y.equal(0), shading, lockStatus.y));

    const luminanceDiff: Node<"float"> = toMinOverMax(lockStatus.y, shading).oneMinus().toVar();

    If(isNewLock, () => {
      lockStatus.y.assign(shading);
      lockStatus.x.assign(select(lockStatus.x.notEqual(0), float(2), float(1)));
    })
      .ElseIf(lockStatus.x.lessThanEqual(1), () => {
        lockStatus.y.assign(mix(lockStatus.y, shading, 0.5));
      })
      .ElseIf(luminanceDiff.greaterThan(0.1), () => {
        lockStatus.x.assign(0);
      });

    reactive.assign(max(reactive, saturate(luminanceDiff.sub(0.1).mul(10))));
    lockStatus.x.assign(
      lockStatus.x
        .mul(reactive.oneMinus())
        .mul(saturate(accumulationMask.oneMinus()))
        .mul(float(depthClip.lessThan(0.1)))
    );

    const lockContribution: Node<"float"> = saturate(
      saturate(saturate(lockStatus.x.sub(1)).mul(4)).mul(saturate(toMinOverMax(lockStatus.y, shading)))
    ).toVar();

    // `ComputeUpsampledColorAndWeight`.
    const upsampled = toUpsampled(inputs.prepared, position, constants, {
      depthClip,
      isNew,
      reactive,
      velocity,
    });
    const box: IRectificationBox = upsampled.box;
    const weight: Node<"float"> = upsampled.weight;

    // `ComputeLumaInstabilityFactor`.
    const frameLuma: Node<"float"> = round(box.centre.x.mul(255)).div(255).toVar();
    const isLumaSampled: Node<"bool"> = max(max(depthClip, accumulationMask), luminanceDiff)
      .lessThan(0.1)
      .and(isNew.not());
    const lumaHistory: Node<"vec4"> = select(
      isLumaSampled,
      texture(inputs.lumaHistory, reprojected).level(int(0)),
      vec4(0)
    ).toVar();
    const firstDiff: Node<"float"> = frameLuma.sub(lumaHistory.x).toVar();
    let smallest: Node<"float"> = abs(firstDiff);

    for (const channel of ["y", "z", "w"] as const) {
      const diff: Node<"float"> = frameLuma.sub(lumaHistory[channel]);

      smallest = select(sign(firstDiff).equal(sign(diff)), min(smallest, abs(diff)), smallest);
    }

    const boxSizeFactor: Node<"float"> = pow(saturate(box.spread.x.div(0.1)), float(6));
    const isUnstable: Node<"float"> = float(
      float(smallest.notEqual(abs(firstDiff)))
        .mul(boxSizeFactor)
        .greaterThan(1 / 255)
    ).mul(max(accumulationMask, pow(max(reactive, 0), float(1 / 6))).oneMinus());
    const instability: Node<"float"> = select(abs(firstDiff).greaterThanEqual(1 / 255), isUnstable, float(0))
      // Nothing yet, until four lumas stand in the history.
      .mul(float(lumaHistory.z.notEqual(0)))
      .toVar();
    const nextLumaHistory: Node<"vec4"> = vec4(frameLuma, lumaHistory.x, lumaHistory.y, lumaHistory.z);

    // `ComputeBaseAccumulationWeight`.
    const baseAccumulation: Node<"float"> = float(MAX_ACCUMULATION_LANCZOS_WEIGHT)
      .mul(float(isExisting))
      .mul(reactive.oneMinus())
      .mul(depthClip.oneMinus())
      .toVar();

    baseAccumulation.assign(
      min(baseAccumulation, mix(baseAccumulation, weight.mul(10), max(float(wasMoving), saturate(velocity.mul(10)))))
    );
    baseAccumulation.assign(min(baseAccumulation, mix(baseAccumulation, weight, saturate(velocity.div(20)))));

    // FSR holds it as `.xxx`: one weight for every channel.
    const accumulation: Node<"float"> = baseAccumulation;
    const resolved: Node<"vec3"> = vec3(0).toVar();

    If(isNew, () => {
      resolved.assign(toRgb(upsampled.color));
    }).Else(() => {
      // `RectifyHistory`.
      const influence: Node<"float"> = min(float(20), pow(float(1).div(downscale.x.mul(downscale.y)), float(3)));
      const boxScaleT: Node<"float"> = max(depthClip, max(accumulationMask, saturate(velocity.div(20))));
      const scaledSpread: Node<"vec3"> = box.spread.mul(mix(influence, float(1), boxScaleT));
      const boxMin: Node<"vec3"> = max(box.least, box.centre.sub(scaledSpread)).toVar();
      const boxMax: Node<"vec3"> = min(box.most, box.centre.add(scaledSpread)).toVar();

      If(boxMin.greaterThan(history).any().or(history.greaterThan(boxMax).any()), () => {
        const contribution: Node<"float"> = saturate(
          max(instability, lockContribution).mul(sqrt(max(dilatedReactive, 0)).oneMinus())
        ).toVar();

        history.assign(mix(clamp(history, boxMin, boxMax), history, contribution));
        accumulation.assign(mix(min(accumulation, float(0.1)), accumulation, contribution));
      });

      // `Accumulate`, for colour in the display's range: no tonemap around the blend.
      const total: Node<"float"> = max(float(FSR2_EPSILON), accumulation.add(weight));

      resolved.assign(toRgb(mix(history, upsampled.color, weight.div(total))));
    });

    // `FinalizeLockStatus`: a lock whose surface leaves the screen next frame dies; else it wears down by this frame's
    // weight.
    const next: Node<"vec2"> = uv.sub(motion);
    const isStaying: Node<"bool"> = next
      .greaterThanEqual(vec2(0))
      .all()
      .and(next.lessThanEqual(vec2(1)).all());
    const decrease: Node<"float"> = weight.div(constants.jitterPhaseCount.mul(AVERAGE_LANCZOS_WEIGHT_PER_FRAME));
    const lifetime: Node<"float"> = select(isStaying, max(float(0), lockStatus.x.sub(decrease)), float(0));

    // `ComputeTemporalReactiveFactor`: negative where the pixel moved enough to be counted as in motion.
    const settled: Node<"float"> = min(float(0.99), reactive).toVar();

    settled.assign(max(settled, mix(settled, float(0.4), saturate(velocity))));
    settled.assign(max(settled.mul(settled), max(depthClip.mul(0.1), dilatedReactive)));
    settled.assign(select(isNew, float(1), settled));

    const nextReactive: Node<"float"> = select(
      saturate(velocity.mul(10)).greaterThanEqual(1),
      max(float(FSR2_EPSILON), settled).negate(),
      settled
    );

    return packOutputs(
      vec4(resolved, nextReactive),
      vec4(lifetime, lockStatus.y, 0, 1),
      nextLumaHistory,
      vec4(resolved, toUpscaledCoverage(inputs.frame, jitter))
    );
  })() as unknown as Node<"mat4">;

  return unpackOutputs(packed, 4);
}

/** `MinDividedByMax`. */
function toMinOverMax(a: Node<"float">, b: Node<"float">): Node<"float"> {
  const most: Node<"float"> = max(a, b);

  return select(most.notEqual(0), min(a, b).div(max(most, 1e-20)), float(0));
}

/**
 * `LoadRwNewLocks`, gathered: the lock the lock pass found for the one drawn sample landing in this display pixel, the
 * texel `m` whose `floor((m + 0.5 - jitter) / render * display)` is this pixel.
 */
function toNewLock(locks: Texture, position: Node<"vec2">, constants: IFsrConstants): Node<"float"> {
  const { renderSize, downscale } = constants;
  // The candidate at or just past this pixel's own edge, and the one before it, lest rounding put it a texel over.
  const candidate: Node<"vec2"> = position.mul(downscale).sub(0.5).add(constants.jitter).ceil().toVar();
  let lock: Node<"float"> = float(0);

  for (const [x, y] of [
    [0, 0],
    [-1, 0],
    [0, -1],
    [-1, -1],
  ] as const) {
    const at: Node<"vec2"> = candidate.add(vec2(x, y));
    const isLanding: Node<"bool"> = toDisplayPosition(at, constants)
      .equal(position)
      .all()
      .and(isOnScreen(at, renderSize));

    lock = select(isLanding, max(lock, loadClamped(locks, at, renderSize).x), lock);
  }

  return lock;
}

/** What the upsample's kernel is sized by. */
interface IUpsampleFactors {
  depthClip: Node<"float">;
  isNew: Node<"bool">;
  reactive: Node<"float">;
  velocity: Node<"float">;
}

/**
 * `ComputeUpsampledColorAndWeight`: the nine prepared texels nearest the pixel through FSR 1's Lanczos approximation, the
 * kernel wider the less the history is to be trusted, deringed to their box, and the box itself, weighed by nearness
 * sharper the faster the view moves.
 */
function toUpsampled(
  prepared: Texture,
  position: Node<"vec2">,
  constants: IFsrConstants,
  factors: IUpsampleFactors
): { color: Node<"vec3">; weight: Node<"float">; box: IRectificationBox } {
  const { renderSize, downscale } = constants;
  const output: Node<"vec2"> = position.add(0.5).mul(downscale).toVar();
  const input: Node<"vec2"> = floor(output).toVar();
  const unjittered: Node<"vec2"> = input.add(0.5).sub(constants.jitter).toVar();
  const isFlippedX: Node<"bool"> = unjittered.x.greaterThan(output.x).toVar();
  const isFlippedY: Node<"bool"> = unjittered.y.greaterThan(output.y).toVar();
  // `offsetTL`: two back where the sample stands past the pixel, else one.
  const topLeft: Node<"vec2"> = vec2(
    select(isFlippedX, float(-2), float(-1)),
    select(isFlippedY, float(-2), float(-1))
  ).toVar();
  const baseOffset: Node<"vec2"> = unjittered.sub(output).toVar();
  const kernelReactive: Node<"float"> = max(factors.reactive, float(factors.isNew));
  const maxKernel: Node<"float"> = min(float(1.99), float(1).add(float(1).div(downscale.x).sub(1)));
  const biasMax: Node<"float"> = maxKernel.mul(kernelReactive.oneMinus()).toVar();
  const biasMin: Node<"float"> = max(float(1), biasMax.add(1).mul(0.3));
  const biasFactor: Node<"float"> = max(float(0), max(factors.depthClip.mul(0.25), kernelReactive));
  const kernelBias: Node<"float"> = mix(biasMax, biasMin, biasFactor).toVar();
  const curveBias: Node<"float"> = mix(float(-2), float(-3), saturate(factors.velocity.div(50))).toVar();
  let color: Node<"vec3"> = vec3(0);
  let weight: Node<"float"> = float(0);
  let centre: Node<"vec3"> = vec3(0);
  let moment: Node<"vec3"> = vec3(0);
  let boxWeight: Node<"float"> = float(0);
  let least: Node<"vec3"> = vec3(0);
  let most: Node<"vec3"> = vec3(0);

  for (let row: number = 0; row < 3; row += 1) {
    for (let column: number = 0; column < 3; column += 1) {
      // Flipped, so the first three rows and columns are always the box's.
      const place: Node<"vec2"> = vec2(
        select(isFlippedX, float(3 - column), float(column)),
        select(isFlippedY, float(3 - row), float(row))
      );
      const at: Node<"vec2"> = input.add(topLeft).add(place).toVar();
      const sample: Node<"vec3"> = loadClamped(prepared, at, renderSize).xyz.toVar();
      const offset: Node<"vec2"> = baseOffset.add(topLeft).add(place).toVar();
      const biased: Node<"vec2"> = offset.mul(kernelBias);
      const tap: Node<"float"> = float(isOnScreen(at, renderSize)).mul(toLanczos2ApproxSq(biased.dot(biased)));
      const nearness: Node<"float"> = exp(curveBias.mul(offset.dot(offset)));

      color = color.add(sample.mul(tap));
      weight = weight.add(tap);

      if (row === 0 && column === 0) {
        least = sample;
        most = sample;
        centre = sample.mul(nearness);
        moment = sample.mul(sample).mul(nearness);
        boxWeight = nearness;
      } else {
        least = min(least, sample);
        most = max(most, sample);
        centre = centre.add(sample.mul(nearness));
        moment = moment.add(sample.mul(sample).mul(nearness));
        boxWeight = boxWeight.add(nearness);
      }
    }
  }

  // `RectificationBoxComputeVarianceBoxData`.
  const normalizer: Node<"float"> = select(abs(boxWeight).greaterThan(FSR2_EPSILON), boxWeight, float(1)).toVar();
  const mean: Node<"vec3"> = centre.div(normalizer).toVar();
  const spread: Node<"vec3"> = sqrt(abs(moment.div(normalizer).sub(mean.mul(mean))));
  const leastVar: Node<"vec3"> = least.toVar();
  const mostVar: Node<"vec3"> = most.toVar();
  const isWeighed: Node<"bool"> = weight.greaterThan(FSR2_EPSILON);
  // Normalized and deringed where anything was weighed, the weight scaled to a frame's share.
  const upsampled: Node<"vec3"> = select(
    isWeighed,
    clamp(color.div(max(weight, FSR2_EPSILON)), leastVar, mostVar),
    color
  ).toVar();
  const scaled: Node<"float"> = select(isWeighed, weight.mul(UPSAMPLE_LANCZOS_WEIGHT_SCALE), float(0)).toVar();

  return {
    box: { centre: mean, least: leastVar, most: mostVar, spread: spread.toVar() },
    color: upsampled,
    weight: scaled,
  };
}
