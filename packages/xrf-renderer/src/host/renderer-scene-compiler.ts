import { Maybe, Nullable } from "@xrf/types";
import { Camera, Object3D, PerspectiveCamera, RenderTarget, WebGPURenderer } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { ICompilingFrame } from "#/graph/compiling-frame";
import { IFrameCompileTargets } from "#/graph/frame-compile-targets";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererScene } from "#/scene/renderer-scene";
import { ISceneBuildStaging } from "#/scene/staging/scene-build-staging";
import { ISceneStaging } from "#/scene/staging/scene-staging";

/** What the scene builds apart from its objects, each compiled against its own target, in the order compiled. */
const STAGED_BUILDS = [
  { failure: "Grass failed to compile:", source: "grass" },
  { failure: "Rain failed to compile:", source: "rain" },
  { failure: "Thunder failed to compile:", source: "thunder" },
] as const;

/**
 * Compiles the materials waiting objects need, off the frame: three builds their pipelines asynchronously, and until
 * they are ready every waiting object keeps drawing what it drew before. One compile is in flight at a time, a batch's
 * passes one after another: three's asynchronous builds share its node state, and two interleaved corrupt bind groups.
 * A pass joining the frame is compiled first for what the scene draws already, then admitted.
 */
export class RendererSceneCompiler {
  private isCompilingBatch: boolean = false;
  private isDisposed: boolean = false;

  /** Whether a batch is compiling. */
  public get isCompiling(): boolean {
    return this.isCompilingBatch;
  }

  /**
   * Starts the next batch, if none is compiling: a staged build, grass first, else a pass joining, else the waiting
   * objects.
   *
   * @param renderer - The renderer drawing.
   * @param scene - The scene whose objects compile.
   * @param frame - Where the frame draws them, read again at each step of a batch: a pass that left the frame
   *   meanwhile is not compiled into its freed target, and one that joined is compiled too.
   * @param camera - The drawing camera.
   */
  public compile(
    renderer: WebGPURenderer,
    scene: RendererScene,
    frame: ICompilingFrame,
    camera: PerspectiveCamera
  ): void {
    if (this.isDisposed || this.isCompilingBatch) {
      return;
    }

    for (const { failure, source } of STAGED_BUILDS) {
      const staging: Nullable<ISceneBuildStaging> = scene[source].takeStaged();

      if (staging) {
        return this.run(
          async () => compileInto(renderer, frame.compileTargets[source], staging.scene, camera),
          failure,
          (isCurrent: boolean) => (isCurrent ? staging.commit() : staging.abandon())
        );
      }
    }

    if (!this.join(renderer, scene, frame, camera)) {
      this.compileWaiting(renderer, scene, frame, camera);
    }
  }

  /** Drops the batch in flight, and compiles nothing more. */
  public dispose(): void {
    this.isDisposed = true;
  }

  /**
   * Compiles the first pass joining the frame for what the scene draws already, and admits it; one with nothing to
   * compile, its pipelines built before, at once.
   *
   * @returns Whether a batch started.
   */
  private join(
    renderer: WebGPURenderer,
    scene: RendererScene,
    frame: ICompilingFrame,
    camera: PerspectiveCamera
  ): boolean {
    const pass: Maybe<IRendererScenePass> = frame.compileTargets.joining[0];

    if (!pass) {
      return false;
    }

    const staging: Nullable<ISceneStaging> = scene.stageDrawn();

    if (!staging?.scenes[pass.scene].children.length) {
      frame.admit(pass);

      return false;
    }

    this.run(
      async () => compileInto(renderer, pass.target, staging.scenes[pass.scene], camera),
      "A pass joining the frame failed to compile:",
      // Left or made again meanwhile, it is admitted to nothing, and what compiled stays compiled.
      (isCurrent: boolean) => {
        if (isCurrent) {
          scene.commit(staging, new Set([pass.scene]));
          frame.admit(pass);
        }
      }
    );

    return true;
  }

  /** Compiles the waiting objects for every scene pass the frame holds, then their shadow materials. */
  private compileWaiting(
    renderer: WebGPURenderer,
    scene: RendererScene,
    frame: ICompilingFrame,
    camera: PerspectiveCamera
  ): void {
    const staging: Nullable<ISceneStaging> = scene.hasPending ? scene.stage() : null;

    if (!staging) {
      return;
    }

    // Each pass as it is tried: one that failed is taken as compiled, and one never reached is left for another batch.
    const compiled: Set<ERendererPass> = new Set();

    this.run(
      async () => {
        const tried: Set<IRendererScenePass> = new Set();

        function next(): Maybe<IRendererScenePass> {
          return frame.compileTargets.passes.find((pass: IRendererScenePass) => !tried.has(pass));
        }

        for (let pass: Maybe<IRendererScenePass> = next(); pass && !this.isDisposed; pass = next()) {
          tried.add(pass);
          compiled.add(pass.scene);
          await compileInto(renderer, pass.target, staging.scenes[pass.scene], camera);
        }

        if (!this.isDisposed && staging.shadows.children.length) {
          const { shadow }: IFrameCompileTargets = frame.compileTargets;

          await compileInto(renderer, shadow.target, staging.shadows, shadow.camera);
        }
      },
      "Materials failed to compile:",
      (isCurrent: boolean) => {
        if (isCurrent) {
          scene.commit(staging, compiled);
        }
      }
    );
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
      .catch((error: unknown) => {
        // One the host dropped fails as the renderer goes, which says nothing about its materials.
        if (!this.isDisposed) {
          console.error(failure, error);
        }
      })
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
