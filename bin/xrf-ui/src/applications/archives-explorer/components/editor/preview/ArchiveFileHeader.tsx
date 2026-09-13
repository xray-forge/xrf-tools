import { default as CloseIcon } from "@mui/icons-material/Close";
import { default as DescriptionOutlinedIcon } from "@mui/icons-material/DescriptionOutlined";
import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { IArchiveEntry } from "@/core/archive/lib";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { MONOSPACE } from "@/core/theme";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ArchiveFileExtractAction } from "./ArchiveFileExtractAction";

interface IArchiveFileHeaderProps extends BaseComponentProps {
  entry: IArchiveEntry;
}

export function ArchiveFileHeader({ entry }: IArchiveFileHeaderProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

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

      <EditorIconAction
        data-testid={"archive-file-deselect"}
        label={"Close file"}
        description={"Clear the selection and close this preview"}
        icon={<CloseIcon />}
        onClick={archivesService.clearFileSelection}
      />
    </Box>
  );
}
