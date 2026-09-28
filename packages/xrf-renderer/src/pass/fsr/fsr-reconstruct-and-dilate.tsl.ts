import {
  atomicMax,
  atomicStore,
  float,
  floatBitsToUint,
  Fn,
  If,
  instanceIndex,
  length,
  max,
  pow,
  screenCoordinate,
  storage,
  uint,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { ComputeNode, Node, StorageBufferAttribute, StorageBufferNode } from "three/webgpu";

import {
  isOnScreen,
  loadFsrMotion,
  RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD,
  toNearestDepth,
  toPerceivedLuma,
  toScalarTexel,
} from "#/pass/fsr/fsr-common.tsl";
import { IFsrConstants } from "#/pass/fsr/fsr-constants";
import { IFsrInputs } from "#/pass/fsr/fsr-inputs";
import { INearestDepth } from "#/pass/fsr/nearest-depth";
import { IBilinearFootprint } from "#/shader/bilinear-footprint";
import { packOutputs, unpackOutputs } from "#/shader/packed-outputs.tsl";
import { loadClamped, toBilinearFootprint } from "#/shader/texel.tsl";

// `ffx_fsr2_reconstruct_dilated_velocity_and_previous_depth.h`: the scatter of each texel's nearest depth to where it
// stood the frame before, as a compute, and the dilation, as a draw.

/**
 * `ClearResourcesForNextFrame`, run before the reconstruction rather than after the lock: every texel of the
 * reconstructed depth at the far plane, zero inverted.
 */
export function createFsrDepthClear(buffer: StorageBufferAttribute, capacity: number): ComputeNode {
  const depths: StorageBufferNode<"uint"> = storage(buffer, "uint", capacity).toAtomic();

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
  const depths: StorageBufferNode<"uint"> = storage(buffer, "uint", capacity).toAtomic();

  return Fn(() => {
    const width: Node<"uint"> = uint(constants.renderSize.x);
    const position: Node<"vec2"> = vec2(float(instanceIndex.mod(width)), float(instanceIndex.div(width))).toVar();
    const nearest: INearestDepth = toNearestDepth(inputs, position, constants);
    const motion: Node<"vec2"> = loadFsrMotion(inputs, nearest.at, constants).toVar();
    // Motion under a tenth of a display pixel is taken for none.
    const moved: Node<"vec2"> = motion.mul(float(length(motion.mul(constants.displaySize)).greaterThan(0.1)));
    const footprint: IBilinearFootprint = toBilinearFootprint(
      position.add(0.5).div(constants.renderSize).add(moved),
      constants.renderSize
    );
    const bits: Node<"uint"> = (floatBitsToUint(nearest.depth) as unknown as Node<"uint">).toVar();

    for (const { offset, weight } of footprint.corners) {
      const at: Node<"vec2"> = footprint.base.add(vec2(...offset));

      If(weight.greaterThan(RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD).and(isOnScreen(at, constants.renderSize)), () => {
        atomicMax(depths.element(uint(at.y).mul(width).add(uint(at.x))), bits);
      });
    }
  })().compute(capacity);
}

/**
 * `ReconstructAndDilate` less the scatter.
 *
 * @returns Three outputs: the nearest depth of the nine, the motion found there, and the luma the locks read.
 */
export function toFsrDilate(inputs: IFsrInputs, constants: IFsrConstants): Node {
  const packed: Node<"mat4"> = Fn(() => {
    const position: Node<"vec2"> = screenCoordinate.xy.floor();
    const nearest: INearestDepth = toNearestDepth(inputs, position, constants);
    const motion: Node<"vec2"> = loadFsrMotion(inputs, nearest.at, constants);
    // `ComputeLockInputLuma`: an exposure of one, the colour already in the display's range.
    const color: Node<"vec3"> = max(loadClamped(inputs.color, position, constants.renderSize).xyz, vec3(0));
    const lockLuma: Node<"float"> = pow(max(toPerceivedLuma(color), float(0)), float(1 / 6));

    return packOutputs(toScalarTexel(nearest.depth), vec4(motion, 0, 1), toScalarTexel(lockLuma));
  })() as unknown as Node<"mat4">;

  return unpackOutputs(packed, 3);
}
