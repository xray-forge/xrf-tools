import { useInjection } from "@wirestate/react";
import { RecordingOutcome } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DeleteOutlinedIcon as DeleteIcon } from "@xrf/ui/icons/delete-outlined";
import { FolderOpenOutlinedIcon as FolderOpenIcon } from "@xrf/ui/icons/folder-open-outlined";
import { LinkIcon } from "@xrf/ui/icons/link";
import { VideocamIcon } from "@xrf/ui/icons/videocam";
import { IconButton } from "@xrf/ui/material/icon-button";
import { Paper } from "@xrf/ui/material/paper";
import { Tooltip } from "@xrf/ui/material/tooltip";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { LevelViewerRecordingReadout } from "@/tools/level-viewer/lib/recording/level-viewer-recording-readout";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";

/** Milliseconds a recording's toast stays once its pointer leaves it, as a screenshot's does. */
const TOAST_LIFETIME: number = 6000;

/**
 * The recording just written, in the viewport's corner for a few seconds and while the pointer is over it, as a
 * screenshot's is: its name, length, size and what it dropped, its folder, its path, and its deletion, which a second
 * click confirms.
 */
export function LevelViewerRecordingToast({
  "data-testid": dataTestId = "level-viewer-recording-toast",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const recordingService: LevelViewerRecordingService = useInjection(LevelViewerRecordingService);
  const outcome: Nullable<RecordingOutcome> = recordingService.outcome;
  const [shownFile, setShownFile] = useState<Nullable<string>>(null);
  const [isHovered, setHovered] = useState<boolean>(false);
  const [isDeleteArmed, setDeleteArmed] = useState<boolean>(false);

  const onReveal = useCallback(() => void recordingService.reveal(), [recordingService]);

  const onCopyPath = useCallback(() => void recordingService.copyPath(), [recordingService]);

  const onDelete = useCallback(() => {
    if (!isDeleteArmed) {
      setDeleteArmed(true);

      return;
    }

    void recordingService.remove();
  }, [isDeleteArmed, recordingService]);

  const onEnter = useCallback(() => setHovered(true), []);

  const onLeave = useCallback(() => {
    setHovered(false);
    setDeleteArmed(false);
  }, []);

  useEffect(() => {
    setShownFile(outcome?.file ?? null);
    setDeleteArmed(false);
  }, [outcome]);

  useEffect(() => {
    if (shownFile === null || isHovered) {
      return;
    }

    const timeout: ReturnType<typeof setTimeout> = setTimeout(() => setShownFile(null), TOAST_LIFETIME);

    return () => clearTimeout(timeout);
  }, [isHovered, shownFile]);

  if (!outcome || shownFile !== outcome.file) {
    return <></>;
  }

  return (
    <Paper
      data-testid={dataTestId}
      id={id}
      className={cn("absolute bottom-2 left-2 z-20 flex w-96 items-center gap-2 p-1.5", className)}
      elevation={8}
      role={"status"}
      aria-label={"Recording saved"}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
    >
      <div className={"flex h-14 w-24 shrink-0 items-center justify-center rounded-surface bg-well"}>
        <VideocamIcon className={"text-text-secondary"} />
      </div>

      <div className={"flex min-w-0 grow flex-col"}>
        <Typography variant={"body2"}>Recording saved</Typography>
        <Typography className={"monospace truncate text-text-secondary"} variant={"caption"} title={outcome.file}>
          {LevelViewerRecordingReadout.getFileName(outcome.file)}
        </Typography>
        <Typography
          className={LevelViewerRecordingReadout.hasDrops(outcome) ? "text-warning" : "text-text-secondary"}
          variant={"caption"}
        >
          {LevelViewerRecordingReadout.describeOutcome(outcome)}
        </Typography>
      </div>

      <div className={"flex shrink-0"}>
        <Tooltip title={"Show in folder"}>
          <IconButton size={"small"} aria-label={"Show in folder"} onClick={onReveal}>
            <FolderOpenIcon fontSize={"small"} />
          </IconButton>
        </Tooltip>
        <Tooltip title={"Copy path"}>
          <IconButton size={"small"} aria-label={"Copy path"} onClick={onCopyPath}>
            <LinkIcon fontSize={"small"} />
          </IconButton>
        </Tooltip>
        <Tooltip title={isDeleteArmed ? "Click again to delete it for good" : "Delete"}>
          <IconButton
            size={"small"}
            color={isDeleteArmed ? "error" : "default"}
            aria-label={isDeleteArmed ? "Delete for good" : "Delete"}
            onClick={onDelete}
          >
            <DeleteIcon fontSize={"small"} />
          </IconButton>
        </Tooltip>
      </div>
    </Paper>
  );
}
