import { ArchiveThmTextureType } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material";
import { ReactElement } from "react";

import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";

interface IArchiveThmEngineSectionProps extends BaseComponentProps {
  textureType: ArchiveThmTextureType;
}

/**
 * The gate `LoadTHM` reads before anything else, and what it means for the rest of the file.
 *
 * 743 of vanilla's 2,736 descriptors are a bump map or a cube map, for which the engine reads nothing past this type
 * however complete the chunks below are. Saying so is the point of the section: it is not derivable from anything else
 * on screen.
 */
export function ArchiveThmEngineSection({
  "data-testid": dataTestId = "archive-thm-engine-section",
  id,
  className,
  textureType,
}: IArchiveThmEngineSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Engine"}>
      <ArchiveDescriptionRow
        label={"Texture type"}
        value={textureType.label}
        caption={textureType.isDeclared ? null : "Not declared; the engine reads the zeroed default"}
      />

      {textureType.isReadByEngine ? null : (
        <Typography className={"py-panel-property text-text-secondary"} variant={"body2"}>
          {`LoadTHM takes nothing further from a ${textureType.label.toLowerCase()} descriptor: the bump, detail and material below are read by the converter and never by the engine.`}
        </Typography>
      )}
    </EditorPanelSection>
  );
}
