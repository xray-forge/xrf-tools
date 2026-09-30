import { ComputeNode, WebGPURenderer } from "three/webgpu";

import { compileComputeAsync } from "#/internals/compute-compile";
import { compileSideBySide } from "#/internals/side-by-side-compile";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { IRendererPipelines } from "#/pass/renderer-pipelines";

/**
 * The pipelines of the frame's own passes and which of them compiled. The passes name theirs before every frame, and
 * the frame is drawn only once nothing named waits: the compile lane builds what waits off the frame, where three makes
 * each pipeline asynchronously rather than on the thread drawing the window, all of them at once. A draw or kernel
 * compiles once; one whose compile failed counts as compiled too, since it would fail the same way again.
 */
export class FramePipelines implements IRendererPipelines {
  private readonly compiled: WeakSet<FullScreenDraw | ComputeNode> = new WeakSet();
  /** What the passes named for the next frame that has not compiled. */
  private readonly draws: Set<FullScreenDraw> = new Set();
  private readonly kernels: Set<ComputeNode> = new Set();
  /** The compile in flight, which what a pass leaving the frame holds is let go after. */
  private compiling: Promise<void> = Promise.resolve();
  private isCompilingNow: boolean = false;

  /** Whether anything named for the next frame waits to compile. */
  public get isWaiting(): boolean {
    return this.draws.size > 0 || this.kernels.size > 0;
  }

  /** Forgets what the passes named for the frame before, for them to name the next frame's. */
  public begin(): void {
    this.draws.clear();
    this.kernels.clear();
  }

  public draw(draw: FullScreenDraw): void {
    if (!this.compiled.has(draw)) {
      this.draws.add(draw);
    }
  }

  public compute(kernels: ReadonlyArray<ComputeNode>): void {
    for (const kernel of kernels) {
      if (!this.compiled.has(kernel)) {
        this.kernels.add(kernel);
      }
    }
  }

  /**
   * Compiles everything that waits side by side: each draw for its target and each kernel, their shaders built one
   * after another and their pipelines made together, which a compile at a time would wait for one by one.
   *
   * @param renderer - The renderer drawing.
   * @returns Settles once everything that waited compiled, or rejects with the first failure once all settled.
   */
  public compile(renderer: WebGPURenderer): Promise<void> {
    this.isCompilingNow = true;
    this.compiling = this.compileWaiting(renderer).finally(() => {
      this.isCompilingNow = false;
    });

    return this.compiling;
  }

  /**
   * @param release - Lets go what a pass leaving the frame holds: at once, or once the compile in flight ends, since
   *   three may be building it still.
   */
  public retire(release: () => void): void {
    if (this.isCompilingNow) {
      this.compiling.then(release, release);
    } else {
      release();
    }
  }

  /** Forgets what waits; a compile in flight runs to its end. */
  public dispose(): void {
    this.begin();
  }

  private compileWaiting(renderer: WebGPURenderer): Promise<void> {
    return compileSideBySide(renderer, [
      ...[...this.draws].map((draw: FullScreenDraw) => () => draw.compile(renderer).finally(() => this.settle(draw))),
      ...[...this.kernels].map(
        (kernel: ComputeNode) => () => compileComputeAsync(renderer, [kernel]).finally(() => this.settle(kernel))
      ),
    ]);
  }

  /** Counts a pipeline compiled once its compile settled: until then the frame waits for it, drawing nothing of it. */
  private settle(pipeline: FullScreenDraw | ComputeNode): void {
    this.compiled.add(pipeline);

    if (pipeline instanceof FullScreenDraw) {
      this.draws.delete(pipeline);
    } else {
      this.kernels.delete(pipeline);
    }
  }
}
