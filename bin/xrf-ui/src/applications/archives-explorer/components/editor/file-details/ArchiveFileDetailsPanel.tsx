import { Box, Divider, Typography } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ARCHIVE_EDITOR_MONOSPACE_FONT } from "@/applications/archives-explorer/components/editor/archive-editor.styles";
import { ArchiveFileDetailRow } from "@/applications/archives-explorer/components/editor/file-details/ArchiveFileDetailRow";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { getFileExtension } from "@/lib/path/extension";
import { Nullable } from "@/lib/types/general";

export interface IArchiveFileDetailsPanelProps extends BaseComponentProps {
  archivesService: ArchivesService;
}

export function ArchiveFileDetailsPanel({ archivesService }: IArchiveFileDetailsPanelProps): ReactElement {
  const descriptor: Nullable<ArchiveFileDescriptor> = archivesService.selectedFile;

  const getCompressionLabel = useCallback((descriptor: ArchiveFileDescriptor): string => {
    if (descriptor.sizeReal === descriptor.sizeCompressed) {
      return "Stored";
    }

    const ratio: number = descriptor.sizeReal ? (descriptor.sizeCompressed / descriptor.sizeReal) * 100 : 0;

    return `Compressed (${ratio.toFixed(1)}%)`;
  }, []);

  return descriptor ? (
    <Box sx={{ display: "flex", flexDirection: "column", minHeight: 0 }}>
      <Box sx={{ padding: 2 }}>
        <Typography variant={"subtitle2"}>File details</Typography>
        <Typography
          variant={"caption"}
          sx={{
            display: "block",
            marginTop: 0.5,
            color: "text.secondary",
            fontFamily: ARCHIVE_EDITOR_MONOSPACE_FONT,
            overflowWrap: "anywhere",
          }}
        >
          {descriptor.name}
        </Typography>
      </Box>

      <Divider />

      <Box sx={{ padding: 2 }}>
        <ArchiveFileDetailRow label={"Extension"} value={getFileExtension(descriptor.name) || "-"} />
        <ArchiveFileDetailRow label={"Source archive"} value={descriptor.source} isPath />
        <ArchiveFileDetailRow label={"Destination root"} value={descriptor.destination} isPath />
        <ArchiveFileDetailRow label={"Real size"} value={formatBytes(descriptor.sizeReal)} />
        <ArchiveFileDetailRow label={"Stored size"} value={formatBytes(descriptor.sizeCompressed)} />
        <ArchiveFileDetailRow label={"Compression"} value={getCompressionLabel(descriptor)} />
        <ArchiveFileDetailRow
          label={"CRC32"}
          value={`0x${descriptor.crc.toString(16).padStart(8, "0").toUpperCase()}`}
          mono
        />
        <ArchiveFileDetailRow
          label={"Offset"}
          value={`${formatBytes(descriptor.offset)} (${descriptor.offset})`}
          mono
        />
      </Box>
    </Box>
  ) : (
    <Box sx={{ padding: 2 }}>
      <Typography variant={"subtitle2"}>File details</Typography>
      <Typography variant={"body2"} sx={{ marginTop: 1, color: "text.secondary" }}>
        Select a file to inspect its archive metadata.
      </Typography>
    </Box>
  );
}
