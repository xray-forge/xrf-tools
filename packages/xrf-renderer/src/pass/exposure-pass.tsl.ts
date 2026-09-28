import {
  clamp,
  dot,
  float,
  Fn,
  instanceIndex,
  ivec2,
  mix,
  sqrt,
  storage,
  textureLoad,
  uint,
  vec2,
  vec3,
} from "three/tsl";
import { ComputeNode, Node, Texture } from "three/webgpu";

import { loopNamed } from "#/shader/named-loop.tsl";
import { WHITE_INTENSITY_SQUARED } from "#/shader/tonemap.tsl";
import { EXPOSURE_CELLS, ExposureUniforms } from "#/uniforms/exposure-uniforms";

/** `LUMINANCE_VECTOR` (`shaders/r3/common_defines.h`). */
const LUMINANCE: readonly [number, number, number] = [0.3, 0.38, 0.22];

/** Samples a side each cell averages, over its extent. */
const CELL_SAMPLES: number = 4;

/**
 * The colour before `tonemap`'s curve, from the colour after it: the curve `x (1 + x / w²) / (1 + x)` solved for `x`,
 * which is the scaled colour `combine_1` writes as its high part.
 */
function toUntonemapped(low: Node<"vec3">): Node<"vec3"> {
  const rest: Node<"vec3"> = vec3(1).sub(low);

  return rest
    .negate()
    .add(sqrt(rest.mul(rest).add(low.mul(4 / WHITE_INTENSITY_SQUARED))))
    .mul(WHITE_INTENSITY_SQUARED / 2);
}

/**
 * `bloom_luminance_1.ps` over the frame as combine finished it: each of 64 by 64 cells' luminance, the scaled colour's
 * `dot(rgb, LUMINANCE_VECTOR)` times two, as the bloom target it reads is twice the average it was built from.
 *
 * @param frame - The frame combine wrote.
 * @param exposure - Where the cells go.
 * @param size - The frame's size in texels.
 * @returns The measurement, one invocation a cell.
 */
export function createExposureMeasure(frame: Texture, exposure: ExposureUniforms, size: Node<"vec2">): ComputeNode {
  const cells = storage(exposure.cells, "float", EXPOSURE_CELLS * EXPOSURE_CELLS);

  return Fn(() => {
    const cell: Node<"vec2"> = vec2(
      float(instanceIndex.mod(EXPOSURE_CELLS)),
      float(instanceIndex.div(EXPOSURE_CELLS))
    ).toVar();
    const total = float(0).toVar();

    loopNamed(
      { end: uint(CELL_SAMPLES * CELL_SAMPLES), name: "sample", start: uint(0), type: "uint" },
      (sample: Node<"uint">) => {
        const offset: Node<"vec2"> = vec2(float(sample.mod(CELL_SAMPLES)), float(sample.div(CELL_SAMPLES)))
          .add(0.5)
          .div(CELL_SAMPLES);
        const at: Node<"vec2"> = cell.add(offset).div(EXPOSURE_CELLS).mul(size);
        const low: Node<"vec3"> = textureLoad(frame, ivec2(at)).xyz;

        total.addAssign(dot(toUntonemapped(low), vec3(...LUMINANCE)).mul(2));
      }
    );

    cells.element(instanceIndex).assign(total.div(CELL_SAMPLES * CELL_SAMPLES));
  })().compute(EXPOSURE_CELLS * EXPOSURE_CELLS);
}

/**
 * `bloom_luminance_3.ps`: the cells averaged, the scale that brings them to the middle gray, and the adapted scale
 * moved towards it by the frame's share. Held between a 128th and twenty, as the engine's own clamp meant to: its line
 * discards the result.
 *
 * @param exposure - The cells, the adapted scale, and the constants.
 * @returns The adaptation, one invocation.
 */
export function createExposureAdaptation(exposure: ExposureUniforms): ComputeNode {
  const cells = storage(exposure.cells, "float", EXPOSURE_CELLS * EXPOSURE_CELLS).toReadOnly();
  const adapted = storage(exposure.adapted, "float", 1);

  return Fn(() => {
    const total = float(0).toVar();

    loopNamed(
      { end: uint(EXPOSURE_CELLS * EXPOSURE_CELLS), name: "cell", start: uint(0), type: "uint" },
      (cell: Node<"uint">) => {
        total.addAssign(cells.element(cell));
      }
    );

    const luminance: Node<"float"> = total.div(EXPOSURE_CELLS * EXPOSURE_CELLS);
    const scale: Node<"float"> = exposure.target.div(luminance.mul(exposure.weight).add(exposure.floor));
    const previous: Node<"float"> = adapted.element(0) as unknown as Node<"float">;

    adapted.element(0).assign(clamp(mix(previous, scale, exposure.blend), 1 / 128, 20));
  })().compute(1);
}
