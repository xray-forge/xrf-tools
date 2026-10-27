import { formatCount } from "@xrf/format";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";

interface ILevelRendererTexturesSectionProps extends BaseComponentProps {
  textures: ILevelTextureReport;
}

/**
 * What the renderer made of the level's textures: how many it uploaded, how many it holds unread until a mode reads
 * them, and how many it could not use.
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
      <EditorPanelProperty label={"Deferred"} value={formatCount(textures.deferred)} />
      <EditorPanelProperty label={"Unusable"} value={formatCount(textures.problems.length)} />
    </EditorPanelSection>
  );
}
