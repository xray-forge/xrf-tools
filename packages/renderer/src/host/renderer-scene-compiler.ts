import { Nullable } from "@xrf/types";
import { Camera, PerspectiveCamera, RenderTarget, WebGPURenderer } from "three/webgpu";

import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererScene } from "#/scene/renderer-scene";
import { ISceneStaging } from "#/scene/staging/scene-staging";

/** Where the shadow materials compile: a cascade's target, and a cascade's camera, as the shadow passes draw them. */
export interface IRendererShadowCompile {
  target: RenderTarget;
  camera: Camera;
}

/**
 * Compiles the materials waiting objects need, off the frame: three builds their pipelines asynchronously, and until
 * they are ready every waiting object keeps drawing what it drew before. One batch is in flight at a time.
 */
export class RendererSceneCompiler {
  /** Bumped by every reset, so a batch finishing late can tell it was superseded. */
  private generation: number = 0;
  private isCompilingBatch: boolean = false;

  /** Whether a batch is compiling. */
  public get isCompiling(): boolean {
    return this.isCompilingBatch;
  }

  /**
   * @param renderer - The renderer drawing.
   * @param scene - The scene whose waiting objects compile.
   * @param passes - The passes drawing its scenes, each compiled against the target it draws into.
   * @param camera - The drawing camera.
   * @param shadow - Where the shadow materials compile.
   */
  public compile(
    renderer: WebGPURenderer,
    scene: RendererScene,
    passes: ReadonlyArray<IRendererScenePass>,
    camera: PerspectiveCamera,
    shadow: IRendererShadowCompile
  ): void {
    if (this.isCompilingBatch || !scene.hasPending) {
      return;
    }

    const staging: Nullable<ISceneStaging> = scene.stage();

    if (!staging) {
      return;
    }

    const generation: number = this.generation;

    this.isCompilingBatch = true;

    const compiles: Array<Promise<unknown>> = passes.map((pass: IRendererScenePass) => {
      renderer.setRenderTarget(pass.target);

      return renderer.compileAsync(staging.scenes[pass.scene], camera);
    });

    if (staging.shadows.children.length) {
      renderer.setRenderTarget(shadow.target);
      compiles.push(renderer.compileAsync(staging.shadows, shadow.camera));
    }

    Promise.all(compiles)
      .then(() => {
        if (generation === this.generation) {
          scene.commit(staging);
        }
      })
      .catch((error: unknown) => console.error("Materials failed to compile:", error))
      .finally(() => {
        if (generation === this.generation) {
          this.isCompilingBatch = false;
        }
      });
  }

  /** Drops the batch in flight, for a device that went away. */
  public reset(): void {
    this.generation += 1;
    this.isCompilingBatch = false;
  }
}
