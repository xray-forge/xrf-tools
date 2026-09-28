import { Nullable } from "@xrf/types";
import { Camera, PerspectiveCamera, RenderTarget, WebGPURenderer } from "three/webgpu";

import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { ISceneGrassStaging } from "#/scene/grass/scene-grass-staging";
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
   * @param grass - Where the grass draws, which its staged builds compile against.
   */
  public compile(
    renderer: WebGPURenderer,
    scene: RendererScene,
    passes: ReadonlyArray<IRendererScenePass>,
    camera: PerspectiveCamera,
    shadow: IRendererShadowCompile,
    grass: RenderTarget
  ): void {
    if (this.isCompilingBatch) {
      return;
    }

    const staged: Nullable<ISceneGrassStaging> = scene.grass.takeStaged();

    if (staged) {
      return this.compileGrass(renderer, staged, camera, grass);
    }

    if (!scene.hasPending) {
      return;
    }

    const staging: Nullable<ISceneStaging> = scene.stage();

    if (!staging) {
      return;
    }

    const generation: number = this.generation;

    this.isCompilingBatch = true;

    // Three builds a staged pipeline's shader after the compile returns, for whatever target is current then: left on
    // the one-channel shadow map, it gave every later shader one output, which a four-channel target refuses.
    const previous: Nullable<RenderTarget> = renderer.getRenderTarget();
    const compiles: Array<Promise<unknown>> = passes.map((pass: IRendererScenePass) => {
      renderer.setRenderTarget(pass.target);

      return renderer.compileAsync(staging.scenes[pass.scene], camera);
    });

    if (staging.shadows.children.length) {
      renderer.setRenderTarget(shadow.target);
      compiles.push(renderer.compileAsync(staging.shadows, shadow.camera));
    }

    renderer.setRenderTarget(previous);

    // A batch that failed commits too, its objects drawing what three makes of their materials: staged again, it would
    // fail the same way every frame, and nothing would ever settle.
    Promise.all(compiles)
      .catch((error: unknown) => console.error("Materials failed to compile:", error))
      .then(() => {
        if (generation === this.generation) {
          scene.commit(staging);
        }
      })
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

  /** Compiles a grass build as a batch of its own: three's asynchronous builds share its node state, one at a time. */
  private compileGrass(
    renderer: WebGPURenderer,
    staged: ISceneGrassStaging,
    camera: PerspectiveCamera,
    target: RenderTarget
  ): void {
    const generation: number = this.generation;
    const previous: Nullable<RenderTarget> = renderer.getRenderTarget();

    this.isCompilingBatch = true;
    renderer.setRenderTarget(target);

    const compiled: Promise<unknown> = renderer.compileAsync(staged.scene, camera);

    renderer.setRenderTarget(previous);

    compiled
      .catch((error: unknown) => console.error("Grass failed to compile:", error))
      .then(() => (generation === this.generation ? staged.commit() : staged.abandon()))
      .finally(() => {
        if (generation === this.generation) {
          this.isCompilingBatch = false;
        }
      });
  }
}
