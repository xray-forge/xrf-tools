import { RenderTarget } from "three/webgpu";

import { drawCleared } from "#/pass/cleared-draw";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { toSunPassFragment } from "#/pass/sun-pass.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * The sun, accumulated as `accum_sun` does: `Ldynamic_color * plight_infinity(m, P, N, L)`, times its shadow.
 */
export class SunPass implements IRendererPass {
  public readonly name: string = "sun";

  private readonly draw: FullScreenDraw;

  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.draw = new FullScreenDraw(
      createQuadMaterial(
        toSunPassFragment(
          targets,
          uniforms,
          targets.shadows.map((target: RenderTarget) => target.depthTexture as NonNullable<typeof target.depthTexture>)
        )
      ),
      targets.light
    );
  }

  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.draw(this.draw);
  }

  public render({ renderer }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    drawCleared(renderer, true, false, () => this.draw.render(renderer));
  }

  public dispose(): void {
    this.draw.dispose();
  }
}
