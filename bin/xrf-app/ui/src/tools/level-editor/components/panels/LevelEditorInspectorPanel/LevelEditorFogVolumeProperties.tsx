import { formatCount } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { describeSceneShape } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorFogVolumePropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.FOG_VOLUME>;
}

/**
 * The profile a fog volume emits with, the volumes it exports with, and its shape.
 */
export function LevelEditorFogVolumeProperties({ properties }: ILevelEditorFogVolumePropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Fog volume"}>
      <EditorPanelProperty label={"Profile"} value={properties.profile ?? "None, an occlusion volume"} isMonospace />
      <EditorPanelProperty label={"Export group"} value={properties.group ?? "None"} />
      <EditorPanelProperty
        label={`Shape (${formatCount(properties.shape.elements.length)})`}
        value={describeSceneShape(properties.shape)}
        isMonospace
      />
    </EditorPanelSection>
  );
}
