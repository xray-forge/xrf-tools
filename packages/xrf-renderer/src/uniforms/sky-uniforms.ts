import { renderGroup, uniform } from "three/tsl";
import { CubeTexture, CubeTextureNode, UniformNode, Vector3 } from "three/webgpu";

import { IRendererSky } from "#/contract/renderer-sky";
import { getClearTexture, getPlaceholderSkyTexture } from "#/texture/placeholder-textures";
import { SlotCubeTextureNode } from "#/texture/slot-cube-texture-node";
import { SlotTextureNode } from "#/texture/slot-texture-node";

/**
 * The sky as the environment binds it (`dxEnvironmentRender::lerp`): the two keyframes' cubes and their irradiance
 * cubes, how far from the first to the second, the keyframes' `sky_color` and `sky_rotation`, and whether the frame draws it at all.
 */
export class SkyUniforms {
  /** `$user$sky0` and `$user$sky1`, pointed at the lighting's two skies as they upload. */
  public readonly cubes: readonly [CubeTextureNode, CubeTextureNode] = [
    new SlotCubeTextureNode(getPlaceholderSkyTexture() as CubeTexture),
    new SlotCubeTextureNode(getPlaceholderSkyTexture() as CubeTexture),
  ];
  /** `env_s0` and `env_s1`, the two skies' irradiance cubes, pointed at them as they upload. */
  public readonly environments: readonly [CubeTextureNode, CubeTextureNode] = [
    new SlotCubeTextureNode(getPlaceholderSkyTexture() as CubeTexture),
    new SlotCubeTextureNode(getPlaceholderSkyTexture() as CubeTexture),
  ];
  /** The sky as drawn, clouds and all, blurred into the haze map the distance fades into: what `SkyHaze` draws. */
  public readonly haze: SlotTextureNode = new SlotTextureNode(getClearTexture());
  /** One while both irradiance cubes are up, zero while the lighting's stand-in lights the hemisphere. */
  public readonly environmentsUp: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** How far from the first sky to the second, `L_ambient.w`. */
  public readonly blend: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** `sky_color`, the box's vertex colour. */
  public readonly color: UniformNode<"vec3", Vector3> = uniform(new Vector3(1, 1, 1)).setGroup(renderGroup);
  /** `sky_rotation`, in radians about the vertical. */
  public readonly rotation: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** One while the frame draws the sky behind the scene and fades the fog into it, zero for its backdrop. */
  public readonly drawn: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** One while the sky is drawn through the tonemap's curve, as Anomaly's `sky2` draws it. */
  public readonly curved: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** One while the distance fades into the sky's haze rather than into the sky itself. */
  public readonly hazed: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);

  /**
   * @param sky - The sky the lighting names.
   */
  public take(sky: IRendererSky): void {
    this.blend.value = sky.blend;
    this.color.value.set(sky.color[0], sky.color[1], sky.color[2]);
    this.rotation.value = (sky.rotation * Math.PI) / 180;
    this.curved.value = sky.isCurved ? 1 : 0;
  }

  /**
   * @param isDrawn - Whether the consumer's frame draws the sky.
   */
  public setDrawn(isDrawn: boolean): void {
    this.drawn.value = isDrawn ? 1 : 0;
  }

  /**
   * @param isHazed - Whether the distance fades into the sky's haze rather than into the sky itself.
   */
  public setHazed(isHazed: boolean): void {
    this.hazed.value = isHazed ? 1 : 0;
  }
}
