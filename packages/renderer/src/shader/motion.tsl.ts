import { Fn, modelWorldMatrix, positionLocal, varying, vec2, vec4 } from "three/tsl";
import { Node, NodeBuilder, SkinnedMesh } from "three/webgpu";

import { isBufferPlacedBuild, toBufferPlacedWorld } from "#/shader/placement.tsl";
import { previousSkinnedPosition } from "#/shader/skinned-basis.tsl";
import { MotionUniforms } from "#/uniforms/motion-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * How far a surface's point moved on the screen since the frame before, for the temporal resolve to find it in its
 * history: where the vertex stands now against where it stood then, each through its own frame's camera, unjittered.
 * A static draw stands still but for a tree's sway, at the wind of each frame; a plain object moves by its matrix, and
 * a skinned one by its bones as well, its skeleton keeping the frame before's bone matrices.
 *
 * @param uniforms - What the frame's shaders read.
 * @returns The motion in texture coordinates, now less then, `y` down.
 */
export function toSurfaceMotion(uniforms: RendererUniforms): Node<"vec2"> {
  const { motion, staticDraws, wind } = uniforms;

  return Fn((_: [], builder: NodeBuilder): Node<"vec2"> => {
    if (isBufferPlacedBuild(builder)) {
      return toClipMotion(
        motion,
        toBufferPlacedWorld(builder, staticDraws, wind),
        toBufferPlacedWorld(builder, staticDraws, wind, true)
      );
    }

    const isSkinned: boolean = Boolean((builder.object as Partial<SkinnedMesh>).isSkinnedMesh);
    const current: Node<"vec3"> = modelWorldMatrix.mul(vec4(positionLocal, 1)).xyz;
    const previous: Node<"vec3"> = motion.previousModelWorld.mul(
      vec4(isSkinned ? previousSkinnedPosition() : positionLocal, 1)
    ).xyz as Node<"vec3">;

    return toClipMotion(motion, current, previous);
  })();
}

/**
 * @param motion - What motion is measured with.
 * @param world - A point that stands still in the world, as an impostor's quad is taken to.
 * @returns How far the camera alone moved it on the screen.
 */
export function toWorldMotion(motion: MotionUniforms, world: Node<"vec3">): Node<"vec2"> {
  return toClipMotion(motion, world, world);
}

/** Both frames' clip positions, interpolated as the vertex's own, and divided per pixel. */
function toClipMotion(motion: MotionUniforms, current: Node<"vec3">, previous: Node<"vec3">): Node<"vec2"> {
  const clip: Node<"vec4"> = varying(motion.viewProjection.mul(vec4(current, 1)));
  const previousClip: Node<"vec4"> = varying(motion.previousViewProjection.mul(vec4(previous, 1)));

  return clip.xy.div(clip.w).sub(previousClip.xy.div(previousClip.w)).mul(vec2(0.5, -0.5));
}
