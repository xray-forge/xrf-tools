import { EPS_L } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { renderGroup, uniform } from "three/tsl";
import { TextureNode, UniformNode, Vector4 } from "three/webgpu";

import { IRendererClouds } from "#/contract/renderer-clouds";
import { getClearTexture } from "#/texture/placeholder-textures";
import { SlotTextureNode } from "#/texture/slot-texture-node";

/**
 * The clouds as `RenderClouds` draws them: the two keyframes' textures, their colour and cover, the dome's turn, and
 * the time their layers scroll by.
 */
export class CloudUniforms {
  /** `s_clouds0` and `s_clouds1`, pointed at the lighting's two textures as they upload, and at nothing without. */
  public readonly textures: readonly [TextureNode, TextureNode] = [
    new SlotTextureNode(getClearTexture()),
    new SlotTextureNode(getClearTexture()),
  ];
  /** The vertex colour: `clouds_color`, clamped as a colour byte clamps it. */
  public readonly color: UniformNode<"vec4", Vector4> = uniform(new Vector4()).setGroup(renderGroup);
  /** `clouds_rotation`, in radians about the vertical. */
  public readonly rotation: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** One while there are clouds to draw, zero for none. */
  public readonly drawn: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** Seconds the renderer has been running, the engine's `fTimeGlobal`. */
  public readonly time: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);

  /**
   * @param clouds - The clouds the lighting names, or none.
   */
  public take(clouds: Nullable<IRendererClouds>): void {
    const [red, green, blue, cover] = (clouds?.color ?? [0, 0, 0, 0]).map((it: number) => Math.min(Math.max(it, 0), 1));

    this.color.value.set(red, green, blue, cover);
    this.rotation.value = ((clouds?.rotation ?? 0) * Math.PI) / 180;
    // Under `EPS_L` of cover the engine draws no clouds at all.
    this.drawn.value = clouds && cover > EPS_L ? 1 : 0;
  }

  /**
   * @param time - Seconds the renderer has been running.
   */
  public update(time: number): void {
    this.time.value = time;
  }
}
