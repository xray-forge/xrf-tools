import { Box } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveLevelAiDescription } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { formatCount, formatLevelBounds, formatNodeSize } from "./ArchiveLevelPartView.utils";

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
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth }}>
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
      </Box>
    </Box>
  );
}
