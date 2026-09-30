import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { ComputeNode, NodeMaterial, RenderTarget, WebGPURenderer } from "three/webgpu";

import { FramePipelines } from "#/graph/frame-pipelines";
import { FullScreenDraw } from "#/pass/full-screen-draw";

interface IFakeRenderer {
  renderer: WebGPURenderer;
  /** What each compile started for, in order: a draw's target, or the kernels. */
  started: Array<Nullable<RenderTarget> | ReadonlyArray<ComputeNode>>;
  /** Settles the oldest compile still pending, or fails it. */
  finish(error?: Error): Promise<void>;
}

async function flush(): Promise<void> {
  for (let turn: number = 0; turn < 8; turn += 1) {
    await Promise.resolve();
  }
}

function createRenderer(): IFakeRenderer {
  const started: Array<Nullable<RenderTarget> | ReadonlyArray<ComputeNode>> = [];
  const pending: Array<{ resolve: () => void; reject: (error: Error) => void }> = [];
  let current: Nullable<RenderTarget> = null;

  function start(what: Nullable<RenderTarget> | ReadonlyArray<ComputeNode>): Promise<void> {
    started.push(what);

    return new Promise((resolve: () => void, reject: (error: Error) => void) => pending.push({ reject, resolve }));
  }

  const renderer = {
    _nodes: { getForCompute: (): void => {}, getForRender: (): void => {} },
    compileAsync: (): Promise<void> => start(current),
    compileComputeAsync: (kernels: ReadonlyArray<ComputeNode>): Promise<void> => start(kernels),
    getRenderTarget: (): Nullable<RenderTarget> => current,
    setRenderTarget: (target: Nullable<RenderTarget>): void => {
      current = target;
    },
  };

  return {
    finish: async (error?: Error): Promise<void> => {
      const next = pending.shift();

      if (error) {
        next?.reject(error);
      } else {
        next?.resolve();
      }

      await flush();
    },
    renderer: renderer as unknown as WebGPURenderer,
    started,
  };
}

function toDraw(target: Nullable<RenderTarget> = new RenderTarget()): FullScreenDraw {
  return new FullScreenDraw(new NodeMaterial(), target);
}

function toKernel(): ComputeNode {
  return { isComputeNode: true } as unknown as ComputeNode;
}

describe("FramePipelines", () => {
  it("waits for what was named until it compiled, and not for it again once it has", async () => {
    const fake: IFakeRenderer = createRenderer();
    const pipelines: FramePipelines = new FramePipelines();
    const draw: FullScreenDraw = toDraw();
    const kernels: Array<ComputeNode> = [toKernel()];

    pipelines.draw(draw);
    pipelines.compute(kernels);

    expect(pipelines.isWaiting).toBe(true);

    const compiled: Promise<void> = pipelines.compile(fake.renderer);

    // Still building, the frame would build it again as it draws.
    pipelines.begin();
    pipelines.draw(draw);
    pipelines.compute(kernels);

    expect(pipelines.isWaiting).toBe(true);

    await fake.finish();
    await fake.finish();
    await compiled;
    pipelines.begin();
    pipelines.draw(draw);
    pipelines.compute(kernels);

    expect(pipelines.isWaiting).toBe(false);
  });

  // One at a time, each waits for the one before's pipeline, and the frame for all of them in a row.
  it("starts every draw, each for its own target, and every kernel at once", () => {
    const fake: IFakeRenderer = createRenderer();
    const pipelines: FramePipelines = new FramePipelines();
    const draws: Array<FullScreenDraw> = [toDraw(), toDraw(null)];
    const kernels: Array<ComputeNode> = [toKernel(), toKernel()];

    draws.forEach((draw: FullScreenDraw) => pipelines.draw(draw));
    pipelines.compute(kernels.slice(0, 1));
    pipelines.compute(kernels);
    pipelines.compile(fake.renderer);

    expect(fake.started).toEqual([draws[0].target, null, [kernels[0]], [kernels[1]]]);
  });

  // Waiting again, a pipeline that failed would fail every frame, and the frame would never be drawn.
  it("counts a draw whose compile failed as compiled, and rejects with its failure once all settled", async () => {
    const fake: IFakeRenderer = createRenderer();
    const pipelines: FramePipelines = new FramePipelines();
    const [failing, next] = [toDraw(), toDraw()];

    pipelines.draw(failing);
    pipelines.draw(next);

    const failure: Promise<unknown> = pipelines.compile(fake.renderer).catch((error: unknown) => error);

    await fake.finish(new Error("refused"));
    pipelines.begin();
    pipelines.draw(failing);

    expect(pipelines.isWaiting).toBe(false);

    pipelines.draw(next);

    expect(pipelines.isWaiting).toBe(true);

    await fake.finish();

    expect(await failure).toEqual(new Error("refused"));
  });

  // Let go while three builds it, a pass's targets would be made again by the build it left behind.
  it("lets what a pass leaving holds go once the compile in flight ends, and at once otherwise", async () => {
    const fake: IFakeRenderer = createRenderer();
    const pipelines: FramePipelines = new FramePipelines();
    const early: jest.Mock<() => void> = jest.fn();
    const late: jest.Mock<() => void> = jest.fn();

    pipelines.draw(toDraw());
    pipelines.compile(fake.renderer);
    pipelines.retire(early);

    expect(early).not.toHaveBeenCalled();

    await fake.finish();

    expect(early).toHaveBeenCalledTimes(1);

    pipelines.retire(late);

    expect(late).toHaveBeenCalledTimes(1);
  });
});
