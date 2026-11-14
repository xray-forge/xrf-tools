import { formatCount } from "@xrf/format";
import { ESceneObjectProperties, SceneSectorItem } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { formatSceneColor, formatSceneList } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorSectorPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.SECTOR>;
}

/**
 * What a sector is drawn in, the map it switches to, and the meshes it holds.
 */
export function LevelEditorSectorProperties({ properties }: ILevelEditorSectorPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Sector"}>
      <EditorPanelProperty label={"Default"} value={properties.isDefault ? "Yes" : "No"} />
      <EditorPanelProperty label={"Colour"} value={formatSceneColor(properties.color, 0)} isMonospace />
      <EditorPanelProperty label={"Level map"} value={properties.mapIndex ?? "None"} />
      <EditorPanelProperty
        label={`Meshes (${formatCount(properties.items.length)})`}
        value={formatSceneList(properties.items, (item: SceneSectorItem) => `${item.object} · ${item.mesh}`)}
        isMonospace
      />
    </EditorPanelSection>
  );
}
