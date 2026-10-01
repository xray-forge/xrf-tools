import { Nullable } from "@xrf/types";
import { Color, LinearSRGBColorSpace, Texture } from "three/webgpu";

import { drawCleared } from "#/pass/cleared-draw";
import { toCombinePassFragment } from "#/pass/combine-pass.tsl";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { SkyHaze } from "#/pass/sky-haze";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * `combine_1` and the tonemap of `combine_2`: the hemisphere model, the lights, fog, and the engine's Reinhard curve.
 */
export class CombinePass implements IRendererPass {
  public readonly name: string = "combine";

  private readonly haze: SkyHaze;
  private readonly backdrop: Color = new Color();
  private readonly targets: RendererTargets;
  private readonly uniforms: RendererUniforms;
  private draw: FullScreenDraw;
  private ambientOcclusion: Nullable<Texture> = null;

  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.targets = targets;
    this.uniforms = uniforms;
    // Before the material, which is built sampling the haze maps it points the sky at.
    this.haze = new SkyHaze({
      clouds: uniforms.clouds,
      engine: uniforms.engine,
      scale: uniforms.exposure.scale,
      sky: uniforms.sky,
    });
    this.draw = this.createDraw();
  }

  /**
   * @param ambientOcclusion - The screen's occlusion combine multiplies the hemisphere and ambient by from now on, or none.
   */
  public setAmbientOcclusion(ambientOcclusion: Nullable<Texture>): void {
    if (ambientOcclusion === this.ambientOcclusion) {
      return;
    }

    this.ambientOcclusion = ambientOcclusion;
    this.draw.dispose();
    this.draw = this.createDraw();
  }

  /** The haze too, which draws whenever the sky is hazed. */
  public listPipelines(pipelines: IRendererPipelines): void {
    this.haze.listPipelines(pipelines);
    pipelines.draw(this.draw);
  }

  public render({ renderer, settings }: IRendererFrame): void {
    if (settings.isSkyDrawn && settings.isSkyHazed) {
      this.haze.render(renderer);
    }

    // The hex is bytes the page shows, so it reaches the canvas as written rather than decoded from srgb.
    this.backdrop.setHex(settings.backdrop ?? 0, LinearSRGBColorSpace);
    renderer.setClearColor(this.backdrop, settings.backdrop === null ? 0 : 1);
    drawCleared(renderer, true, false, () => this.draw.render(renderer));
  }

  public dispose(): void {
    this.haze.dispose();
    this.draw.dispose();
  }

  private createDraw(): FullScreenDraw {
    return new FullScreenDraw(
      createQuadMaterial(
        toCombinePassFragment(this.targets, this.targets.light.texture, this.uniforms, this.ambientOcclusion)
      ),
      this.targets.scene
    );
  }
}
