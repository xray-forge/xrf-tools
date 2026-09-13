import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { IArchiveEntry } from "@/core/archive/lib";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ArchiveFileExtractAction } from "./ArchiveFileExtractAction";

interface IArchiveFileHeaderProps extends BaseComponentProps {
  entry: IArchiveEntry;
}

export function ArchiveFileHeader({ entry }: IArchiveFileHeaderProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  return (
    <EditorFileHeader
      data-testid={"archive-file-header"}
      name={entry.name}
      caption={formatBytes(entry.sizeReal)}
      actions={<ArchiveFileExtractAction entry={entry} />}
      closeDescription={"Clear the selection and close this preview"}
      onClose={archivesService.clearFileSelection}
    />
  );
}
