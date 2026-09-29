import { Nullable } from "@xrf/types";
import { RenderTarget, Scene } from "three/webgpu";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { SceneRain } from "#/scene/rain/scene-rain";
import { RainUniforms } from "#/uniforms/rain-uniforms";

/**
 * The rain, as `RenderLast` draws it after the forward surfaces: its streaks and splashes blended over the tonemapped
 * frame, tested against its depth. Draws nothing while it does not rain or its draws are still compiling.
 */
export class RainPass implements IRendererPass {
  public readonly name: string = "rain";

  private readonly target: RenderTarget;
  private readonly scene: SceneRain;
  private readonly rain: RainUniforms;

  /**
   * @param targets - The frame's targets, whose composite the rain draws over.
   * @param scene - The rain's draws.
   * @param rain - The rain's uniforms.
   */
  public constructor(targets: RendererTargets, scene: SceneRain, rain: RainUniforms) {
    this.target = targets.composite;
    this.scene = scene;
    this.rain = rain;
  }

  public render({ renderer, camera }: IRendererFrame): void {
    const drawn: Nullable<Scene> = this.scene.drawn;

    if (!this.rain.isFalling || !drawn) {
      return;
    }

    renderer.setRenderTarget(this.target);
    renderer.render(drawn, camera);
  }

  /** The draws are the scene's, which lets them go with the rest of what the consumer put. */
  public dispose(): void {}
}
