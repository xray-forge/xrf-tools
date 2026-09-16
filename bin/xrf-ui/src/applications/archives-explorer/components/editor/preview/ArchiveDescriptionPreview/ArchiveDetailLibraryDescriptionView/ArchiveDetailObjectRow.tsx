import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDetailEntry } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";
import { describeEntryPlanting } from "./ArchiveDetailLibraryDescriptionView.utils";
import { describeModelDetail } from "../ArchiveDetailModelSection/ArchiveDetailModelSection.utils";

interface IArchiveDetailObjectRowProps extends BaseComponentProps {
  entry: ArchiveDetailEntry;
}

/**
 * One object of a level's detail library: what it draws with, how widely it is planted, and what qualifies it.
 */
export function ArchiveDetailObjectRow({
  "data-testid": dataTestId = "archive-detail-object-row",
  id,
  className,
  entry,
}: IArchiveDetailObjectRowProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {entry.model.texture ? (
            <ArchiveDescriptionReferenceLink reference={entry.model.texture} />
          ) : (
            `Object ${entry.index}`
          )}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {describeEntryPlanting(entry)}
        </Typography>
      </div>

      <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
        {describeModelDetail(entry.model)}
      </Typography>
    </div>
  );
}
