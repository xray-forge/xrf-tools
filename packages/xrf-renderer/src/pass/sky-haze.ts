import { HalfFloatType, NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { createColourTarget } from "#/pass/colour-target";
import { createQuadMaterial } from "#/pass/quad-material";
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
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh = new QuadMesh();

  /**
   * @param uniforms - What the sky is drawn with, whose haze map it points at what it draws.
   */
  public constructor(uniforms: ISkyWithCloudsUniforms) {
    this.material = createQuadMaterial(toSkyHazeFragment(uniforms));
    this.quad.material = this.material;
    uniforms.sky.haze.value = this.target.texture;
  }

  /**
   * @param renderer - The renderer drawing the frame.
   */
  public render(renderer: WebGPURenderer): void {
    const previous = renderer.getRenderTarget();

    renderer.setRenderTarget(this.target);
    this.quad.render(renderer);
    renderer.setRenderTarget(previous);
  }

  public dispose(): void {
    this.target.dispose();
    this.material.dispose();
  }

  private static createTarget(): RenderTarget {
    const target: RenderTarget = createColourTarget([{ name: "sky-haze", type: HalfFloatType }]);

    target.setSize(WIDTH, HEIGHT);

    return target;
  }
}
