import { ERendererOverlay, TRendererOverlay } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { ERenderOverlay, RenderOverlay } from "@/core/ipc/types/xrf-renderer";
import { IRenderLines } from "@/core/render/lib/scene/render-grid-lines";

/**
 * @param overlay - A helper as the viewers build it.
 * @returns The same, as a native viewport draws it; null for the kinds it does not draw.
 */
export function toNativeOverlay(overlay: TRendererOverlay): Nullable<RenderOverlay> {
  switch (overlay.kind) {
    case ERendererOverlay.LINES:
      return {
        colors: Array.from(overlay.colors),
        isDepthTested: overlay.isDepthTested,
        kind: ERenderOverlay.LINES,
        positions: Array.from(overlay.positions),
      };

    case ERendererOverlay.SUN:
      return { color: [...overlay.color], kind: ERenderOverlay.SUN, size: overlay.size };

    case ERendererOverlay.POINTS:
    case ERendererOverlay.SKELETON:
      return null;
  }
}

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
