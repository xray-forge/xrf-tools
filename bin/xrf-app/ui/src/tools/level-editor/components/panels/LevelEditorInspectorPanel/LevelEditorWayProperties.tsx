import { formatCount } from "@xrf/format";
import { ESceneObjectProperties, SceneWayPoint } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import {
  describeSceneWayType,
  formatSceneList,
  formatScenePoint,
} from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

/** One way point as the Inspector lists it: its name, its place and how many links lead on from it. */
function describeWayPoint(point: SceneWayPoint): string {
  return `${point.name} at ${formatScenePoint(point.position)}, ${formatCount(point.links.length)} links`;
}

interface ILevelEditorWayPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.WAY>;
}

/**
 * A way's type, and its points with the links leading on from each.
 */
export function LevelEditorWayProperties({ properties }: ILevelEditorWayPropertiesProps): ReactElement {
  const { points } = properties;
  const links: number = points.reduce((sum: number, point: SceneWayPoint) => sum + point.links.length, 0);

  return (
    <EditorPanelSection title={"Way"}>
      <EditorPanelProperty label={"Way type"} value={describeSceneWayType(properties.wayType)} />
      <EditorPanelProperty label={"Links"} value={formatCount(links)} />
      <EditorPanelProperty
        label={`Points (${formatCount(points.length)})`}
        value={formatSceneList(points, describeWayPoint)}
        isMonospace
      />
    </EditorPanelSection>
  );
}
