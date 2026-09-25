import { RenderTarget, WebGPURenderer } from "three/webgpu";

/** The part of three's renderer that remembers whether a target's depth was cleared yet. */
interface IRendererTargetData {
  _textures: { get: (target: RenderTarget) => { depthInitialized?: boolean } };
}

/**
 * Allocates a target that borrows the G-buffer's depth. Three clears a target's depth on the first draw into it when
 * the renderer does not clear on its own, which for a borrowed depth erases what the G-buffer drew that frame.
 *
 * @param renderer - The renderer the target is drawn by.
 * @param target - The target, its depth the G-buffer's.
 */
export function initBorrowedDepthTarget(renderer: WebGPURenderer, target: RenderTarget): void {
  renderer.initRenderTarget(target);
  (renderer as unknown as IRendererTargetData)._textures.get(target).depthInitialized = true;
}
