import { getViewPosition, outputStruct, screenUV, texture, vec4 } from "three/tsl";
import { Node } from "three/webgpu";

import { IGBufferSample } from "#/shader/gbuffer-sample";
import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { decodeOctahedral, encodeOctahedral } from "#/shader/octahedral-normal.tsl";
import { CameraUniforms } from "#/uniforms/camera-uniforms";

/**
 * What a surface writes into the G-buffer, by location in the order `RendererTargets` attaches the targets.
 * Never three's `mrt`: it finds its locations on whichever render target is current, and a pipeline compiled off the
 * frame is built after the frame moved on from the G-buffer.
 *
 * @param albedo - Raw albedo in colour, gloss in alpha.
 * @param normal - The view space normal, unit length.
 * @param hemi - The baked hemisphere occlusion.
 * @param sun - The baked sun occlusion.
 * @param slice - The lighting model slice.
 * @param motion - How far the point moved on the screen since the frame before, in texture coordinates.
 * @returns The fragment's outputs.
 */
export function toGBufferOutput(
  albedo: Node<"vec4">,
  normal: Node<"vec3">,
  hemi: Node<"float">,
  sun: Node<"float">,
  slice: Node<"float">,
  motion: Node<"vec2">
): Node {
  return outputStruct(albedo, vec4(encodeOctahedral(normal), 0, 1), vec4(hemi, sun, slice, 0), vec4(motion, 0, 1));
}

/**
 * @param textures - The G-buffer's attachments.
 * @param camera - The drawing camera's uniforms, which the view position is rebuilt with.
 * @returns The G-buffer at the pixel being shaded.
 */
export function readGBuffer(textures: IGBufferTextures, camera: CameraUniforms): IGBufferSample {
  const depth: Node<"float"> = texture(textures.depth, screenUV).x;
  const albedo: Node<"vec4"> = texture(textures.albedo, screenUV);
  const surface: Node<"vec4"> = texture(textures.surface, screenUV);

  return {
    albedo: albedo.xyz,
    depth,
    gloss: albedo.w,
    hemi: surface.x,
    point: {
      normal: decodeOctahedral(texture(textures.normal, screenUV).xy),
      position: getViewPosition(screenUV, depth, camera.projectionInverse),
      slice: surface.z,
    },
    sun: surface.y,
  };
}
