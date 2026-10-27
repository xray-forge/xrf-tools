import { useInjection } from "@wirestate/react";
import { formatBytes } from "@xrf/format";
import { ReactElement } from "react";

import { ArchivePreviewService } from "@/applications/archives-explorer/services/preview";
import { IArchiveEntry } from "@/core/archive/lib";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveFileExtractAction } from "./ArchiveFileExtractAction";

interface IArchiveFileHeaderProps extends BaseComponentProps {
  entry: IArchiveEntry;
}

export function ArchiveFileHeader({ entry }: IArchiveFileHeaderProps): ReactElement {
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);

  return (
    <EditorFileHeader
      data-testid={"archive-file-header"}
      name={entry.name}
      caption={formatBytes(entry.sizeReal)}
      actions={<ArchiveFileExtractAction entry={entry} />}
      closeDescription={"Clear the selection and close this preview"}
      onClose={previewService.clearSelection}
    />
  );
}
