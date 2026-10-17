import { ERenderOverlay, RenderOverlay, RenderOverlaySegment } from "@/core/ipc/types/xrf-renderer";
import { IRenderLines } from "@/core/render/lib/scene/render-grid-lines";

/** Floats a segment takes in each of a line set's arrays: three a vertex, two vertices a segment. */
const FLOATS_PER_SEGMENT: number = 6;

/**
 * @param lines - Line segments with a colour at every end.
 * @param isDepthTested - Whether what the scene draws in front hides them.
 * @returns The same, as a native viewport draws them.
 */
export function toNativeLines(lines: IRenderLines, isDepthTested: boolean): RenderOverlay {
  const segments: Array<RenderOverlaySegment> = [];

  for (let at: number = 0; at < lines.positions.length; at += FLOATS_PER_SEGMENT) {
    segments.push({
      colors: [
        [lines.colors[at], lines.colors[at + 1], lines.colors[at + 2]],
        [lines.colors[at + 3], lines.colors[at + 4], lines.colors[at + 5]],
      ],
      ends: [
        [lines.positions[at], lines.positions[at + 1], lines.positions[at + 2]],
        [lines.positions[at + 3], lines.positions[at + 4], lines.positions[at + 5]],
      ],
    });
  }

  return { isDepthTested, kind: ERenderOverlay.LINES, segments };
}
