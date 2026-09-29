import { attribute, cameraWorldMatrix, texture, vec2, vec4 } from "three/tsl";
import { DoubleSide, MeshBasicNodeMaterial, Node, TextureNode } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { applySurfaceCompositing, ISurfaceCompositing, toSurfaceCompositing } from "#/material/surface-compositing";
import { getClearTexture } from "#/texture/placeholder-textures";
import { IThunderGlowUniforms } from "#/uniforms/thunder-glow-uniforms";
import { ThunderUniforms } from "#/uniforms/thunder-uniforms";

/**
 * A thunder material and the sampler its texture is bound to.
 */
export interface IThunderSurface {
  material: MeshBasicNodeMaterial;
  /** Samples nothing until its texture is bound. */
  texture: TextureNode;
}

/**
 * A material drawn over the tonemapped frame as its shader composites it, tested against its depth and writing none,
 * unlit, both sides: `CULL_NONE` for the model, and a glow faces the view anyway.
 */
function createThunderMaterial(draw: ERendererDraw, fragment: (sampler: TextureNode) => Node<"vec4">): IThunderSurface {
  const material: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
  const sampler: TextureNode = texture(getClearTexture());
  // A shader that composites nothing still lights the air: the engine's lightning is added.
  const compositing: ISurfaceCompositing =
    toSurfaceCompositing({ draw }) ?? (toSurfaceCompositing({ draw: ERendererDraw.ADDED }) as ISurfaceCompositing);

  material.fragmentNode = fragment(sampler);
  material.side = DoubleSide;
  material.fog = false;
  applySurfaceCompositing(material, { ...compositing, isPulled: false });

  return { material, texture: sampler };
}

/**
 * A bolt's `lightning_model`: its texture, white as its vertices are, its coordinates shifted down by the strike.
 *
 * @param draw - How its shader composites it.
 * @param thunder - The strike's uniforms.
 * @returns The material, and the sampler its texture is bound to.
 */
export function toThunderboltSurface(draw: ERendererDraw, thunder: ThunderUniforms): IThunderSurface {
  const coordinates: Node<"vec2"> = attribute("uv", "vec2");

  return createThunderMaterial(draw, (sampler: TextureNode) => sampler.sample(coordinates.add(vec2(0, thunder.shift))));
}

/**
 * One of a bolt's glows (`SFlare`): a quad about its point, facing the view along its right and its top, its texture
 * scaled in colour and alpha by the glow's opacity.
 *
 * @param draw - How its shader composites it.
 * @param glow - The glow's uniforms.
 * @returns The material, and the sampler its texture is bound to.
 */
export function toThunderGlowSurface(draw: ERendererDraw, glow: IThunderGlowUniforms): IThunderSurface {
  const corner: Node<"vec2"> = attribute("corner", "vec2");
  // The corner to its right and top has the first texel, as the engine lays the quad out.
  const coordinates: Node<"vec2"> = vec2(corner.x.oneMinus(), corner.y.oneMinus()).mul(0.5);
  const surface: IThunderSurface = createThunderMaterial(draw, (sampler: TextureNode) =>
    sampler.sample(coordinates).mul(vec4(glow.opacity))
  );
  const right: Node<"vec3"> = cameraWorldMatrix.mul(vec4(1, 0, 0, 0)).xyz;
  const up: Node<"vec3"> = cameraWorldMatrix.mul(vec4(0, 1, 0, 0)).xyz;

  surface.material.positionNode = glow.position
    .add(right.mul(corner.x.mul(glow.extent.x)))
    .add(up.mul(corner.y.mul(glow.extent.y)));

  return surface;
}
