import { formatCount } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorGroupPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.GROUP>;
}

/**
 * How many objects a group holds, each listed under its own tool with the group's name.
 */
export function LevelEditorGroupProperties({ properties }: ILevelEditorGroupPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Group"}>
      <EditorPanelProperty label={"Members"} value={formatCount(properties.members)} />
    </EditorPanelSection>
  );
}
