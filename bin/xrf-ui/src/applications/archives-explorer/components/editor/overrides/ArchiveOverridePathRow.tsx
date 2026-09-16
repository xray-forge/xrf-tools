import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { MONOSPACE } from "@/core/theme/tokens";
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
    <Box
      data-testid={dataTestId}
      data-kind={EArchiveOverrideRow.PATH}
      id={id}
      className={className}
      role={"button"}
      tabIndex={-1}
      title={"Open this file"}
      onClick={() => onOpen(row.entry.name)}
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

      <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0, minWidth: 72, textAlign: "right" }}>
        {`${formatBytes(row.hiddenSize)} hidden`}
      </Typography>
    </Box>
  );
}
