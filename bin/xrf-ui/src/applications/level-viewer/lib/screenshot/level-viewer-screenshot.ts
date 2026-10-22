import { Nullable } from "@xrf/types";

import { ILevelViewerImage } from "@/applications/level-viewer/lib/files/level-viewer-image";
import { ILevelViewerSnapshot } from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { RenderFrameReport } from "@/core/ipc/types/xrf-renderer";

/** A screenshot as the viewer holds it once taken: its image, and the report beside it. */
export interface ILevelViewerScreenshot extends ILevelViewerImage {
  /** Its report beside it, once written: the view it was taken of and what its frame cost. */
  reportPath: Nullable<string>;
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
