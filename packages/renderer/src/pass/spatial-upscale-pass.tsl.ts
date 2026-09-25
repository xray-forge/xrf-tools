import {
  abs,
  clamp,
  float,
  floor,
  Fn,
  int,
  inverseSqrt,
  ivec2,
  max,
  min,
  saturate,
  screenCoordinate,
  screenSize,
  select,
  texture,
  textureLoad,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { toUpscaledCoverage } from "#/pass/upscale-depth.tsl";

/** The twelve taps around `f`, by name: `b c` above, `e f g h` and `i j k l` across, `n o` below. */
const TAPS = {
  b: [0, -1],
  c: [1, -1],
  e: [-1, 0],
  f: [0, 0],
  g: [1, 0],
  h: [2, 0],
  i: [-1, 1],
  j: [0, 1],
  k: [1, 1],
  l: [2, 1],
  n: [0, 2],
  o: [1, 2],
} as const;

type TTap = keyof typeof TAPS;

interface IEasuSums {
  direction: Node<"vec2">;
  length: Node<"float">;
}

/**
 * `ffxFsrEasuFloat` (FidelityFX, `ffx_fsr1.h`, MIT): each output pixel reconstructed from the twelve drawn texels around
 * it with a Lanczos-2 approximation stretched along the local edge, deringed to the four nearest, so an edge upscales
 * sharp without the stair a bilinear stretch leaves.
 *
 * @param frame - The frame as drawn, in the display's range.
 * @returns The upscaled colour, the drawn coverage nearest each pixel kept.
 */
export function toSpatialUpscale(frame: Texture): Node<"vec4"> {
  return Fn(() => {
    const inputSize: Node<"vec2"> = vec2(texture(frame).size(int(0)) as Node<"uvec2">);
    const last: Node<"vec2"> = inputSize.sub(1);
    // `con0`: the output pixel's centre in the drawn pixels, less half a texel, so `f` is the texel at or left of it.
    const position: Node<"vec2"> = screenCoordinate.xy.floor().add(0.5).mul(inputSize.div(screenSize)).sub(0.5);
    const base: Node<"vec2"> = floor(position);
    const fraction: Node<"vec2"> = position.sub(base).toVar();
    const colors = {} as Record<TTap, Node<"vec3">>;
    const lumas = {} as Record<TTap, Node<"float">>;

    for (const [name, [x, y]] of Object.entries(TAPS) as Array<[TTap, readonly [number, number]]>) {
      const color: Node<"vec3"> = textureLoad(frame, ivec2(clamp(base.add(vec2(x, y)), vec2(0), last))).xyz.toVar();

      colors[name] = color;
      // Luma times two, in two multiply-adds.
      lumas[name] = color.z.mul(0.5).add(color.x.mul(0.5).add(color.y));
    }

    const sums: IEasuSums = { direction: vec2(0), length: float(0) };
    const { x: px, y: py } = { x: fraction.x, y: fraction.y };

    toEasuSet(sums, px.oneMinus().mul(py.oneMinus()), lumas.b, lumas.e, lumas.f, lumas.g, lumas.j);
    toEasuSet(sums, px.mul(py.oneMinus()), lumas.c, lumas.f, lumas.g, lumas.h, lumas.k);
    toEasuSet(sums, px.oneMinus().mul(py), lumas.f, lumas.i, lumas.j, lumas.k, lumas.n);
    toEasuSet(sums, px.mul(py), lumas.g, lumas.j, lumas.k, lumas.l, lumas.o);

    // Normalized, and a direction too short to trust taken as across.
    const squared: Node<"float"> = sums.direction.dot(sums.direction);
    const isFlat: Node<"bool"> = squared.lessThan(1 / 32768);
    const direction: Node<"vec2"> = select(isFlat, vec2(1, sums.direction.y), sums.direction)
      .mul(select(isFlat, float(1), inverseSqrt(max(squared, 1e-12))))
      .toVar();
    // From `0..2` to `0..1`, shaped by its square.
    const edge: Node<"float"> = sums.length.mul(0.5).toVar();
    const shaped: Node<"float"> = edge.mul(edge).toVar();
    // The kernel stretched from one across or down to `sqrt(2)` on a diagonal.
    const stretch: Node<"float"> = direction.dot(direction).div(max(max(abs(direction.x), abs(direction.y)), 1e-8));
    const anisotropy: Node<"vec2"> = vec2(
      float(1).add(stretch.sub(1).mul(shaped)),
      float(1).sub(shaped.mul(0.5))
    ).toVar();
    // The window shifts from `sqrt(2)` to slightly past two as the edge grows.
    const lobe: Node<"float"> = float(0.5)
      .add(float(1 / 4 - 0.04 - 0.5).mul(shaped))
      .toVar();
    const clip: Node<"float"> = float(1).div(lobe);
    const least: Node<"vec3"> = min(min(colors.f, colors.g), min(colors.j, colors.k));
    const most: Node<"vec3"> = max(max(colors.f, colors.g), max(colors.j, colors.k));
    let color: Node<"vec3"> = vec3(0);
    let weight: Node<"float"> = float(0);

    for (const [name, [x, y]] of Object.entries(TAPS) as Array<[TTap, readonly [number, number]]>) {
      const tap: Node<"float"> = toEasuTap(vec2(x, y).sub(fraction), direction, anisotropy, lobe, clip);

      color = color.add(colors[name].mul(tap));
      weight = weight.add(tap);
    }

    const upscaled: Node<"vec3"> = min(most, max(least, color.div(weight)));

    return vec4(upscaled, toUpscaledCoverage(frame, vec2(0)));
  })();
}

/**
 * `fsrEasuSetFloat`: one bilinear corner's share of the edge's direction and strength, from the `+` of lumas around it.
 *
 * @param sums - The direction and length summed so far.
 * @param weight - The corner's bilinear weight.
 * @param above - The luma above its centre.
 * @param left - Left of it.
 * @param centre - At it.
 * @param right - Right of it.
 * @param below - And below it.
 */
function toEasuSet(
  sums: IEasuSums,
  weight: Node<"float">,
  above: Node<"float">,
  left: Node<"float">,
  centre: Node<"float">,
  right: Node<"float">,
  below: Node<"float">
): void {
  function toAxis(before: Node<"float">, after: Node<"float">): [Node<"float">, Node<"float">] {
    // A reversal of the gradient comes to nothing, a steady one to one, shaped by its square. HLSL's approximate
    // reciprocal of zero is large but finite; in WGSL it is infinite, so the span is kept off zero.
    const span: Node<"float"> = max(max(abs(after.sub(centre)), abs(centre.sub(before))), 1e-8);
    const delta: Node<"float"> = after.sub(before);
    const strength: Node<"float"> = saturate(abs(delta).div(span));

    return [delta, strength.mul(strength)];
  }

  const [dx, lx] = toAxis(left, right);
  const [dy, ly] = toAxis(above, below);

  sums.direction = sums.direction.add(vec2(dx, dy).mul(weight));
  sums.length = sums.length.add(lx.add(ly).mul(weight));
}

/**
 * `fsrEasuTapFloat`: one tap's weight, its offset turned along the edge and stretched, under the approximate Lanczos-2
 * `(25/16 (2/5 x² - 1)² - 9/16) (w x² - 1)²` clipped at the window's end.
 */
function toEasuTap(
  offset: Node<"vec2">,
  direction: Node<"vec2">,
  anisotropy: Node<"vec2">,
  lobe: Node<"float">,
  clip: Node<"float">
): Node<"float"> {
  const turned: Node<"vec2"> = vec2(
    offset.x.mul(direction.x).add(offset.y.mul(direction.y)),
    offset.x.mul(direction.y.negate()).add(offset.y.mul(direction.x))
  ).mul(anisotropy);
  const distance: Node<"float"> = min(turned.dot(turned), clip);
  const base: Node<"float"> = distance.mul(2 / 5).sub(1);
  const window: Node<"float"> = lobe.mul(distance).sub(1);

  return base
    .mul(base)
    .mul(25 / 16)
    .sub(25 / 16 - 1)
    .mul(window.mul(window));
}
