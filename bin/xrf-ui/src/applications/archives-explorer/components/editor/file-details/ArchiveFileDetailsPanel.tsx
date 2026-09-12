import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { getArchiveVolumeOf } from "@/core/archive/files";
import { ArchiveDescriptor, ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { getFileExtension } from "@/lib/path/extension";
import { Nullable } from "@/lib/types/general";

import { ArchiveSharedPayloadDetail } from "./ArchiveSharedPayloadDetail";

// todo: Inject or supply data directly as props.
export interface IArchiveFileDetailsPanelProps extends BaseComponentProps {
  archivesService: ArchivesService;
}

export function ArchiveFileDetailsPanel({
  "data-testid": dataTestId = "archive-file-details-panel",
  id,
  className,
  archivesService,
}: IArchiveFileDetailsPanelProps): ReactElement {
  const descriptor: Nullable<ArchiveFileDescriptor> = archivesService.selectedFile;
  const volume: Nullable<ArchiveDescriptor> = getArchiveVolumeOf(archivesService.project.value, descriptor);

  const getCompressionLabel = useCallback((descriptor: ArchiveFileDescriptor): string => {
    if (descriptor.sizeReal === descriptor.sizeCompressed) {
      return "Stored";
    }

    const ratio: number = descriptor.sizeReal ? (descriptor.sizeCompressed / descriptor.sizeReal) * 100 : 0;

    return `Compressed (${ratio.toFixed(1)}%)`;
  }, []);

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"File details"}>
      {descriptor ? (
        <EditorPanelSection title={"Metadata"} isFirst>
          <EditorPanelProperty label={"Name"} value={descriptor.name} isMonospace />
          <EditorPanelProperty label={"Extension"} value={getFileExtension(descriptor.name) || "-"} />
          <EditorPanelProperty label={"Source archive"} value={volume?.path ?? "-"} isMonospace />
          <EditorPanelProperty label={"Destination root"} value={volume?.outputRootPath ?? "-"} isMonospace />
          <EditorPanelProperty label={"Real size"} value={formatBytes(descriptor.sizeReal)} />
          <EditorPanelProperty label={"Stored size"} value={formatBytes(descriptor.sizeCompressed)} />
          <EditorPanelProperty label={"Compression"} value={getCompressionLabel(descriptor)} />
          <EditorPanelProperty
            label={"CRC32"}
            value={`0x${descriptor.crc.toString(16).padStart(8, "0").toUpperCase()}`}
            isMonospace
          />
          <EditorPanelProperty
            label={"Offset"}
            value={`${formatBytes(descriptor.offset)} (${descriptor.offset})`}
            isMonospace
          />
          <ArchiveSharedPayloadDetail descriptor={descriptor} sharedPayloads={archivesService.sharedPayloads} />
        </EditorPanelSection>
      ) : (
        <EditorPanelEmpty label={"Select a file to inspect its archive metadata."} />
      )}
    </EditorPanel>
  );
}
