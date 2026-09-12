import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { EArchiveSubject, getArchiveVolumeOf, IArchiveEntry } from "@/core/archive/lib";
import { ArchiveSubject, ArchiveWorldEntry } from "@/core/bindings/types/xrf-app";
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
import { ArchiveWorldOriginDetail } from "./ArchiveWorldOriginDetail";

/**
 * What the tree has selected, described in the vocabulary of whichever subject is open.
 */
export function ArchiveFileDetailsPanel({
  "data-testid": dataTestId = "archive-file-details-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const subject: Nullable<ArchiveSubject> = archivesService.subject.value;
  const entry: Nullable<IArchiveEntry> = archivesService.selectedEntry;
  const descriptor: Nullable<ArchiveFileDescriptor> = archivesService.selectedDescriptor;
  const worldEntry: Nullable<ArchiveWorldEntry> = archivesService.selectedWorldEntry;

  const volume: Nullable<ArchiveDescriptor> = getArchiveVolumeOf(
    subject?.kind === EArchiveSubject.VOLUMES ? subject.project : null,
    descriptor
  );

  const getCompressionLabel = useCallback((descriptor: ArchiveFileDescriptor): string => {
    if (descriptor.sizeReal === descriptor.sizeCompressed) {
      return "Stored";
    }

    const ratio: number = descriptor.sizeReal ? (descriptor.sizeCompressed / descriptor.sizeReal) * 100 : 0;

    return `Compressed (${ratio.toFixed(1)}%)`;
  }, []);

  if (!entry) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"File details"}>
        <EditorPanelEmpty label={"Select a file to inspect where it comes from."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"File details"}>
      <EditorPanelSection title={"Metadata"} isFirst>
        <EditorPanelProperty label={"Name"} value={entry.name} isMonospace />

        <EditorPanelProperty label={"Extension"} value={getFileExtension(entry.name) || "-"} />

        <EditorPanelProperty label={"Real size"} value={formatBytes(entry.sizeReal)} />

        {descriptor ? (
          <>
            <EditorPanelProperty label={"Source archive"} value={volume?.path ?? "-"} isMonospace />

            <EditorPanelProperty label={"Destination root"} value={volume?.outputRootPath ?? "-"} isMonospace />

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
          </>
        ) : null}
      </EditorPanelSection>

      {worldEntry ? <ArchiveWorldOriginDetail entry={worldEntry} /> : null}
    </EditorPanel>
  );
}
