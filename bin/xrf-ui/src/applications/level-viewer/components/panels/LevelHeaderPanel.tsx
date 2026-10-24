import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelLoadService } from "@/core/level/services/level-load.service";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
        <EditorPanelProperty label={"Sectors"} value={formatCount(description.sectors.length)} />
        <EditorPanelProperty label={"Portals"} value={formatCount(description.portals)} />
        <EditorPanelProperty label={"Visuals"} value={formatCount(description.visuals)} />
        <EditorPanelProperty label={"Drawable"} value={formatCount(description.drawables)} />
        <EditorPanelProperty label={"Shader entries"} value={formatCount(description.shaderEntries)} />
      </EditorPanelSection>

      <EditorPanelSection title={"Lighting"}>
        <EditorPanelProperty label={"Level lights"} value={formatCount(description.lights)} />
        <EditorPanelProperty label={"Sun"} value={description.hasSun ? "Present" : "Absent"} />
      </EditorPanelSection>
    </EditorPanel>
  );
}
