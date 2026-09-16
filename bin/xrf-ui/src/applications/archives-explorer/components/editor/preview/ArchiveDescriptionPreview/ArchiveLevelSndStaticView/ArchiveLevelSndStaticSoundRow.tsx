import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveLevelSndStaticSound } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeReferenceStatus } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";
import { describePlayback, describeSchedule } from "./ArchiveLevelSndStaticView.utils";

interface IArchiveLevelSndStaticSoundRowProps extends BaseComponentProps {
  sound: ArchiveLevelSndStaticSound;
  scope: ArchiveDescribeScope;
}

/**
 * One sound the level plants: what it plays, how it plays it, and when it is allowed to.
 */
export function ArchiveLevelSndStaticSoundRow({
  "data-testid": dataTestId = "archive-level-snd-static-sound-row",
  id,
  className,
  sound,
  scope,
}: IArchiveLevelSndStaticSoundRowProps): ReactElement {
  const schedule: Nullable<string> = describeSchedule(sound);
  const status: Nullable<string> = sound.sound ? describeReferenceStatus(sound.sound, scope) : null;

  return (
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {sound.sound ? <ArchiveDescriptionReferenceLink reference={sound.sound} /> : "Names no sound"}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {describePlayback(sound)}
        </Typography>
      </div>

      {schedule ? (
        <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
          {schedule}
        </Typography>
      ) : null}

      {status ? (
        <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
          {status}
        </Typography>
      ) : null}
    </div>
  );
}
