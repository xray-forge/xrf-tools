import { Nullable } from "@xrf/types";
import { renderGroup, uniform } from "three/tsl";
import { UniformNode, Vector2, Vector3 } from "three/webgpu";

import { IRendererThunderboltGlow } from "#/contract/weather/renderer-thunderbolt-glow";
import { IRendererThunderboltStrike } from "#/contract/weather/renderer-thunderbolt-strike";
import { IThunderGlowUniforms } from "#/uniforms/thunder-glow-uniforms";

/**
 * The bolt striking as `dxThunderboltRender::Render` draws it: its texture's shift and its two glows.
 */
export class ThunderUniforms {
  /** What the model's texture coordinates are shifted down by. */
  public readonly shift: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  public readonly top: IThunderGlowUniforms = createGlowUniforms();
  public readonly center: IThunderGlowUniforms = createGlowUniforms();

  /**
   * @param strike - The bolt striking this frame, or null for none.
   */
  public take(strike: Nullable<IRendererThunderboltStrike>): void {
    if (strike) {
      this.shift.value = strike.shift;
      takeGlow(this.top, strike.top);
      takeGlow(this.center, strike.center);
    }
  }
}

function createGlowUniforms(): IThunderGlowUniforms {
  return {
    extent: uniform(new Vector2()).setGroup(renderGroup),
    opacity: uniform(0).setGroup(renderGroup),
    position: uniform(new Vector3()).setGroup(renderGroup),
  };
}

function takeGlow(uniforms: IThunderGlowUniforms, glow: IRendererThunderboltGlow): void {
  const [x, y, z] = glow.position;

  // Engine `z` negated into renderer space.
  uniforms.position.value.set(x, y, -z);
  uniforms.extent.value.set(glow.extent[0], glow.extent[1]);
  uniforms.opacity.value = glow.opacity;
}
