import { useInjection } from "@wirestate/react";
import { formatBytes } from "@xrf/format";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DeleteOutlinedIcon as DeleteIcon, FolderOpenOutlinedIcon as FolderOpenIcon, LinkIcon } from "@xrf/ui/icons";
import { ButtonBase, IconButton, Paper, Tooltip, Typography } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { ILevelViewerScreenshot } from "@/applications/level-viewer/lib/screenshot/level-viewer-screenshot";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/level-viewer-screenshot";

/** Milliseconds a screenshot's toast stays once its pointer leaves it. */
const TOAST_LIFETIME: number = 6000;

/**
 * The screenshot just taken, in the viewport's corner for a few seconds and while the pointer is over it: its
 * thumbnail opening it whole, its folder, its path, and its deletion, which a second click confirms.
 */
export function LevelViewerScreenshotToast({
  "data-testid": dataTestId = "level-viewer-screenshot-toast",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);
  const shot: Nullable<ILevelViewerScreenshot> = screenshotService.shot.value;
  const [shownPath, setShownPath] = useState<Nullable<string>>(null);
  const [isHovered, setHovered] = useState<boolean>(false);
  const [isDeleteArmed, setDeleteArmed] = useState<boolean>(false);

  const onOpen = useCallback(() => {
    setShownPath(null);
    screenshotService.setShown(true);
  }, [screenshotService]);

  const onReveal = useCallback(() => void screenshotService.reveal(), [screenshotService]);

  const onCopyPath = useCallback(() => void screenshotService.copyPath(), [screenshotService]);

  const onDelete = useCallback(() => {
    if (!isDeleteArmed) {
      setDeleteArmed(true);

      return;
    }

    void screenshotService.remove();
  }, [isDeleteArmed, screenshotService]);

  const onEnter = useCallback(() => setHovered(true), []);

  const onLeave = useCallback(() => {
    setHovered(false);
    setDeleteArmed(false);
  }, []);

  useEffect(() => {
    setShownPath(shot?.path ?? null);
    setDeleteArmed(false);
  }, [shot?.path]);

  useEffect(() => {
    if (shownPath === null || isHovered) {
      return;
    }

    const timeout: ReturnType<typeof setTimeout> = setTimeout(() => setShownPath(null), TOAST_LIFETIME);

    return () => clearTimeout(timeout);
  }, [isHovered, shownPath]);

  if (!shot || shownPath !== shot.path) {
    return <></>;
  }

  return (
    <Paper
      data-testid={dataTestId}
      id={id}
      className={cn("absolute bottom-2 left-2 z-20 flex w-96 items-center gap-2 p-1.5", className)}
      elevation={8}
      role={"status"}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
    >
      <Tooltip title={"Open the screenshot"}>
        <ButtonBase
          className={"shrink-0 overflow-hidden rounded-surface"}
          aria-label={"Open the screenshot"}
          onClick={onOpen}
        >
          <img className={"block h-14 w-24 object-cover"} src={shot.url} alt={""} />
        </ButtonBase>
      </Tooltip>

      <div className={"flex min-w-0 grow flex-col"}>
        <Typography variant={"body2"}>Screenshot saved</Typography>
        <Typography className={"monospace truncate text-text-secondary"} variant={"caption"} title={shot.path}>
          {shot.name}
        </Typography>
        <Typography className={"text-text-secondary"} variant={"caption"}>
          {`${shot.width} × ${shot.height} · ${formatBytes(shot.size)}`}
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
