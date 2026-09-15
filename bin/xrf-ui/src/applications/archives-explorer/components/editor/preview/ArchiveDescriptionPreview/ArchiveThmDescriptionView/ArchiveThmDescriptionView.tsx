import { Box } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveThmDescription } from "@/core/ipc/types/xrf-app";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveThmEngineSection } from "./ArchiveThmEngineSection";
import { ArchiveThmFileSection } from "./ArchiveThmFileSection";
import { ArchiveThmRecipeSection } from "./ArchiveThmRecipeSection";
import { ArchiveThmReferencesSection } from "./ArchiveThmReferencesSection";
import { ArchiveThmShadingSection } from "./ArchiveThmShadingSection";
import { ArchiveThmTextureSection } from "./ArchiveThmTextureSection";

interface IArchiveThmDescriptionViewProps extends BaseComponentProps {
  description: ArchiveThmDescription;
  scope: ArchiveDescribeScope;
}

/**
 * A texture descriptor, read in the engine's order rather than the file's.
 */
export function ArchiveThmDescriptionView({
  "data-testid": dataTestId = "archive-thm-description-view",
  id,
  className,
  description,
  scope,
}: IArchiveThmDescriptionViewProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth }}>
        <ArchiveThmTextureSection texture={description.texture} scope={scope} />

        <ArchiveThmEngineSection textureType={description.textureType} />

        <ArchiveThmReferencesSection
          bump={description.bump}
          detail={description.detail}
          externalNormalMap={description.externalNormalMap}
          scope={scope}
        />

        <ArchiveThmShadingSection material={description.material} />

        <ArchiveThmRecipeSection parameters={description.parameters} fadeDelay={description.fadeDelay} />

        <ArchiveThmFileSection file={description.file} />
      </Box>
    </Box>
  );
}
