import { formatBytes } from "@xrf/format";
import { ArchiveThmFile } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement } from "react";

import { formatChunkId, NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeThumbnailType } from "./ArchiveThmDescriptionView.utils";

interface IArchiveThmFileSectionProps extends BaseComponentProps {
  file: ArchiveThmFile;
}

/**
 * What the file is, apart from what it declares.
 */
export function ArchiveThmFileSection({
  "data-testid": dataTestId = "archive-thm-file-section",
  id,
  className,
  file,
}: IArchiveThmFileSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"File"}>
      <ArchiveDescriptionRow
        label={"Version"}
        value={file.version === null ? NOT_DECLARED : formatChunkId(file.version)}
        caption={file.isSupportedVersion ? null : "Not the version the SDK loads"}
        isMonospace
      />

      <ArchiveDescriptionRow label={"Thumbnail subject"} value={describeThumbnailType(file.thumbnailType)} />

      <ArchiveDescriptionRow
        label={"Preview picture"}
        value={file.thumbnail ? formatBytes(file.thumbnail.size) : "None"}
        caption={
          file.thumbnail
            ? `Carried through as stored${file.thumbnail.isCompressed ? ", compressed" : ""}; the SDK stopped writing this chunk`
            : null
        }
      />

      {file.extraChunks.length ? (
        <ArchiveDescriptionRow
          label={"Other chunks"}
          value={file.extraChunks.map(formatChunkId).join(", ")}
          caption={"Read and kept, with no field of their own"}
          isMonospace
        />
      ) : null}

      {file.trailingChunks.length ? (
        <ArchiveDescriptionRow
          label={"Bytes past fields"}
          value={file.trailingChunks.map(formatChunkId).join(", ")}
          caption={"Chunks longer than their fields: the engine ignores the extra bytes, and a save keeps them"}
          isMonospace
        />
      ) : null}
    </EditorPanelSection>
  );
}
