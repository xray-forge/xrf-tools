import { formatCount } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { formatSceneList, formatScenePoint } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorPortalPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.PORTAL>;
}

/**
 * The two sectors a portal joins, and its corners.
 */
export function LevelEditorPortalProperties({ properties }: ILevelEditorPortalPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Portal"}>
      <EditorPanelProperty label={"Front sector"} value={properties.sectorFront} isMonospace />
      <EditorPanelProperty label={"Back sector"} value={properties.sectorBack} isMonospace />
      <EditorPanelProperty
        label={`Vertices (${formatCount(properties.vertices.length)})`}
        value={formatSceneList(properties.vertices, formatScenePoint)}
        isMonospace
      />
    </EditorPanelSection>
  );
}
