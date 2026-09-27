import { ReactElement } from "react";

import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { formatCount } from "@/lib/format/number";

interface ILevelStreamTexturesSectionProps {
  textures: ILevelTextureReport;
}

/**
 * What the renderer made of the level's textures: how many it uploaded, and how many it could not use.
 */
export function LevelStreamTexturesSection({ textures }: ILevelStreamTexturesSectionProps): ReactElement {
  return (
    <EditorPanelSection title={"Textures"}>
      <EditorPanelProperty label={"Uploaded"} value={formatCount(textures.uploaded)} />
      <EditorPanelProperty label={"Unusable"} value={formatCount(textures.problems.length)} />
    </EditorPanelSection>
  );
}
