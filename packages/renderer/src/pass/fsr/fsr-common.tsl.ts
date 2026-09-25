import {
  abs,
  clamp,
  dot,
  float,
  floor,
  int,
  ivec2,
  log,
  mat4,
  max,
  min,
  outputStruct,
  pow,
  select,
  sin,
  textureLoad,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node, Texture } from "three/webgpu";

// FidelityFX FSR 2.2 (`ffx_fsr2_common.h`, `ffx_fsr2_sample.h`, AMD, MIT): the helpers its passes share, for inverted
// depth, colour in the display's range with an exposure of one, and motion drawn at the render size.

export const FSR2_EPSILON: number = 1e-3;
export const FSR2_FP16_MAX: number = 65504;
/** `fReconstructedDepthBilinearWeightThreshold`. */
export const RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD: number = 0.01;
/** `fUpsampleLanczosWeightScale`. */
export const UPSAMPLE_LANCZOS_WEIGHT_SCALE: number = 1 / 12;
/** `fMaxAccumulationLanczosWeight`. */
export const MAX_ACCUMULATION_LANCZOS_WEIGHT: number = 1;
/** `fAverageLanczosWeightPerFrame`. */
export const AVERAGE_LANCZOS_WEIGHT_PER_FRAME: number = 0.74 * UPSAMPLE_LANCZOS_WEIGHT_SCALE;

/** The 3x3 neighbourhood, row by row. */
export const NEIGHBOURHOOD: ReadonlyArray<readonly [number, number]> = [-1, 0, 1].flatMap((y: number) =>
  [-1, 0, 1].map((x: number) => [x, y] as const)
);

/** What every FSR pass reads of the frame: `cbFSR2`'s fields the passes use, as uniforms. */
export interface IFsrConstants {
  renderSize: Node<"vec2">;
  displaySize: Node<"vec2">;
  /** In FSR's sense: a drawn texel `m` stands at `m + 0.5 - jitter`. */
  jitter: Node<"vec2">;
  /** The drawing's size over the display's. */
  downscale: Node<"vec2">;
  /** `fDeviceToViewDepth`: device depth to view depth, `[1] / (d - [0])`, and the projection's inverse scales. */
  deviceToView: Node<"vec4">;
  lumaMipSize: Node<"vec2">;
  jitterPhaseCount: Node<"float">;
  /** Zero on the first frame after a reset. */
  frameIndex: Node<"float">;
}

export function toYCoCg(rgb: Node<"vec3">): Node<"vec3"> {
  return vec3(
    rgb.x.mul(0.25).add(rgb.y.mul(0.5)).add(rgb.z.mul(0.25)),
    rgb.x.mul(0.5).sub(rgb.z.mul(0.5)),
    rgb.x.mul(-0.25).add(rgb.y.mul(0.5)).sub(rgb.z.mul(0.25))
  );
}

export function toRgb(yCoCg: Node<"vec3">): Node<"vec3"> {
  return vec3(yCoCg.x.add(yCoCg.y).sub(yCoCg.z), yCoCg.x.add(yCoCg.z), yCoCg.x.sub(yCoCg.y).sub(yCoCg.z));
}

export function toLuma(rgb: Node<"vec3">): Node<"float"> {
  return dot(rgb, vec3(0.2126, 0.7152, 0.0722));
}

/** `RGBToPerceivedLuma`: CIE lightness over a hundred. */
export function toPerceivedLuma(rgb: Node<"vec3">): Node<"float"> {
  const luminance: Node<"float"> = toLuma(rgb).toVar();

  return select(
    luminance.lessThanEqual(216 / 24389),
    luminance.mul(24389 / 27),
    pow(max(luminance, 0), float(1 / 3))
      .mul(116)
      .sub(16)
  ).mul(0.01);
}

/** `GetViewSpaceDepth`: a device depth as a distance along the view. */
export function toViewDepth(depth: Node<"float">, constants: IFsrConstants): Node<"float"> {
  return constants.deviceToView.y.div(depth.sub(constants.deviceToView.x));
}

/** `GetMaxDistanceInMeters`: inverted, the far plane stands at zero. */
export function toMaxDistance(constants: IFsrConstants): Node<"float"> {
  return toViewDepth(float(0), constants);
}

/** `IsOnScreen`, for a texel position held as floats. */
export function isOnScreen(position: Node<"vec2">, size: Node<"vec2">): Node<"bool"> {
  return position.greaterThanEqual(vec2(0)).all().and(position.lessThan(size).all());
}

/** `ClampLoad`: a texel, kept on the texture. */
export function toClampedTexel(position: Node<"vec2">, size: Node<"vec2">): Node<"ivec2"> {
  return ivec2(clamp(position, vec2(0), size.sub(1)));
}

/** A texture's texel at a float position, kept on it. */
export function loadClamped(source: Texture, position: Node<"vec2">, size: Node<"vec2">): Node<"vec4"> {
  return textureLoad(source, toClampedTexel(position, size));
}

/** `ClampUv`: a coordinate kept half a texel inside a texture of a size. */
export function toClampedUv(uv: Node<"vec2">, size: Node<"vec2">): Node<"vec2"> {
  return clamp(uv.mul(size), vec2(0.5), size.sub(0.5)).div(size);
}

