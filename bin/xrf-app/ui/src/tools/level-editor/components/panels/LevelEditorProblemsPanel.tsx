import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import {
  SceneBinaryToolsDescription,
  SceneObjectsDescription,
  SelectedSceneDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useMemo } from "react";

import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import { EditorProblemsPanel, IEditorProblem } from "@/core/shell/chrome/EditorProblemsPanel";
import { listLevelEditorProblems } from "@/tools/level-editor/lib/problems/level-editor-problems";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/** What the panel says while a part's problems are not all in: that it is read, or why it could not be. */
function toPartStatus(name: string, state: AsyncState<unknown>): Nullable<string> {
  if (state.error) {
    return `The scene's ${name} could not be read: ${state.error.message}`;
  }

  return state.value === null ? `Reading the scene's ${name}.` : null;
}

/**
 * Everything of the open scene the SDK's Level Editor would skip or refuse: what the open found of its part files,
 * then what reading its objects and its binary tools found, said to be incomplete until both are read.
 */
export function LevelEditorProblemsPanel({
  "data-testid": dataTestId = "level-editor-problems-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);

  const scene: Nullable<SelectedSceneDescription> = sceneService.scene.value;
  const objects: AsyncState<SceneObjectsDescription> = sceneService.objects.state;
  const binaryTools: AsyncState<SceneBinaryToolsDescription> = sceneService.binaryTools.state;

  const problems: Array<IEditorProblem> = useMemo(
    () =>
      listLevelEditorProblems({
        binaryTools: binaryTools.value?.problems ?? [],
        objects: objects.value?.problems ?? [],
        scene: scene?.problems ?? [],
        tools: scene?.tools ?? [],
      }),
    [binaryTools.value, objects.value, scene]
  );
  const status: Nullable<string> = toPartStatus("objects", objects) ?? toPartStatus("binary tools", binaryTools);

  if (!scene) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Problems"}>
        <EditorPanelEmpty label={"No scene open. Open one to see what the Level Editor would skip of it."} />
      </EditorPanel>
    );
  }

  return (
    <EditorProblemsPanel
      data-testid={dataTestId}
      id={id}
      className={className}
      findings={problems}
      status={status}
      emptyDescription={"Every part file of this scene reads as its tool's, and every tool's objects read."}
    />
  );
}
