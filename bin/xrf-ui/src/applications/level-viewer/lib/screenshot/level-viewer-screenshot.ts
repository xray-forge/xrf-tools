import { Nullable } from "@xrf/types";

import { ILevelViewerSnapshot } from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { RenderFrameReport } from "@/core/ipc/types/xrf-renderer";

/** A screenshot as the viewer holds it once taken: its file, its size, and its pixels for the page to show. */
export interface ILevelViewerScreenshot {
  path: string;
  /** Its report beside it: the view it was taken of and what its frame cost. */
  reportPath: string;
  name: string;
  width: number;
  height: number;
  /** Bytes the PNG takes. */
  size: number;
  /** An object URL of the PNG, let go once the screenshot is replaced or deleted. */
  url: string;
  /** What its frame cost, once its report is written; null until then or where it could not be written. */
  frame: Nullable<RenderFrameReport>;
}

/** A screenshot's report: the view as File › Export view writes it, the image it goes with and its frame's cost. */
export interface ILevelViewerScreenshotReport extends ILevelViewerSnapshot {
  screenshot: {
    file: string;
    width: number;
    height: number;
    /** Whether the GPU timing was switched on for the screenshot alone, so the frame's passes are timed. */
    isTimingSwitched: boolean;
  };
  frame: RenderFrameReport;
}

/** Where a PNG names its width and height: big-endian words after its signature and its header's length and type. */
const PNG_WIDTH_OFFSET: number = 16;

/**
 * @param path - A screenshot's PNG.
 * @returns Its report's path beside it.
 */
export function toLevelViewerScreenshotReportPath(path: string): string {
  return `${path.slice(0, path.length - EXrayExtension.PNG.length)}${EXrayExtension.JSON}`;
}

/**
 * @param bytes - A PNG file.
 * @returns Its width and height, from its header.
 */
export function readPngSize(bytes: Uint8Array): [number, number] {
  const view: DataView = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);

  return [view.getUint32(PNG_WIDTH_OFFSET), view.getUint32(PNG_WIDTH_OFFSET + 4)];
}
