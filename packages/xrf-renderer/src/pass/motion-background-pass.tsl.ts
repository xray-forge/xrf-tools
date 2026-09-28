import { Discard, Fn, If, screenCoordinate, screenUV, vec2, vec4 } from "three/tsl";
import { DepthTexture, Node } from "three/webgpu";

import { loadDepth, toTextureSize } from "#/shader/texel.tsl";
import { MotionUniforms } from "#/uniforms/motion-uniforms";

/**
 * The motion of what nothing was drawn over: the far plane, which moves with the camera alone, in the motion target's
 * terms, now less then in texture coordinates, `y` down. Every pixel something was drawn over keeps its own.
 *
 * @param depth - The drawn depth, reversed: zero where nothing was drawn.
 * @param motion - The motion uniforms, their view-projections unjittered.
 * @returns The motion, where nothing was drawn.
 */
export function toBackgroundMotion(depth: DepthTexture, motion: MotionUniforms): Node<"vec4"> {
  return Fn(() => {
    If(loadDepth(depth, screenCoordinate.xy.floor(), toTextureSize(depth)).greaterThan(0), () => {
      Discard();
    });

    const world: Node<"vec4"> = motion.inverseViewProjection.mul(
      vec4(screenUV.x.mul(2).sub(1), screenUV.y.mul(-2).add(1), 0, 1)
    );
    const before: Node<"vec4"> = motion.previousViewProjection.mul(vec4(world.xyz.div(world.w), 1));
    const uvBefore: Node<"vec2"> = before.xy.div(before.w).mul(vec2(0.5, -0.5)).add(0.5);

    return vec4(screenUV.sub(uvBefore), 0, 1);
  })();
}
