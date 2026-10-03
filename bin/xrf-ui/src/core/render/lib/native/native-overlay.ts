import { ERenderOverlay, RenderOverlay } from "@/core/ipc/types/xrf-renderer";
import { IRenderLines } from "@/core/render/lib/scene/render-grid-lines";

/**
 * @param lines - Line segments with a colour at every end.
 * @param isDepthTested - Whether what the scene draws in front hides them.
 * @returns The same, as a native viewport draws them.
 */
export function toNativeLines(lines: IRenderLines, isDepthTested: boolean): RenderOverlay {
  return {
    colors: Array.from(lines.colors),
    isDepthTested,
    kind: ERenderOverlay.LINES,
    positions: Array.from(lines.positions),
  };
}
