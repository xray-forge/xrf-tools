import { Nullable } from "@xrf/types";
import { Scene } from "three/webgpu";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { SceneGrass } from "#/scene/grass/scene-grass";

/**
 * The grass, as `Details->Render` draws it into the G-buffer after everything else it holds: planted around the view
 * on the GPU, then drawn a model at a time into the targets the G-buffer pass cleared. In the frame only while the
 * grass is on, so off it costs nothing.
 */
export class GrassPass implements IRendererPass {
  public readonly name: string = "grass";

  private readonly grass: SceneGrass;
  private readonly targets: RendererTargets;

  /**
   * @param grass - The level's grass, which may be none.
   * @param targets - What the frame draws into.
   */
  public constructor(grass: SceneGrass, targets: RendererTargets) {
    this.grass = grass;
    this.targets = targets;
  }

  public render({ renderer, camera, viewCamera, settings }: IRendererFrame): void {
    if (!this.grass.isReady) {
      return;
    }

    // Planted around the view unjittered, so the jitter never moves which cells are planted; drawn as the scene draws.
    const scene: Nullable<Scene> = this.grass.plant(renderer, viewCamera, settings.features.grass);

    if (scene) {
      renderer.setRenderTarget(this.targets.gbuffer);
      renderer.render(scene, camera);
    }
  }

  /** The grass is the scene's, which lets it go with the rest of what the consumer put. */
  public dispose(): void {}
}
