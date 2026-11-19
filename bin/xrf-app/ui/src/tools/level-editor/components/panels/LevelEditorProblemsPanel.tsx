import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import {
  SceneBinaryToolsDescription,
  SceneDrawDescription,
  SceneHelpersDescription,
  SceneObjectsDescription,
  SelectedSceneDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { IEditorProblem } from "@xrf/ui/editor/problems";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useMemo } from "react";

import { describeLevelResidentWait } from "@/core/level/lib/load/level-resident-wait";
import { EMPTY_LEVEL_RESIDENT, ILevelResident } from "@/core/level/lib/render/level-resident";
import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import { EditorProblemsPanel } from "@/core/shell/chrome/EditorProblemsPanel";
import { listLevelEditorProblems } from "@/tools/level-editor/lib/problems/level-editor-problems";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/** What the panel says while a part is unread: that it is being read, or why it could not be; null once read. */
function toPartStatus(name: string, state: AsyncState<unknown>): Nullable<string> {
  if (state.error) {
    return `The scene's ${name} could not be read: ${state.error.message}`;
  }

  return state.value === null ? `Reading the scene's ${name}.` : null;
}

/**
 * What the SDK's Level Editor would skip or refuse in the open scene, and what the renderer does not draw, in the order
 * {@link listLevelEditorProblems} gives.
 *
 * Lists the findings read so far, with a status for the first part still being read or failed, and notes that render
 * problems wait for the scene to be resident.
 */
export function LevelEditorProblemsPanel({
  "data-testid": dataTestId = "level-editor-problems-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);
  const renderService: LevelEditorRenderService = useInjection(LevelEditorRenderService);

  const scene: Nullable<SelectedSceneDescription> = sceneService.scene.value;
  const objects: AsyncState<SceneObjectsDescription> = sceneService.objects.state;
  const binaryTools: AsyncState<SceneBinaryToolsDescription> = sceneService.binaryTools.state;
  const drawing: AsyncState<SceneDrawDescription> = sceneService.drawing.state;
  const helpers: Nullable<SceneHelpersDescription> = sceneService.helpers.value;
  const resident: ILevelResident = renderService.resident.value ?? EMPTY_LEVEL_RESIDENT;

  const problems: Array<IEditorProblem> = useMemo(
    () =>
      listLevelEditorProblems({
        binaryTools: binaryTools.value?.problems ?? [],
        drawing: drawing.value?.problems ?? [],
        models: resident.problems.models,
        objectTools: objects.value?.tools ?? [],
        objects: objects.value?.problems ?? [],
        scene: scene?.problems ?? [],
        spawnModels: helpers?.spawnModels ?? [],
        textures: resident.textures.problems,
        tools: scene?.tools ?? [],
      }),
    [binaryTools.value, drawing.value, helpers, objects.value, resident, scene]
  );
  const status: Nullable<string> =
    toPartStatus("objects", objects) ??
    toPartStatus("binary tools", binaryTools) ??
    toPartStatus("library objects", drawing) ??
    toPartStatus("helpers", sceneService.helpers.state) ??
    describeLevelResidentWait(renderService.resident.state, "what it could not draw", "the scene");

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
      emptyDescription={
        "Every part file of this scene reads as its tool's, every tool's objects read, and every scene object draws " +
        "with its textures."
      }
    />
  );
}
