import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { SceneGrass } from "#/scene/grass/scene-grass";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

/**
 * The grass, as `Details->Render` draws it into the G-buffer after everything else it holds: planted around the
 * camera on the GPU, then drawn a model at a time into the targets the G-buffer pass cleared. In the frame only while
 * the grass is on, so off it costs nothing.
 */
export class GrassPass implements IRendererPass {
  public readonly name: string = "grass";

  private readonly grass: SceneGrass;
  private readonly targets: RendererTargets;
  private readonly uniforms: GrassUniforms = new GrassUniforms();

  /**
   * @param grass - The level's grass, which may be none.
   * @param targets - What the frame draws into.
   */
  public constructor(grass: SceneGrass, targets: RendererTargets) {
    this.grass = grass;
    this.targets = targets;
  }

  public render({ renderer, camera, settings }: IRendererFrame): void {
    const grid = this.grass.grid;

    if (!this.grass.isReady || !grid) {
      return;
    }

    this.uniforms.configure(settings.features.grass);
    this.uniforms.follow(camera, grid.sizeX, grid.sizeZ, grid.offsetX, grid.offsetZ);
    this.grass.plant(renderer, this.uniforms);
    renderer.setRenderTarget(this.targets.gbuffer);
    renderer.render(this.grass.scene, camera);
  }

  public dispose(): void {}
}
