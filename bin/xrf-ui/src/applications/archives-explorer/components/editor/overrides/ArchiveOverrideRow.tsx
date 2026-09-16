import { Box, Chip, Typography } from "@mui/material";
import { MouseEvent, ReactElement } from "react";

import { describeAssetContainer, isLooseContainer } from "@/core/assets/lib";
import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { EArchiveOverrideRow, TArchiveOverrideRow } from "./archive-override-rows";

/** Height every row of the listing draws at, so the virtualizer never measures one. */
export const ARCHIVE_OVERRIDE_ROW_HEIGHT = 28;

export interface IArchiveOverrideRowProps extends BaseComponentProps {
  row: TArchiveOverrideRow;
  rowId: string;
  /** Opens the file this row names. Only a path row carries one. */
  onOpen?: (name: string) => void;
}

/**
 * One row of the overrides listing: a contested path, or one copy claiming it.
 */
export function ArchiveOverrideRow({
  "data-testid": dataTestId = "archive-override-row",
  id,
  className,
  row,
  rowId,
  onOpen,
}: IArchiveOverrideRowProps): ReactElement {
  if (row.kind === EArchiveOverrideRow.PATH) {
    return (
      <Box
        data-testid={dataTestId}
        data-kind={EArchiveOverrideRow.PATH}
        id={id ?? rowId}
        className={className}
        role={"button"}
        tabIndex={-1}
        title={"Open this file"}
        onClick={(event: MouseEvent<HTMLElement>) => {
          event.preventDefault();
          onOpen?.(row.entry.name);
        }}
        sx={{
          display: "flex",
          alignItems: "center",
          gap: 1,
          height: ARCHIVE_OVERRIDE_ROW_HEIGHT,
          paddingX: 1,
          cursor: "pointer",
          borderRadius: 1,
          "&:hover": { backgroundColor: "action.hover" },
        }}
      >
        <Typography
          variant={"body2"}
          sx={{ ...MONOSPACE, flexGrow: 1, minWidth: 0, overflow: "hidden", textOverflow: "ellipsis" }}
          noWrap
        >
          {row.entry.name}
        </Typography>

        <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
          {`${row.copies} copies`}
        </Typography>

        <Typography
          variant={"caption"}
          sx={{ color: "text.secondary", flexShrink: 0, minWidth: 72, textAlign: "right" }}
        >
          {`${formatBytes(row.hiddenSize)} hidden`}
        </Typography>
      </Box>
    );
  }

  return (
    <Box
      data-testid={dataTestId}
      data-kind={EArchiveOverrideRow.COPY}
      id={id ?? rowId}
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
        title={describeAssetContainer(row.container)}
      >
        {describeAssetContainer(row.container)}
      </Typography>

      <Chip
        size={"small"}
        variant={"outlined"}
        label={isLooseContainer(row.container) ? "Loose" : "Archived"}
        sx={{ flexShrink: 0, height: 18, "& .MuiChip-label": { paddingX: 0.75, fontSize: 10 } }}
      />

      {row.isWinner ? (
        <Chip
          size={"small"}
          color={"primary"}
          label={"Loaded"}
          title={"The copy the engine loads"}
          sx={{ flexShrink: 0, height: 18, "& .MuiChip-label": { paddingX: 0.75, fontSize: 10 } }}
        />
      ) : null}

      <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0, minWidth: 72, textAlign: "right" }}>
        {formatBytes(row.sizeReal)}
      </Typography>
    </Box>
  );
}
