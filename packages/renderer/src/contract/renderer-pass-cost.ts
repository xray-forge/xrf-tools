/**
 * What one pass of the frame cost on the GPU.
 */
export interface IRendererPassCost {
  /** The pass, as the frame names it. */
  name: string;
  /** Mean GPU milliseconds over the report window; zero while no timing has resolved. */
  gpuTime: number;
}
