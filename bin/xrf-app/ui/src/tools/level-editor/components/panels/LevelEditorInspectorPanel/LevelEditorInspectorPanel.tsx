import { useInjection } from "@wirestate/react";
import { formatHex } from "@xrf/format";
import { SceneObjectDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty, EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import {
  formatScenePoint,
  formatSceneRotation,
  formatSceneScale,
} from "@/tools/level-editor/lib/inspector/scene-object-format";
import { SCENE_TOOL_LABELS } from "@/tools/level-editor/lib/tools/scene-tools";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

import { LevelEditorObjectProperties } from "./LevelEditorObjectProperties";

/**
 * What the selected scene object is: its name, tool and group, where it stands, and what its kind keeps.
 */
export function LevelEditorInspectorPanel({
  "data-testid": dataTestId = "level-editor-inspector-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);
  const selectionService: LevelEditorSelectionService = useInjection(LevelEditorSelectionService);

  const object: Nullable<SceneObjectDescription> = selectionService.selected;

  if (!object) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Inspector"}>
        <EditorPanelEmpty
          label={
            sceneService.scene.value
              ? "Nothing selected. Pick an object in the Outliner."
              : "No scene open. Open one to inspect its objects."
          }
        />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Inspector"}>
      <EditorPanelSection title={object.name} caption={SCENE_TOOL_LABELS[object.id.tool]}>
        {object.group ? <EditorPanelProperty label={"Group"} value={object.group} /> : null}
        <EditorPanelProperty label={"Position"} value={formatScenePoint(object.transform.position)} />
        <EditorPanelProperty label={"Rotation"} value={formatSceneRotation(object.transform.rotation)} />
        <EditorPanelProperty label={"Scale"} value={formatSceneScale(object.transform.scale)} />
        <EditorPanelProperty label={"Flags"} value={formatHex(object.flags)} isMonospace />
      </EditorPanelSection>

      <LevelEditorObjectProperties properties={object.properties} />
    </EditorPanel>
  );
}
