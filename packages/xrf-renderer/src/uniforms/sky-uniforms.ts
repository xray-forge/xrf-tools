import { cubeTexture, renderGroup, uniform } from "three/tsl";
import { CubeTexture, CubeTextureNode, UniformNode, Vector3 } from "three/webgpu";

import { IRendererSky } from "#/contract/renderer-sky";
import { getPlaceholderSkyTexture } from "#/texture/placeholder-textures";

/**
 * The sky as the environment binds it (`dxEnvironmentRender::lerp`): the two keyframes' cubes, how far from the first
 * to the second, the keyframes' `sky_color` and `sky_rotation`, and whether the frame draws it at all.
 */
export class SkyUniforms {
  /** `$user$sky0` and `$user$sky1`, pointed at the lighting's two skies as they upload. */
  public readonly cubes: readonly [CubeTextureNode, CubeTextureNode] = [
    cubeTexture(getPlaceholderSkyTexture() as CubeTexture),
    cubeTexture(getPlaceholderSkyTexture() as CubeTexture),
  ];
  /** How far from the first sky to the second, `L_ambient.w`. */
  public readonly blend: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** `sky_color`, the box's vertex colour. */
  public readonly color: UniformNode<"vec3", Vector3> = uniform(new Vector3(1, 1, 1)).setGroup(renderGroup);
  /** `sky_rotation`, in radians about the vertical. */
  public readonly rotation: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** One while the frame draws the sky behind the scene and fades the fog into it, zero for its backdrop. */
  public readonly drawn: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);

  /**
   * @param sky - The sky the lighting names.
   */
  public take(sky: IRendererSky): void {
    this.blend.value = sky.blend;
    this.color.value.set(sky.color[0], sky.color[1], sky.color[2]);
    this.rotation.value = (sky.rotation * Math.PI) / 180;
  }

  /**
   * @param isDrawn - Whether the consumer's frame draws the sky.
   */
  public setDrawn(isDrawn: boolean): void {
    this.drawn.value = isDrawn ? 1 : 0;
  }
}
