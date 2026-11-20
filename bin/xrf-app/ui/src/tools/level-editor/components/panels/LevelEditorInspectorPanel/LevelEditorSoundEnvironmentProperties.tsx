import { formatCount } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { describeSceneShape } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorSoundEnvironmentPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.SOUND_ENVIRONMENT>;
}

/**
 * The reverb a volume sets inside and outside its first box, and its shape.
 */
export function LevelEditorSoundEnvironmentProperties({
  properties,
}: ILevelEditorSoundEnvironmentPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Sound environment"}>
      <EditorPanelProperty label={"Inside"} value={properties.inner || "None"} isMonospace />
      <EditorPanelProperty label={"Outside"} value={properties.outer || "None"} isMonospace />
      <EditorPanelProperty
        label={`Shape (${formatCount(properties.shape.elements.length)})`}
        value={describeSceneShape(properties.shape)}
        isMonospace
      />
    </EditorPanelSection>
  );
}
