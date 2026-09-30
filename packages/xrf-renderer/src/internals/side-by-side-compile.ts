import { WebGPURenderer } from "three/webgpu";

/** The node manager as a compile reaches it: each shader built at once, or yielding part way through. */
interface INodeBuilds {
  getForRender(renderObject: unknown): unknown;
  getForCompute(computeNode: unknown): unknown;
  getForRenderAsync?: (renderObject: unknown) => Promise<unknown>;
  getForComputeAsync?: (computeNode: unknown) => Promise<unknown>;
}

/**
 * Runs compiles side by side, their pipelines made together on the browser's threads, while three builds every shader
 * whole, at once: a build that yields part way lets another run inside it, and two interleaved corrupt the node state
 * they share. Each compile's builds run as it reaches them, so only its pipelines are awaited together. One batch at a
 * time, in the compile lane alone.
 *
 * @param renderer - The renderer compiling.
 * @param compiles - Each starts one compile, all started at once and in order.
 * @returns Settles once every compile settled, or rejects with the first failure once they all have.
 */
export async function compileSideBySide(
  renderer: WebGPURenderer,
  compiles: ReadonlyArray<() => Promise<void>>
): Promise<void> {
  const nodes: INodeBuilds = (renderer as unknown as { _nodes: INodeBuilds })._nodes;

  nodes.getForRenderAsync = (renderObject: unknown): Promise<unknown> =>
    Promise.resolve(nodes.getForRender(renderObject));
  nodes.getForComputeAsync = (computeNode: unknown): Promise<unknown> =>
    Promise.resolve(nodes.getForCompute(computeNode));

  try {
    const settled: Array<PromiseSettledResult<void>> = await Promise.allSettled(
      compiles.map((compile: () => Promise<void>) => compile())
    );
    const failure: PromiseSettledResult<void> | undefined = settled.find(
      (result: PromiseSettledResult<void>) => result.status === "rejected"
    );

    if (failure) {
      throw (failure as PromiseRejectedResult).reason;
    }
  } finally {
    // The prototype's own, yielding again.
    delete nodes.getForRenderAsync;
    delete nodes.getForComputeAsync;
  }
}
