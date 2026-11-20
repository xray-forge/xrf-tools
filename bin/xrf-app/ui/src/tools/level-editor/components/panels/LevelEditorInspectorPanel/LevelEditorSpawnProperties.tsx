import { formatCount } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { describeSceneShape, describeSceneShapeUsage } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

import { LevelEditorSpawnKindProperties } from "./LevelEditorSpawnKindProperties";

interface ILevelEditorSpawnPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.SPAWN>;
}

/**
 * What a spawn point places, and the shape attached to it where its entity needs a volume.
 */
export function LevelEditorSpawnProperties({ properties }: ILevelEditorSpawnPropertiesProps): ReactElement {
  const { attached, spawn } = properties;

  return (
    <>
      <LevelEditorSpawnKindProperties spawn={spawn} />

      {attached ? (
        <EditorPanelSection title={"Attached shape"} caption={describeSceneShapeUsage(attached.usage)}>
          <EditorPanelProperty
            label={`Elements (${formatCount(attached.elements.length)})`}
            value={describeSceneShape(attached)}
            isMonospace
          />
        </EditorPanelSection>
      ) : null}
    </>
  );
}
