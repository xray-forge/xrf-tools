import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveLevelAiDescription } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

import { formatLevelBounds } from "../ArchiveDescriptionPreview.utils";
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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Navigation grid"}>
        <EditorPanelProperty
          isInline
          label={"Nodes"}
          value={formatCount(description.nodes)}
          caption={formatNodeSize(description.nodeSize, description.nodeHeight)}
        />

        <EditorPanelProperty
          isInline
          label={"Covers"}
          value={formatLevelBounds(description.bounds)}
          caption={"Width, height and depth of the box it declares"}
        />

        <EditorPanelProperty
          isInline
          label={"Graph identity"}
          value={description.guid}
          isMonospace
          caption={"A spawn set built against this grid carries the same value"}
        />

        <EditorPanelProperty isInline label={"Version"} value={`${description.version}`} />

        <EditorPanelProperty
          isInline
          label={"Size"}
          value={formatBytes(description.size)}
          caption={"The nodes are read past; only the header is read"}
        />
      </EditorPanelSection>
    </ReadingColumn>
  );
}
