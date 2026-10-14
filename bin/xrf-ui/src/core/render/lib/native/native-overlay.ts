import { ERenderOverlay, RenderOverlay, RenderOverlaySegment } from "@/core/ipc/types/xrf-renderer";

/**
 * @param segments - Line segments with a colour at every end.
 * @param isDepthTested - Whether what the scene draws in front hides them.
 * @returns The same, as a native viewport draws them.
 */
export function toNativeLines(segments: Array<RenderOverlaySegment>, isDepthTested: boolean): RenderOverlay {
  return { isDepthTested, kind: ERenderOverlay.LINES, segments };
}
