import { Nullable } from "@xrf/types";
import { PerspectiveCamera, WebGPURenderer } from "three/webgpu";

import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererScene } from "#/scene/renderer-scene";
import { ISceneStaging } from "#/scene/staging/scene-staging";

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
   */
  public compile(
    renderer: WebGPURenderer,
    scene: RendererScene,
    passes: ReadonlyArray<IRendererScenePass>,
    camera: PerspectiveCamera
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
