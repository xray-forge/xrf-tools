import { default as DescriptionOutlinedIcon } from "@mui/icons-material/DescriptionOutlined";
import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { IArchiveEntry } from "@/core/archive";
import { MONOSPACE } from "@/core/theme";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ArchiveFileExtractAction } from "./ArchiveFileExtractAction";

interface IArchiveFileHeaderProps extends BaseComponentProps {
  entry: IArchiveEntry;
}

export function ArchiveFileHeader({ entry }: IArchiveFileHeaderProps): ReactElement {
  return (
    <Box
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 1,
        minHeight: 40,
        paddingX: 1.5,
        borderBottom: 1,
        borderColor: "divider",
        backgroundColor: "background.paper",
      }}
    >
      <DescriptionOutlinedIcon fontSize={"small"} sx={{ color: "text.secondary" }} />

      <Typography noWrap variant={"body2"} sx={{ flexGrow: 1, minWidth: 0, fontFamily: MONOSPACE.fontFamily }}>
        {entry.name}
      </Typography>

      <Typography noWrap variant={"caption"} sx={{ color: "text.secondary" }}>
        {formatBytes(entry.sizeReal)}
      </Typography>

      <ArchiveFileExtractAction entry={entry} />
    </Box>
  );
}