/** `ComputeHrPosFromLrPos`: the display pixel a drawn texel's sample falls in. */
export function toDisplayPosition(renderPosition: Node<"vec2">, constants: IFsrConstants): Node<"vec2"> {
  return floor(renderPosition.add(0.5).sub(constants.jitter).div(constants.renderSize).mul(constants.displaySize));
}

/** `Lanczos2`: the reference two-lobe window. */
export function toLanczos2(x: Node<"float">): Node<"float"> {
  const at: Node<"float"> = min(abs(x), float(2)).toVar();
  const pi: number = Math.PI;

  return select(
    at.lessThan(FSR2_EPSILON),
    float(1),
    sin(at.mul(pi))
      .div(at.mul(pi))
      .mul(sin(at.mul(0.5 * pi)).div(at.mul(0.5 * pi)))
  );
}

/** `Lanczos2ApproxSq`: FSR 1's approximation, of the squared distance, no further than two. */
export function toLanczos2ApproxSq(x2: Node<"float">): Node<"float"> {
  const at: Node<"float"> = min(x2, float(4));
  const a: Node<"float"> = at.mul(2 / 5).sub(1);
  const b: Node<"float"> = at.mul(1 / 4).sub(1);

  return a
    .mul(a)
    .mul(25 / 16)
    .sub(25 / 16 - 1)
    .mul(b.mul(b));
}

/**
 * `DeclareCustomTextureSample(..., Lanczos2, FetchBicubicSamples)`: a texture at a coordinate through a separable
 * Lanczos-2 over the sixteen texels around it, deringed to the four nearest.
 */
export function toLanczos2Sample(source: Texture, uv: Node<"vec2">, size: Node<"vec2">): Node<"vec4"> {
  const position: Node<"vec2"> = clamp(uv.mul(size).sub(0.5), vec2(0), size).toVar();
  const base: Node<"vec2"> = floor(position).toVar();
  const fraction: Node<"vec2"> = position.sub(base).toVar();
  const rows: Array<Node<"vec4">> = [];

  function texel(x: number, y: number): Node<"vec4"> {
    return loadClamped(source, base.add(vec2(x, y)), size).toVar();
  }

  function weights(t: Node<"float">): Array<Node<"float">> {
    return [-1, 0, 1, 2].map((offset: number) => toLanczos2(float(offset).sub(t)).toVar());
  }

  const across: Array<Node<"float">> = weights(fraction.x);
  const down: Array<Node<"float">> = weights(fraction.y);
  const acrossTotal: Node<"float"> = across[0].add(across[1]).add(across[2]).add(across[3]);
  const downTotal: Node<"float"> = down[0].add(down[1]).add(down[2]).add(down[3]);
  const nearest: Array<Node<"vec4">> = [];

  for (let y: number = -1; y <= 2; y += 1) {
    const samples: Array<Node<"vec4">> = [-1, 0, 1, 2].map((x: number) => texel(x, y));

    if (y === 0 || y === 1) {
      nearest.push(samples[1], samples[2]);
    }

    rows.push(
      samples[0]
        .mul(across[0])
        .add(samples[1].mul(across[1]))
        .add(samples[2].mul(across[2]))
        .add(samples[3].mul(across[3]))
        .div(acrossTotal)
    );
  }

  const filtered: Node<"vec4"> = rows[0]
    .mul(down[0])
    .add(rows[1].mul(down[1]))
    .add(rows[2].mul(down[2]))
    .add(rows[3].mul(down[3]))
    .div(downTotal);
  const least: Node<"vec4"> = min(min(nearest[0], nearest[1]), min(nearest[2], nearest[3]));
  const most: Node<"vec4"> = max(max(nearest[0], nearest[1]), max(nearest[2], nearest[3]));

  return clamp(filtered, least, most);
}

/** The natural log of a luma kept off zero, as the luminance pyramid takes it. */
export function toLogLuma(rgb: Node<"vec3">): Node<"float"> {
  return log(max(toLuma(rgb), float(FSR2_EPSILON)));
}

/**
 * Several targets' outputs from one `Fn`: three takes an `outputStruct` only built outside a `Fn`, so the `Fn` packs
 * them as a matrix's columns, which this unpacks.
 *
 * @param outputs - Up to four outputs.
 * @returns The packed matrix, for the `Fn` to return.
 */
export function packOutputs(...outputs: Array<Node<"vec4">>): Node<"mat4"> {
  const columns: Array<Node<"vec4">> = [...outputs, vec4(0), vec4(0), vec4(0)].slice(0, 4);

  return mat4(columns[0], columns[1], columns[2], columns[3]);
}

/**
 * @param packed - What `packOutputs` packed, as the `Fn` returned it.
 * @param count - How many outputs it packed.
 * @returns The outputs, one a target.
 */
export function unpackOutputs(packed: Node<"mat4">, count: number): Node {
  return outputStruct(
    ...Array.from({ length: count }, (_, index: number) =>
      (packed as unknown as { element(index: Node<"int">): Node<"vec4"> }).element(int(index))
    )
  );
}

/** A vec4 of one value, for a single-channel target. */
export function toScalarTexel(value: Node<"float">): Node<"vec4"> {
  return vec4(value, 0, 0, 1);
}
