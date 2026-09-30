import { ComputeNode, WebGPURenderer } from "three/webgpu";

/** Three's compile of compute kernels, which its types leave out. */
interface IComputeCompiler {
  compileComputeAsync(kernels: Array<ComputeNode>): Promise<void>;
}

/**
 * Builds compute kernels' pipelines as three dispatches them, each made asynchronously rather than on the thread
 * drawing the window: three's `compileComputeAsync`, one kernel after another.
 *
 * @param renderer - The renderer the kernels dispatch on.
 * @param kernels - What compiles.
 * @returns Settles once every pipeline is built.
 */
export function compileComputeAsync(renderer: WebGPURenderer, kernels: ReadonlyArray<ComputeNode>): Promise<void> {
  return (renderer as unknown as IComputeCompiler).compileComputeAsync([...kernels]);
}
