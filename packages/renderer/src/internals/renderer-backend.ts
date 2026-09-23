import { WebGPURenderer } from "three/webgpu";

/**
 * The part of three's backend the renderer reads, which its typings do not state.
 */
export interface IRendererBackend {
  isWebGPUBackend?: boolean;
  device?: { features?: Iterable<string>; adapterInfo?: { vendor?: string; architecture?: string } };
  hasTimestampQuery?(uid: string): boolean;
  getTimestamp?(uid: string): number;
}

/**
 * @param renderer - A renderer.
 * @returns Its backend, as far as it is read.
 */
export function getRendererBackend(renderer: WebGPURenderer): IRendererBackend {
  return renderer.backend as unknown as IRendererBackend;
}
