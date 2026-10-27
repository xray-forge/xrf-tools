import { ELevelCaptureMode } from "@/applications/level-viewer/lib/capture/level-capture";
import { ILevelViewerImage } from "@/applications/level-viewer/lib/files/level-viewer-image";
import { ILevelMapRect } from "@/applications/level-viewer/lib/map/level-map-capture";

/** A capture written: its picture as the dialog shows it and the mode it was taken in, a map with the ground it covers. */
export type TLevelCapture =
  | { mode: ELevelCaptureMode.MAP; image: ILevelViewerImage; area: ILevelMapRect }
  | { mode: ELevelCaptureMode.CUBEMAP; image: ILevelViewerImage };
