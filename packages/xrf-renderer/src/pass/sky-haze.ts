import { HalfFloatType, NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { createColourTarget } from "#/pass/colour-target";
import { createQuadMaterial } from "#/pass/quad-material";
import { toSkyHazeFragment } from "#/shader/sky-haze.tsl";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

/** A haze map's texels across, one a bearing, and down, one a height. */
const WIDTH: number = 64;
const HEIGHT: number = 32;

/**
 * Both skies blurred into haze maps the distance fades into, drawn again every frame they are read, which costs a few
 * thousand texels: the skies change as they upload, and the pair changes as the clock moves.
 */
export class SkyHaze {
  private readonly targets: readonly [RenderTarget, RenderTarget];
  private readonly materials: readonly [NodeMaterial, NodeMaterial];
  private readonly quad: QuadMesh = new QuadMesh();

  /**
   * @param sky - The skies, whose haze maps it points at what it draws.
   */
  public constructor(sky: SkyUniforms) {
    this.targets = [SkyHaze.createTarget(), SkyHaze.createTarget()];
    this.materials = [
      createQuadMaterial(toSkyHazeFragment(sky.cubes[0])),
      createQuadMaterial(toSkyHazeFragment(sky.cubes[1])),
    ];
    sky.hazes[0].value = this.targets[0].texture;
    sky.hazes[1].value = this.targets[1].texture;
  }

  /**
   * @param renderer - The renderer drawing the frame.
   */
  public render(renderer: WebGPURenderer): void {
    const previous = renderer.getRenderTarget();

    this.targets.forEach((target: RenderTarget, index: number) => {
      renderer.setRenderTarget(target);
      this.quad.material = this.materials[index];
      this.quad.render(renderer);
    });
    renderer.setRenderTarget(previous);
  }

  public dispose(): void {
    this.targets.forEach((target: RenderTarget) => target.dispose());
    this.materials.forEach((material: NodeMaterial) => material.dispose());
  }

  private static createTarget(): RenderTarget {
    const target: RenderTarget = createColourTarget([{ name: "sky-haze", type: HalfFloatType }]);

    target.setSize(WIDTH, HEIGHT);

    return target;
  }
}
