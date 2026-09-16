import { Box } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDetailLibraryDescription } from "@/core/ipc/types/xrf-app";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDetailLayerSection } from "./ArchiveDetailLayerSection";
import { ArchiveDetailObjectsSection } from "./ArchiveDetailObjectsSection";

interface IArchiveDetailLibraryDescriptionViewProps extends BaseComponentProps {
  description: ArchiveDetailLibraryDescription;
}

/**
 * A level's detail layer: how much of the level it dresses, and what it dresses it with.
 */
export function ArchiveDetailLibraryDescriptionView({
  "data-testid": dataTestId = "archive-detail-library-description-view",
  id,
  className,
  description,
}: IArchiveDetailLibraryDescriptionViewProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth }}>
        <ArchiveDetailLayerSection description={description} />

        <ArchiveDetailObjectsSection entries={description.entries} />
      </Box>
    </Box>
  );
}
