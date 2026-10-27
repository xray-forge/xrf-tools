import { useInjection } from "@wirestate/react";
import { formatBytes, formatHex, formatPercent } from "@xrf/format";
import { ArchiveSubject, ArchiveWorldEntry, EArchiveSubject } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveDescriptor, ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";
import { getFileExtension } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty, EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback } from "react";

import { getArchiveVolumeOf, IArchiveEntry } from "@/core/archive/lib";
import { EditorPanel } from "@/core/shell/editor/EditorPanel";
import { ArchivePreviewService } from "@/tools/archives-explorer/services/archive-preview";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

import { ArchiveOriginDetail } from "./ArchiveOriginDetail";
import { ArchiveSharedPayloadDetail } from "./ArchiveSharedPayloadDetail";

/**
 * What the tree has selected, described in the vocabulary of whichever subject is open.
 */
export function ArchiveFileDetailsPanel({
  "data-testid": dataTestId = "archive-file-details-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);

  const subject: Nullable<ArchiveSubject> = archivesService.subject.value;
  const entry: Nullable<IArchiveEntry> = previewService.selectedEntry;
  const descriptor: Nullable<ArchiveFileDescriptor> = previewService.selectedDescriptor;
  const origin: Nullable<ArchiveWorldEntry> = previewService.selectedOrigin;

  const volume: Nullable<ArchiveDescriptor> = getArchiveVolumeOf(
    subject?.kind === EArchiveSubject.VOLUMES ? subject.project : null,
    descriptor
  );

  const getCompressionLabel = useCallback((descriptor: ArchiveFileDescriptor): string => {
    if (descriptor.sizeReal === descriptor.sizeCompressed) {
      return "Stored";
    }

    const ratio: number = descriptor.sizeReal ? descriptor.sizeCompressed / descriptor.sizeReal : 0;

    return `Compressed (${formatPercent(ratio, 1)})`;
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
            <EditorPanelProperty label={"Destination root"} value={volume?.outputRootPath ?? "-"} isMonospace />

            <EditorPanelProperty label={"Stored size"} value={formatBytes(descriptor.sizeCompressed)} />

            <EditorPanelProperty label={"Compression"} value={getCompressionLabel(descriptor)} />

            <EditorPanelProperty label={"CRC32"} value={formatHex(descriptor.crc, 8)} isMonospace />

            <EditorPanelProperty
              label={"Offset"}
              value={`${formatBytes(descriptor.offset)} (${descriptor.offset})`}
              isMonospace
            />

            <ArchiveSharedPayloadDetail descriptor={descriptor} sharedPayloads={archivesService.sharedPayloads.state} />
          </>
        ) : null}
      </EditorPanelSection>

      {origin ? <ArchiveOriginDetail entry={origin} /> : null}
    </EditorPanel>
  );
}
