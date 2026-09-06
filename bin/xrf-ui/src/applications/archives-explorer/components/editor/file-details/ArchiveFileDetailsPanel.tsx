import { Box, Divider, Typography } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ARCHIVE_EDITOR_MONOSPACE_FONT } from "@/applications/archives-explorer/components/editor/archive-editor.styles";
import { ArchiveSharedPayloadDetail } from "@/applications/archives-explorer/components/editor/file-details/ArchiveSharedPayloadDetail";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { getArchiveVolumeOf } from "@/core/archive/files";
import { ArchiveDescriptor, ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { EditorPanelRow } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { getFileExtension } from "@/lib/path/extension";
import { Nullable } from "@/lib/types/general";

export interface IArchiveFileDetailsPanelProps extends BaseComponentProps {
  archivesService: ArchivesService;
}

export function ArchiveFileDetailsPanel({ archivesService }: IArchiveFileDetailsPanelProps): ReactElement {
  const descriptor: Nullable<ArchiveFileDescriptor> = archivesService.selectedFile;
  const volume: Nullable<ArchiveDescriptor> = getArchiveVolumeOf(archivesService.project.value, descriptor);

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
        <EditorPanelRow label={"Extension"} value={getFileExtension(descriptor.name) || "-"} />
        <EditorPanelRow label={"Source archive"} value={volume?.path ?? "-"} isMonospace />
        <EditorPanelRow label={"Destination root"} value={volume?.outputRootPath ?? "-"} isMonospace />
        <EditorPanelRow label={"Real size"} value={formatBytes(descriptor.sizeReal)} />
        <EditorPanelRow label={"Stored size"} value={formatBytes(descriptor.sizeCompressed)} />
        <EditorPanelRow label={"Compression"} value={getCompressionLabel(descriptor)} />
        <EditorPanelRow
          label={"CRC32"}
          value={`0x${descriptor.crc.toString(16).padStart(8, "0").toUpperCase()}`}
          isMonospace
        />
        <EditorPanelRow
          label={"Offset"}
          value={`${formatBytes(descriptor.offset)} (${descriptor.offset})`}
          isMonospace
        />
        <ArchiveSharedPayloadDetail descriptor={descriptor} sharedPayloads={archivesService.sharedPayloads} />
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
