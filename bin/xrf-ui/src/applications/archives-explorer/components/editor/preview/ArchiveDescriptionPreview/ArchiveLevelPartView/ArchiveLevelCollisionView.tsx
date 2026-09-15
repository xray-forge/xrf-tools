import { Box } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveLevelCollisionDescription } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { formatCount, formatLevelBounds } from "./ArchiveLevelPartView.utils";

interface IArchiveLevelCollisionViewProps extends BaseComponentProps {
  description: ArchiveLevelCollisionDescription;
}

/**
 * A level's collision mesh, read from the header alone.
 */
export function ArchiveLevelCollisionView({
  "data-testid": dataTestId = "archive-level-collision-view",
  id,
  className,
  description,
}: IArchiveLevelCollisionViewProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth }}>
        <EditorPanelSection title={"Collision mesh"} isFirst>
          <ArchiveDescriptionRow
            label={"Faces"}
            value={formatCount(description.faces)}
            caption={`Over ${formatCount(description.vertices)} vertices`}
          />

          <ArchiveDescriptionRow
            label={"Covers"}
            value={formatLevelBounds(description.bounds)}
            caption={"Width, height and depth of the box it declares"}
          />

          <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />

          <ArchiveDescriptionRow
            label={"Size"}
            value={formatBytes(description.size)}
            caption={"The mesh itself is read past; only the header is read"}
          />
        </EditorPanelSection>
      </Box>
    </Box>
  );
}
