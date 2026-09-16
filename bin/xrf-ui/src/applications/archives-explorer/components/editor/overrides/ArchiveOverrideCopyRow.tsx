import { Box, Chip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { describeAssetContainer, isLooseContainer } from "@/core/assets/lib";
import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ARCHIVE_OVERRIDE_ROW_HEIGHT, EArchiveOverrideRow, IArchiveOverrideCopyRow } from "./archive-override-rows";

/** Chip sizing shared by both marks, so the rank column stays aligned whichever marks a row carries. */
const CHIP_SX = { flexShrink: 0, height: 18, "& .MuiChip-label": { paddingX: 0.75, fontSize: 10 } } as const;

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
    <Box
      data-testid={dataTestId}
      data-kind={EArchiveOverrideRow.COPY}
      id={id}
      className={className}
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 1,
        height: ARCHIVE_OVERRIDE_ROW_HEIGHT,
        paddingLeft: 3,
        paddingRight: 1,
        opacity: row.isWinner ? 1 : 0.7,
      }}
    >
      <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0, width: 16 }}>
        {row.rank}
      </Typography>

      <Typography
        variant={"caption"}
        sx={{ ...MONOSPACE, flexGrow: 1, minWidth: 0, overflow: "hidden", textOverflow: "ellipsis" }}
        noWrap
        title={container}
      >
        {container}
      </Typography>

      <Chip
        size={"small"}
        variant={"outlined"}
        label={isLooseContainer(row.container) ? "Loose" : "Archived"}
        sx={CHIP_SX}
      />

      {row.isWinner ? (
        <Chip size={"small"} color={"primary"} label={"Loaded"} title={"The copy the engine loads"} sx={CHIP_SX} />
      ) : null}

      <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0, minWidth: 72, textAlign: "right" }}>
        {formatBytes(row.sizeReal)}
      </Typography>
    </Box>
  );
}
