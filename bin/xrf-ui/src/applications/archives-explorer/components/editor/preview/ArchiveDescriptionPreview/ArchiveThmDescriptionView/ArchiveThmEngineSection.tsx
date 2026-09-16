import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveThmTextureType } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
        <Typography className={"text-text-secondary"} variant={"body2"} sx={{ paddingY: PANEL.propertyPaddingY }}>
          {`LoadTHM takes nothing further from a ${textureType.label.toLowerCase()} descriptor: the bump, detail and material below are read by the converter and never by the engine.`}
        </Typography>
      )}
    </EditorPanelSection>
  );
}
