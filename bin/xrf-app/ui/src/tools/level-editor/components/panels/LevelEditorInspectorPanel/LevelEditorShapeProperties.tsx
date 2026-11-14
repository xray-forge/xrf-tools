import { formatCount } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { describeSceneShape, describeSceneShapeUsage } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorShapePropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.SHAPE>;
}

/**
 * What a shape is used for, and its spheres and boxes.
 */
export function LevelEditorShapeProperties({ properties }: ILevelEditorShapePropertiesProps): ReactElement {
  const { shape } = properties;

  return (
    <EditorPanelSection title={"Shape"} caption={describeSceneShapeUsage(shape.usage)}>
      <EditorPanelProperty
        label={`Elements (${formatCount(shape.elements.length)})`}
        value={describeSceneShape(shape)}
        isMonospace
      />
    </EditorPanelSection>
  );
}
