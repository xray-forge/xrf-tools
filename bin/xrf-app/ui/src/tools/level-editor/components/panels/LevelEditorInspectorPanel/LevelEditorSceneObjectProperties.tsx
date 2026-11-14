import { formatHex } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorSceneObjectPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.SCENE_OBJECT>;
}

/**
 * The library object a scene object places, and whether the objects roots the scene opened with hold it.
 */
export function LevelEditorSceneObjectProperties({ properties }: ILevelEditorSceneObjectPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Library object"}>
      <EditorPanelProperty label={"Reference"} value={properties.reference} isMonospace />
      <EditorPanelProperty
        label={"Object file"}
        value={properties.isFound ? "Found in the objects roots" : "Not found in the objects roots"}
      />
      <EditorPanelProperty label={"Object flags"} value={formatHex(properties.flags)} isMonospace />
    </EditorPanelSection>
  );
}
