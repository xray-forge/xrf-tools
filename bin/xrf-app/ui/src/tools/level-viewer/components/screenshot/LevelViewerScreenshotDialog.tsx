import { useInjection } from "@wirestate/react";
import { formatBytes, formatMilliseconds, formatNumber } from "@xrf/format";
import { RenderFrameReport } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { DialogHeader } from "@xrf/ui/dialog/header";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { PhotoCameraOutlinedIcon as PhotoCameraIcon } from "@xrf/ui/icons/photo-camera-outlined";
import { Button } from "@xrf/ui/material/button";
import { Dialog, DialogActions, DialogContent } from "@xrf/ui/material/dialog";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback, useId, useState } from "react";

import {
  ILevelViewerScreenshot,
  ILevelViewerScreenshotReportFile,
} from "@/tools/level-viewer/lib/screenshot/level-viewer-screenshot";
import { LevelViewerScreenshotService } from "@/tools/level-viewer/services/level-viewer-screenshot";

/** What a screenshot's frame cost, as its report holds it; null until the report is written or where it was not. */
function toFrameLine(frame: Nullable<RenderFrameReport>): Nullable<string> {
  if (!frame?.framesPerSecond) {
    return null;
  }

  return [
    `${formatNumber(frame.framesPerSecond, 1)} frames/s`,
    frame.frameTime === null ? null : `${formatMilliseconds(frame.frameTime)} a frame`,
    frame.cpuTime === null ? null : `CPU ${formatMilliseconds(frame.cpuTime)}`,
    frame.gpuTime === null ? null : `GPU ${formatMilliseconds(frame.gpuTime)}`,
    frame.passes.length ? `${frame.passes.length} passes timed` : null,
  ]
    .filter(Boolean)
    .join(" · ");
}

export interface ILevelViewerScreenshotDialogProps extends BaseComponentProps {
  shot: ILevelViewerScreenshot;
}

/**
 * A screenshot whole: its image, its file and size, what its frame cost, and what can be done with it, its deletion
 * confirmed in place.
 */
export function LevelViewerScreenshotDialog({
  "data-testid": dataTestId = "level-viewer-screenshot-dialog",
  id,
  className,
  shot,
}: ILevelViewerScreenshotDialogProps): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);
  const titleId: string = useId();
  const [isDeleteArmed, setDeleteArmed] = useState<boolean>(false);
  const report: Nullable<ILevelViewerScreenshotReportFile> = screenshotService.report.value;
  const frame: Nullable<string> = toFrameLine(report?.frame ?? null);

  const onArmDelete = useCallback(() => setDeleteArmed(true), []);

  const onDisarmDelete = useCallback(() => setDeleteArmed(false), []);

  const onDelete = useCallback(() => void screenshotService.remove(), [screenshotService]);

  const onReveal = useCallback(() => void screenshotService.reveal(), [screenshotService]);

  const onRevealReport = useCallback(() => void screenshotService.revealReport(), [screenshotService]);

  const onCopyImage = useCallback(() => void screenshotService.copyImage(), [screenshotService]);

  const onSaveAs = useCallback(() => void screenshotService.saveAs(), [screenshotService]);

  const onClose = useCallback(() => screenshotService.setShown(false), [screenshotService]);

  return (
    <Dialog
      data-testid={dataTestId}
      id={id}
      className={className}
      aria-labelledby={titleId}
      open={true}
      maxWidth={"lg"}
      fullWidth={true}
      onClose={onClose}
    >
      <DialogHeader
        title={"Screenshot"}
        titleId={titleId}
        icon={<PhotoCameraIcon fontSize={"small"} />}
        closeLabel={"Close the screenshot"}
        onClose={onClose}
      />

      <DialogContent className={"flex flex-col gap-3 px-dialog"}>
        <img
          className={"block max-h-[64vh] w-full rounded-surface bg-viewport-backdrop object-contain"}
          src={shot.url}
          alt={`Screenshot ${shot.name}`}
        />

        <div className={"flex flex-col gap-0.5"}>
          <Typography className={"monospace truncate"} variant={"body2"} title={shot.path}>
            {shot.name}
          </Typography>
          <Typography className={"text-text-secondary"} variant={"caption"}>
            {`${shot.width} × ${shot.height} pixels · ${formatBytes(shot.size)}`}
          </Typography>
          <Typography className={"text-text-secondary"} variant={"caption"}>
            {frame ?? (screenshotService.report.error ? "Its report was not written." : "Reading what the frame cost…")}
          </Typography>
        </div>
      </DialogContent>

      <DialogActions className={"px-dialog"}>
        {isDeleteArmed ? (
          <>
            <Typography className={"mr-auto text-error"} variant={"body2"}>
              Delete the screenshot and its report?
            </Typography>
            <Button onClick={onDisarmDelete}>Keep</Button>
            <Button color={"error"} variant={"contained"} onClick={onDelete}>
              Delete for good
            </Button>
          </>
        ) : (
          <>
            <Button className={"mr-auto"} color={"error"} onClick={onArmDelete}>
              Delete
            </Button>
            <Button onClick={onReveal}>Show in folder</Button>
            <Button disabled={!report} onClick={onRevealReport}>
              Show report
            </Button>
            <Button onClick={onCopyImage}>Copy image</Button>
            <Button variant={"contained"} onClick={onSaveAs}>
              Save as…
            </Button>
          </>
        )}
      </DialogActions>
    </Dialog>
  );
}
