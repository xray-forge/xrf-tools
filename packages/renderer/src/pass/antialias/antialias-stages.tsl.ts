import { fxaa } from "three/addons/tsl/display/FXAANode.js";
import { texture, uniform } from "three/tsl";
import { Node, Texture, UniformNode, Vector2 } from "three/webgpu";

import { ISmaaStages, toSmaaStages } from "#/pass/antialias/smaa-stages.tsl";

/** What SMAA's stages sample, as textures the pass holds. */
export interface ISmaaTextures {
  frame: Texture;
  edges: Texture;
  weights: Texture;
  area: Texture;
  search: Texture;
}

/**
 * @returns One over a frame's size in pixels, which the pass sets as the frame resizes.
 */
export function createAntialiasSize(): UniformNode<"vec2", Vector2> {
  return uniform(new Vector2());
}

/**
 * @param frame - The tonemapped frame.
 * @returns It, as it is: what a pass draws while its lookups are still on their way.
 */
export function toFrameCopy(frame: Texture): Node<"vec4"> {
  return texture(frame);
}

/**
 * @param frame - The tonemapped frame.
 * @returns It smoothed by three's own `FXAANode`.
 */
export function toFxaaStage(frame: Texture): Node {
  return fxaa(texture(frame));
}

/**
 * @param textures - What the stages sample.
 * @param invSize - One over the frame's size in pixels.
 * @returns SMAA's three stages over them.
 */
export function toSmaaPipeline(textures: ISmaaTextures, invSize: UniformNode<"vec2", Vector2>): ISmaaStages {
  return toSmaaStages({
    areaTexture: texture(textures.area),
    edgesTexture: texture(textures.edges),
    invSize,
    searchTexture: texture(textures.search),
    sourceTexture: texture(textures.frame),
    weightsTexture: texture(textures.weights),
  });
}
