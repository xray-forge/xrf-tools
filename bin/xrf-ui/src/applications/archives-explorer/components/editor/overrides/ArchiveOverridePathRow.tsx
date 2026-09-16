import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ARCHIVE_OVERRIDE_ROW_HEIGHT, EArchiveOverrideRow, IArchiveOverridePathRow } from "./archive-override-rows";

export interface IArchiveOverridePathRowProps extends BaseComponentProps {
  row: IArchiveOverridePathRow;
  /** Opens the file this row names, which is the only reason a contested path is worth finding. */
  onOpen: (name: string) => void;
}

/**
 * One contested engine path, standing over the copies that claim it.
 */
export function ArchiveOverridePathRow({
  "data-testid": dataTestId = "archive-override-row",
  id,
  className,
  row,
  onOpen,
}: IArchiveOverridePathRowProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      data-kind={EArchiveOverrideRow.PATH}
      id={id}
      className={cn("flex cursor-pointer items-center gap-2 rounded-surface px-2 hover:bg-action-hover", className)}
      style={{ height: ARCHIVE_OVERRIDE_ROW_HEIGHT }}
      role={"button"}
      tabIndex={-1}
      title={"Open this file"}
      onClick={() => onOpen(row.entry.name)}
    >
      <Typography className={"monospace min-w-0 grow overflow-hidden text-ellipsis"} variant={"body2"} noWrap={true}>
        {row.entry.name}
      </Typography>

      <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
        {`${row.copies} copies`}
      </Typography>

      <Typography className={"min-w-18 shrink-0 text-right text-text-secondary"} variant={"caption"}>
        {`${formatBytes(row.hiddenSize)} hidden`}
      </Typography>
    </div>
  );
}
