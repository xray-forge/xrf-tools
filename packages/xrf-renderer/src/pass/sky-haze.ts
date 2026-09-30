import { HalfFloatType, RenderTarget, WebGPURenderer } from "three/webgpu";

import { createColourTarget } from "#/pass/colour-target";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { toSkyHazeFragment } from "#/shader/sky-haze.tsl";
import { ISkyWithCloudsUniforms } from "#/shader/sky-with-clouds-uniforms";

/** The haze map's texels across, one a bearing, and down, one a height. */
const WIDTH: number = 64;
const HEIGHT: number = 32;

/**
 * The sky as drawn, clouds and all, blurred into the haze map the distance fades into, drawn again every frame it is
 * read, which costs a few thousand texels: the skies blend as the clock moves and the clouds drift.
 */
export class SkyHaze {
  private readonly target: RenderTarget = SkyHaze.createTarget();
  private readonly draw: FullScreenDraw;

  /**
   * @param uniforms - What the sky is drawn with, whose haze map it points at what it draws.
   */
  public constructor(uniforms: ISkyWithCloudsUniforms) {
    this.draw = new FullScreenDraw(createQuadMaterial(toSkyHazeFragment(uniforms)), this.target);
    uniforms.sky.haze.value = this.target.texture;
  }

  /**
   * @param pipelines - Where the pass drawing it names what it draws with.
   */
  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.draw(this.draw);
  }

  /**
   * @param renderer - The renderer drawing the frame.
   */
  public render(renderer: WebGPURenderer): void {
    const previous = renderer.getRenderTarget();

    this.draw.render(renderer);
    renderer.setRenderTarget(previous);
  }

  public dispose(): void {
    this.target.dispose();
    this.draw.dispose();
  }

  private static createTarget(): RenderTarget {
    const target: RenderTarget = createColourTarget([{ name: "sky-haze", type: HalfFloatType }]);

    target.setSize(WIDTH, HEIGHT);

    return target;
  }
}
