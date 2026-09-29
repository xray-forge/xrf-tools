import { Nullable } from "@xrf/types";
import { RenderTarget, Scene } from "three/webgpu";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IWeatherDrawnScene } from "#/pass/weather-drawn-scene";
import { IWeatherPassInput } from "#/pass/weather-pass-input";

/**
 * What the weather draws as `RenderLast` does after the forward surfaces, the rain and then the bolt striking: over the
 * tonemapped frame, tested against its depth. Draws nothing while its scene has nothing to show.
 */
export class WeatherPass implements IRendererPass {
  public readonly name: string;

  private readonly target: RenderTarget;
  private readonly source: IWeatherDrawnScene;

  public constructor(input: IWeatherPassInput) {
    this.name = input.name;
    this.target = input.targets.composite;
    this.source = input.source;
  }

  public render({ renderer, camera }: IRendererFrame): void {
    const drawn: Nullable<Scene> = this.source.drawn;

    if (!drawn) {
      return;
    }

    renderer.setRenderTarget(this.target);
    renderer.render(drawn, camera);
  }

  /** The draws are the scene's, which lets them go with the rest of what the consumer put. */
  public dispose(): void {}
}
