import { RenderTarget, WebGPURenderer } from "three/webgpu";

/** The part of three's renderer that remembers whether a target's depth was cleared yet. */
interface IRendererTargetData {
  _textures: { get: (target: RenderTarget) => { depthInitialized?: boolean } };
}

/**
 * Allocates a target whose depth another draw fills, which three would otherwise clear on the first draw into it when
 * the renderer does not clear on its own.
 *
 * @param renderer - The renderer the target is drawn by.
 * @param target - The target, its depth written before it draws: borrowed from the G-buffer, or a resolve's own.
 */
export function initPreservedDepthTarget(renderer: WebGPURenderer, target: RenderTarget): void {
  renderer.initRenderTarget(target);
  (renderer as unknown as IRendererTargetData)._textures.get(target).depthInitialized = true;
}
