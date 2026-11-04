import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DeleteOutlinedIcon as DeleteIcon } from "@xrf/ui/icons/delete-outlined";
import { FolderOpenOutlinedIcon as FolderOpenIcon } from "@xrf/ui/icons/folder-open-outlined";
import { LinkIcon } from "@xrf/ui/icons/link";
import { ButtonBase } from "@xrf/ui/material/button-base";
import { IconButton } from "@xrf/ui/material/icon-button";
import { Paper } from "@xrf/ui/material/paper";
import { Tooltip } from "@xrf/ui/material/tooltip";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode, useCallback, useEffect, useState } from "react";

/** What a toast's preview opens, the toast put away as it does. */
export interface ILevelViewerFileToastOpening {
  label: string;
  onOpen: () => void;
}

/** Milliseconds a toast stays once its pointer leaves it. */
const TOAST_LIFETIME: number = 6000;

interface ILevelViewerFileToastProps extends BaseComponentProps {
  /** The file just written, which shows the toast again whenever it changes; null shows none. */
  file: Nullable<string>;
  /** What was written, as its toast is named. */
  title: string;
  /** The file's name, its path in the name's tooltip. */
  name: string;
  /** What the file holds. */
  details: string;
  /** Whether the details warn, as of a recording that dropped something. */
  isWarned?: boolean;
  /** Drawn at the toast's start, in a 96 by 56 well. */
  preview: ReactNode;
  /** What the preview opens when clicked, for a file the application shows itself. */
  opening?: ILevelViewerFileToastOpening;
  onReveal: () => void;
  onCopyPath: () => void;
  onDelete: () => void;
}

/**
 * A file the level viewer just wrote, in the viewport's corner for a few seconds and while the pointer is over it: its
 * preview, its name and what it holds, its folder, its path, and its deletion, which a second click confirms.
 */
export function LevelViewerFileToast({
  "data-testid": dataTestId = "level-viewer-file-toast",
  id,
  className,
  file,
  title,
  name,
  details,
  isWarned = false,
  preview,
  opening,
  onReveal,
  onCopyPath,
  onDelete,
}: ILevelViewerFileToastProps): ReactElement {
  const [shownFile, setShownFile] = useState<Nullable<string>>(null);
  const [isHovered, setHovered] = useState<boolean>(false);
  const [isDeleteArmed, setDeleteArmed] = useState<boolean>(false);

  const onOpen = useCallback(() => {
    setShownFile(null);
    opening?.onOpen();
  }, [opening]);

  const onDeleteClick = useCallback(() => {
    if (!isDeleteArmed) {
      setDeleteArmed(true);

      return;
    }

    onDelete();
  }, [isDeleteArmed, onDelete]);

  const onEnter = useCallback(() => setHovered(true), []);

  const onLeave = useCallback(() => {
    setHovered(false);
    setDeleteArmed(false);
  }, []);

  useEffect(() => {
    setShownFile(file);
    setDeleteArmed(false);
  }, [file]);

  useEffect(() => {
    if (shownFile === null || isHovered) {
      return;
    }

    const timeout: ReturnType<typeof setTimeout> = setTimeout(() => setShownFile(null), TOAST_LIFETIME);

    return () => clearTimeout(timeout);
  }, [isHovered, shownFile]);

  if (file === null || shownFile !== file) {
    return <></>;
  }

  return (
    <Paper
      data-testid={dataTestId}
      id={id}
      className={cn("absolute bottom-2 left-2 z-20 flex w-96 items-center gap-2 p-1.5", className)}
      elevation={8}
      role={"status"}
      aria-label={title}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
    >
      {opening ? (
        <Tooltip title={opening.label}>
          <ButtonBase
            className={"h-14 w-24 shrink-0 overflow-hidden rounded-surface"}
            aria-label={opening.label}
            onClick={onOpen}
          >
            {preview}
          </ButtonBase>
        </Tooltip>
      ) : (
        <div className={"flex h-14 w-24 shrink-0 items-center justify-center rounded-surface bg-well"}>{preview}</div>
      )}

      <div className={"flex min-w-0 grow flex-col"}>
        <Typography variant={"body2"}>{title}</Typography>
        <Typography className={"monospace truncate text-text-secondary"} variant={"caption"} title={file}>
          {name}
        </Typography>
        <Typography className={isWarned ? "text-warning" : "text-text-secondary"} variant={"caption"}>
          {details}
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
            onClick={onDeleteClick}
          >
            <DeleteIcon fontSize={"small"} />
          </IconButton>
        </Tooltip>
      </div>
    </Paper>
  );
}
