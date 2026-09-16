import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveThmDescription } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
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
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
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
    </ArchiveDescriptionLayout>
  );
}
