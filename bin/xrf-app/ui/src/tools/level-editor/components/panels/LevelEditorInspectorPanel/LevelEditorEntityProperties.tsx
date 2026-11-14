import { formatCount, formatHex } from "@xrf/format";
import { ESceneSpawnKind, SceneSpawnField } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { formatSceneList } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneSpawnKindOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorEntityPropertiesProps {
  spawn: TSceneSpawnKindOf<ESceneSpawnKind.ENTITY>;
}

/**
 * The server entity a spawn point places: its section, and its packet's fields as the scene saves them.
 */
export function LevelEditorEntityProperties({ spawn }: ILevelEditorEntityPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Spawn point"} caption={"Server entity"}>
      <EditorPanelProperty label={"Section"} value={spawn.section} isMonospace />
      <EditorPanelProperty label={"Entity flags"} value={formatHex(spawn.flags)} isMonospace />
      <EditorPanelProperty
        label={`Spawn data (${formatCount(spawn.data.length)})`}
        value={formatSceneList(spawn.data, (field: SceneSpawnField) => `${field.key} = ${field.value}`)}
        isMonospace
      />
    </EditorPanelSection>
  );
}
