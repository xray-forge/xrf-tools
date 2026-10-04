import { ReactElement } from "react";

import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatCount } from "@/lib/format/number";

interface ILevelRendererTexturesSectionProps extends BaseComponentProps {
  textures: ILevelTextureReport;
}

/**
 * What the renderer made of the level's textures: how many it uploaded, and how many it could not use.
 */
export function LevelRendererTexturesSection({
  "data-testid": dataTestId = "level-renderer-textures-section",
  id,
  className,
  textures,
}: ILevelRendererTexturesSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Textures"}>
      <EditorPanelProperty label={"Uploaded"} value={formatCount(textures.uploaded)} />
      <EditorPanelProperty label={"Unusable"} value={formatCount(textures.problems.length)} />
    </EditorPanelSection>
  );
}
