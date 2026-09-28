import { Nullable } from "@xrf/types";
import { Camera, Object3D, PerspectiveCamera, RenderTarget, WebGPURenderer } from "three/webgpu";

import { IFrameCompileTargets } from "#/graph/frame-compile-targets";
import { ISceneGrassStaging } from "#/scene/grass/scene-grass-staging";
import { RendererScene } from "#/scene/renderer-scene";
import { ISceneStaging } from "#/scene/staging/scene-staging";

/**
 * Compiles the materials waiting objects need, off the frame: three builds their pipelines asynchronously, and until
 * they are ready every waiting object keeps drawing what it drew before. One compile is in flight at a time, a batch's
 * passes one after another: three's asynchronous builds share its node state, and two interleaved corrupt bind groups.
 */
export class RendererSceneCompiler {
  private isCompilingBatch: boolean = false;
  private isDisposed: boolean = false;

  /** Whether a batch is compiling. */
  public get isCompiling(): boolean {
    return this.isCompilingBatch;
  }

  /**
   * @param renderer - The renderer drawing.
   * @param scene - The scene whose waiting objects compile.
   * @param targets - Where the frame draws them.
   * @param camera - The drawing camera.
   */
  public compile(
    renderer: WebGPURenderer,
    scene: RendererScene,
    targets: IFrameCompileTargets,
    camera: PerspectiveCamera
  ): void {
    if (this.isDisposed || this.isCompilingBatch) {
      return;
    }

    const grass: Nullable<ISceneGrassStaging> = scene.grass.takeStaged();

    if (grass) {
      return this.run(
        () => compileInto(renderer, targets.grass, grass.scene, camera),
        "Grass failed to compile:",
        (isCurrent: boolean) => (isCurrent ? grass.commit() : grass.abandon())
      );
    }

    const staging: Nullable<ISceneStaging> = scene.hasPending ? scene.stage() : null;

    if (!staging) {
      return;
    }

    this.run(
      async () => {
        for (const pass of targets.passes) {
          if (this.isDisposed) {
            return;
          }

          await compileInto(renderer, pass.target, staging.scenes[pass.scene], camera);
        }

        if (!this.isDisposed && staging.shadows.children.length) {
          await compileInto(renderer, targets.shadow.target, staging.shadows, targets.shadow.camera);
        }
      },
      "Materials failed to compile:",
      (isCurrent: boolean) => {
        if (isCurrent) {
          scene.commit(staging);
        }
      }
    );
  }

  /** Drops the batch in flight, and compiles nothing more. */
  public dispose(): void {
    this.isDisposed = true;
  }

  /**
   * @param batch - Compiles the batch, one compile after another.
   * @param failure - What a failure is logged under.
   * @param settle - Takes the batch, or lets it go where the compiler went first.
   */
  private run(batch: () => Promise<void>, failure: string, settle: (isCurrent: boolean) => void): void {
    this.isCompilingBatch = true;

    // A batch that failed settles too, its objects drawing what three makes of their materials: staged again, it would
    // fail the same way every frame, and nothing would ever settle.
    batch()
      .catch((error: unknown) => console.error(failure, error))
      .then(() => settle(!this.isDisposed))
      .finally(() => {
        this.isCompilingBatch = false;
      });
  }
}

/**
 * Compiles a scene's pipelines for a target. Three takes the target as the compile starts and builds each shader later
 * for the render context it took then, so the target current before is restored at once.
 *
 * @param renderer - The renderer drawing.
 * @param target - Where the scene draws.
 * @param scene - What compiles.
 * @param camera - What it draws with.
 * @returns Settles once every pipeline is built.
 */
function compileInto(renderer: WebGPURenderer, target: RenderTarget, scene: Object3D, camera: Camera): Promise<void> {
  const previous: Nullable<RenderTarget> = renderer.getRenderTarget();

  renderer.setRenderTarget(target);

  const compiled: Promise<void> = renderer.compileAsync(scene, camera);

  renderer.setRenderTarget(previous);

  return compiled;
}
