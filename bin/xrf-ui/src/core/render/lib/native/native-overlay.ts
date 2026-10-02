import { ERendererOverlay, TRendererOverlay } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { ERenderOverlay, RenderOverlay } from "@/core/ipc/types/xrf-renderer";

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
