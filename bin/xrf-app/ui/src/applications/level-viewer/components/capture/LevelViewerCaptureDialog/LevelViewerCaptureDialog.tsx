import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { DialogHeader } from "@xrf/ui/dialog/DialogHeader";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ChoiceFormRow } from "@xrf/ui/form";
import { MapOutlinedIcon as MapIcon, PanoramaPhotosphereOutlinedIcon as PanoramaPhotosphereIcon } from "@xrf/ui/icons";
import { Button, Dialog, DialogActions, DialogContent, Typography } from "@xrf/ui/material";
import { ReactElement, useCallback, useId, useState } from "react";

import { ELevelCaptureMode, LEVEL_CAPTURE_NAMES } from "@/applications/level-viewer/lib/capture/level-capture";
import { TLevelCapture } from "@/applications/level-viewer/lib/capture/level-captured";
import { ILevelMapParameters } from "@/applications/level-viewer/lib/map/level-map-capture";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/level-viewer-capture";
import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/level-viewer-capture-result";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";

import { LevelViewerCubeParameters } from "./LevelViewerCubeParameters";
import { LevelViewerCubeResult } from "./LevelViewerCubeResult";
import { LevelViewerMapParameters } from "./LevelViewerMapParameters";
import { LevelViewerMapResult } from "./LevelViewerMapResult";

/**
 * The level viewer's capture of a level map or a cubemap: what it captures and with what, then its progress while it
 * captures, then the picture with what can be done with it, its deletion confirmed in place. It stays open, and cannot
 * be closed, while a capture runs. Drawn only while open, so nothing of one opening is left for the next.
 */
export function LevelViewerCaptureDialog({
  "data-testid": dataTestId = "level-viewer-capture-dialog",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);
  const resultService: LevelViewerCaptureResultService = useInjection(LevelViewerCaptureResultService);
  const titleId: string = useId();
  const [isDeleteArmed, setDeleteArmed] = useState<boolean>(false);
  const mode: ELevelCaptureMode = captureService.mode;
  const mapParameters: Nullable<ILevelMapParameters> = captureService.mapParameters;
  const captured: Nullable<TLevelCapture> = resultService.captured;
  const job: Nullable<IJobState> = captureService.operation.job;
  const isCapturing: boolean = captureService.isCapturing;
  const isExporting: boolean = resultService.isExporting;
  const isMap: boolean = (captured?.mode ?? mode) === ELevelCaptureMode.MAP;
  const name: string = LEVEL_CAPTURE_NAMES[captured?.mode ?? mode];

  const onClose = useCallback(() => captureService.close(), [captureService]);

  const onMode = useCallback((value: ELevelCaptureMode) => captureService.setMode(value), [captureService]);

  const onCapture = useCallback(() => void captureService.capture(), [captureService]);

  const onStop = useCallback(() => captureService.stop(), [captureService]);

  const onBack = useCallback(() => resultService.back(), [resultService]);

  const onExport = useCallback(() => void resultService.exportAs(), [resultService]);

  const onReveal = useCallback(() => void resultService.reveal(), [resultService]);

  const onCopyImage = useCallback(() => void resultService.copyImage(), [resultService]);

  const onCopyPath = useCallback(() => void resultService.copyPath(), [resultService]);

  const onArmDelete = useCallback(() => setDeleteArmed(true), []);

  const onDisarmDelete = useCallback(() => setDeleteArmed(false), []);

  const onDelete = useCallback(() => {
    setDeleteArmed(false);
    void resultService.remove();
  }, [resultService]);

  return (
    <Dialog
      data-testid={dataTestId}
      id={id}
      className={className}
      aria-labelledby={titleId}
      open={true}
      maxWidth={captured ? "lg" : "md"}
      fullWidth={true}
      onClose={onClose}
    >
      <DialogHeader
        title={captured ? (isMap ? "Level map" : "Cubemap") : "Capture"}
        titleId={titleId}
        icon={isMap ? <MapIcon fontSize={"small"} /> : <PanoramaPhotosphereIcon fontSize={"small"} />}
        closeLabel={"Close the capture"}
        onClose={isCapturing || isExporting ? undefined : onClose}
      />

      <DialogContent className={"flex flex-col gap-4 px-dialog"}>
        {isCapturing ? (
          <div className={"flex flex-col gap-2"}>
            <Typography variant={"body2"}>
              {`Drawing the ${name} ${isMap ? "tile by tile" : "face by face"} in place of the viewport; the view ` +
                "comes back as it was once it ends."}
            </Typography>
            {job ? <JobProgressView job={job} onCancel={onStop} /> : null}
          </div>
        ) : captured ? (
          isMap ? (
            <LevelViewerMapResult map={captured.image} />
          ) : (
            <LevelViewerCubeResult cross={captured.image} />
          )
        ) : (
          <>
            <ChoiceFormRow
              label={"Capture"}
              options={[
                { value: ELevelCaptureMode.MAP, label: "Map" },
                { value: ELevelCaptureMode.CUBEMAP, label: "Cubemap" },
              ]}
              value={mode}
              onChange={onMode}
            />
            {mode === ELevelCaptureMode.CUBEMAP ? (
              <LevelViewerCubeParameters parameters={captureService.cubeParameters} />
            ) : mapParameters ? (
              <LevelViewerMapParameters parameters={mapParameters} />
            ) : null}
          </>
        )}
      </DialogContent>

      <DialogActions className={"px-dialog"}>
        {isCapturing ? (
          <Button disabled={job === null} onClick={onStop}>
            Stop
          </Button>
        ) : captured && isDeleteArmed ? (
          <>
            <Typography className={"mr-auto text-error"} variant={"body2"}>
              {`Delete the ${name}?`}
            </Typography>
            <Button onClick={onDisarmDelete}>Keep</Button>
            <Button color={"error"} variant={"contained"} onClick={onDelete}>
              Delete for good
            </Button>
          </>
        ) : captured ? (
          <>
            <Button className={"mr-auto"} color={"error"} disabled={isExporting} onClick={onArmDelete}>
              Delete
            </Button>
            <Button disabled={isExporting} onClick={onBack}>
              Back to parameters
            </Button>
            <Button onClick={onReveal}>Show in folder</Button>
            <Button onClick={onCopyPath}>Copy path</Button>
            <Button onClick={onCopyImage}>Copy image</Button>
            <Button variant={"contained"} disabled={isExporting} onClick={onExport}>
              {isExporting ? "Exporting…" : "Export…"}
            </Button>
          </>
        ) : (
          <>
            <Button onClick={onClose}>Close</Button>
            <Button variant={"contained"} disabled={!captureService.canCapture} onClick={onCapture}>
              Capture
            </Button>
          </>
        )}
      </DialogActions>
    </Dialog>
  );
}
