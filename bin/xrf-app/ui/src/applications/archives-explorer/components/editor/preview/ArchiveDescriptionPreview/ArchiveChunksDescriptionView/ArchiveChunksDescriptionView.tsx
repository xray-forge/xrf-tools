import { formatBytes } from "@xrf/format";
import { ArchiveChunkNode, ArchiveChunksDescription } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement } from "react";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveChunkNodeRow } from "./ArchiveChunkNodeRow";

interface IArchiveChunksDescriptionViewProps extends BaseComponentProps {
  description: ArchiveChunksDescription;
}

/**
 * The container a file is, for a file nothing here reads.
 */
export function ArchiveChunksDescriptionView({
  "data-testid": dataTestId = "archive-chunks-description-view",
  id,
  className,
  description,
}: IArchiveChunksDescriptionViewProps): ReactElement {
  const { chunks, nodes, depth, size } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection
        title={"Container"}
        caption={"No reader claims this format. What follows is the framing every X-Ray binary shares."}
        isFirst
      >
        <ArchiveDescriptionRow
          label={"Chunks"}
          value={`${nodes}`}
          caption={
            depth === 1
              ? "A flat sequence, none of whose payloads is itself a container"
              : `Nested ${depth} deep, counting every level`
          }
        />

        <ArchiveDescriptionRow
          label={"Size"}
          value={formatBytes(size)}
          caption={"Unpacked, and accounted for in full by the chunks below"}
        />
      </EditorPanelSection>

      <EditorPanelSection
        title={"Chunks"}
        caption={"Ids as the numbers they are; naming one would mean reading the format"}
      >
        {chunks.map((node: ArchiveChunkNode, index: number) => (
          <ArchiveChunkNodeRow key={`${index}-${node.id}`} node={node} />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
