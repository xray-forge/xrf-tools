import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { LevelLoadService } from "@/core/level/services";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * What the open level is built out of, which is everything the open read without touching its geometry.
 */
export function LevelHeaderPanel({
  "data-testid": dataTestId = "level-header-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const service: LevelLoadService = useInjection(LevelLoadService);
  const description: Nullable<SelectedLevelDescription> = service.level.value?.selected.value ?? null;

  if (!description) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Level"}>
        <EditorPanelEmpty label={"No level open. Open one from a game root to see what it holds."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Level"}>
      <EditorPanelSection title={"Built by"} isFirst>
        <EditorPanelProperty label={"xrLC version"} value={description.xrlcVersion} />
        <EditorPanelProperty label={"Quality"} value={description.xrlcQuality} />
      </EditorPanelSection>

      <EditorPanelSection title={"Structure"}>
        <EditorPanelProperty label={"Sectors"} value={description.sectors.length} />
        <EditorPanelProperty label={"Portals"} value={description.portals} />
        <EditorPanelProperty label={"Visuals"} value={description.visuals} />
        <EditorPanelProperty label={"Drawable"} value={description.drawables} />
        <EditorPanelProperty label={"Shader entries"} value={description.shaderEntries} />
      </EditorPanelSection>

      <EditorPanelSection title={"Lighting"}>
        <EditorPanelProperty label={"Static lights"} value={description.lights} />
        <EditorPanelProperty label={"Sun"} value={description.hasSun ? "Present" : "Absent"} />
      </EditorPanelSection>
    </EditorPanel>
  );
}
