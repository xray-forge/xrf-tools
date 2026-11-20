import { Nullable } from "@xrf/types";
import { FocusEvent, ReactElement, ReactNode, useCallback, useEffect, useState } from "react";

import { ButtonBase } from "#/button";
import { EditorIconAction } from "#/editor/icon-action";
import { BaseComponentProps } from "#/element-types";
import { DeleteOutlinedIcon } from "#/icons/delete-outlined";
import { FolderOpenOutlinedIcon } from "#/icons/folder-open-outlined";
import { LinkIcon } from "#/icons/link";
import { Paper } from "#/layout/card";
import { Tooltip } from "#/overlay";
import { cn } from "#/theme";
import { Typography } from "#/typography";

/** What a toast's preview opens, the toast put away as it does. */
export interface IFileToastOpening {
  label: string;
  onOpen: () => void;
}

/** Milliseconds a toast stays once its pointer leaves it. */
const TOAST_LIFETIME: number = 6000;

/** An armed delete wears the error colour, so the second click is not taken for the first. */
const ARMED_DELETE_SX = { color: "error.main" } as const;

export interface IFileToastProps extends BaseComponentProps {
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
  opening?: IFileToastOpening;
  onReveal: () => void;
  onCopyPath: () => void;
  onDelete: () => void;
}

/**
 * A file just written, in the viewport's corner for a few seconds and while the pointer is over it. Its delete takes a
 * second click to confirm.
 */
export function FileToast({
  "data-testid": dataTestId = "file-toast",
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
}: IFileToastProps): ReactElement {
  const [shownFile, setShownFile] = useState<Nullable<string>>(null);
  const [isHovered, setHovered] = useState<boolean>(false);
  const [isFocused, setFocused] = useState<boolean>(false);
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

  // The keyboard's hover: a toast its actions are focused in stays, and a delete armed by keys disarms as focus goes.
  const onFocus = useCallback(() => setFocused(true), []);

  const onBlur = useCallback((event: FocusEvent<HTMLElement>) => {
    if (event.currentTarget.contains(event.relatedTarget as Nullable<Node>)) {
      return;
    }

    setFocused(false);
    setDeleteArmed(false);
  }, []);

  useEffect(() => {
    setShownFile(file);
    setDeleteArmed(false);
  }, [file]);

  const isShown: boolean = file !== null && shownFile === file;

  // A toast put away under the pointer or the focus, by its own Open or Delete, hears no leave or blur; the next one
  // starts as nothing rests on it, or it would never time out.
  useEffect(() => {
    if (!isShown) {
      setHovered(false);
      setFocused(false);
      setDeleteArmed(false);
    }
  }, [isShown]);

  useEffect(() => {
    if (shownFile === null || isHovered || isFocused) {
      return;
    }

    const timeout: ReturnType<typeof setTimeout> = setTimeout(() => setShownFile(null), TOAST_LIFETIME);

    return () => clearTimeout(timeout);
  }, [isFocused, isHovered, shownFile]);

  if (file === null || !isShown) {
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
      onFocus={onFocus}
      onBlur={onBlur}
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
        <EditorIconAction
          label={"Show in folder"}
          description={"Show the file in its folder"}
          icon={<FolderOpenOutlinedIcon />}
          onClick={onReveal}
        />
        <EditorIconAction
          label={"Copy path"}
          description={"Copy the file's path"}
          icon={<LinkIcon />}
          onClick={onCopyPath}
        />
        <EditorIconAction
          label={isDeleteArmed ? "Delete for good" : "Delete"}
          description={isDeleteArmed ? "Click again to delete it for good" : "Delete the file"}
          icon={<DeleteOutlinedIcon />}
          sx={isDeleteArmed ? ARMED_DELETE_SX : undefined}
          onClick={onDeleteClick}
        />
      </div>
    </Paper>
  );
}
