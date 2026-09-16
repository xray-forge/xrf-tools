import { Chip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { describeAssetContainer, isLooseContainer } from "@/core/assets/lib";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import {
  ARCHIVE_OVERRIDE_CHIP_SX,
  ARCHIVE_OVERRIDE_ROW_HEIGHT,
  EArchiveOverrideRow,
  IArchiveOverrideCopyRow,
} from "./archive-override-rows";

export interface IArchiveOverrideCopyRowProps extends BaseComponentProps {
  row: IArchiveOverrideCopyRow;
}

/**
 * One copy of a contested path, at its place in the search order.
 */
export function ArchiveOverrideCopyRow({
  "data-testid": dataTestId = "archive-override-row",
  id,
  className,
  row,
}: IArchiveOverrideCopyRowProps): ReactElement {
  const container: string = describeAssetContainer(row.container);

  return (
    <div
      data-testid={dataTestId}
      data-kind={EArchiveOverrideRow.COPY}
      id={id}
      className={cn("flex items-center gap-2 pr-2 pl-6", row.isWinner ? "opacity-100" : "opacity-70", className)}
      style={{ height: ARCHIVE_OVERRIDE_ROW_HEIGHT }}
    >
      <Typography className={"w-4 shrink-0 text-text-secondary"} variant={"caption"}>
        {row.rank}
      </Typography>

      <Typography
        className={"monospace min-w-0 grow overflow-hidden text-ellipsis"}
        variant={"caption"}
        noWrap={true}
        title={container}
      >
        {container}
      </Typography>

      <Chip
        size={"small"}
        variant={"outlined"}
        label={isLooseContainer(row.container) ? "Loose" : "Archived"}
        sx={ARCHIVE_OVERRIDE_CHIP_SX}
      />

      {row.isWinner ? (
        <Chip
          size={"small"}
          color={"primary"}
          label={"Loaded"}
          title={"The copy the engine loads"}
          sx={ARCHIVE_OVERRIDE_CHIP_SX}
        />
      ) : null}

      <Typography className={"min-w-18 shrink-0 text-right text-text-secondary"} variant={"caption"}>
        {formatBytes(row.sizeReal)}
      </Typography>
    </div>
  );
}
