import { RenderFrameReport } from "@xrf/ipc/types/xrf-renderer";

import { ILevelViewerImage } from "@/tools/level-viewer/lib/files/level-viewer-image";
import { ILevelViewerSnapshot } from "@/tools/level-viewer/lib/snapshot/level-viewer-snapshot";

/** A screenshot as the viewer holds it once taken: its image, and the view it was taken of. */
export interface ILevelViewerScreenshot extends ILevelViewerImage {
  /** The view as it stood for the frame saved, which its report holds. */
  view: ILevelViewerSnapshot;
}

/** A screenshot's report as written beside it: where, and what its frame cost. */
export interface ILevelViewerScreenshotReportFile {
  path: string;
  frame: RenderFrameReport;
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
