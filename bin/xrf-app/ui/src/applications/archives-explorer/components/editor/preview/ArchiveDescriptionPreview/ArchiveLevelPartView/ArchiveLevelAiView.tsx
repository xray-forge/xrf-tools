import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveLevelAiDescription } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement } from "react";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatLevelBounds } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { formatNodeSize } from "./ArchiveLevelPartView.utils";

interface IArchiveLevelAiViewProps extends BaseComponentProps {
  description: ArchiveLevelAiDescription;
}

/**
 * A level's navigation grid, read from the header alone.
 */
export function ArchiveLevelAiView({
  "data-testid": dataTestId = "archive-level-ai-view",
  id,
  className,
  description,
}: IArchiveLevelAiViewProps): ReactElement {
  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Navigation grid"} isFirst>
        <ArchiveDescriptionRow
          label={"Nodes"}
          value={formatCount(description.nodes)}
          caption={formatNodeSize(description.nodeSize, description.nodeHeight)}
        />

        <ArchiveDescriptionRow
          label={"Covers"}
          value={formatLevelBounds(description.bounds)}
          caption={"Width, height and depth of the box it declares"}
        />

        <ArchiveDescriptionRow
          label={"Graph identity"}
          value={description.guid}
          isMonospace
          caption={"A spawn set built against this grid carries the same value"}
        />

        <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />

        <ArchiveDescriptionRow
          label={"Size"}
          value={formatBytes(description.size)}
          caption={"The nodes are read past; only the header is read"}
        />
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
