import { ContainerDefinition } from "@/lib/container/container-definition";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

export { LevelEditorTool as Component } from "./LevelEditorTool";

export const container: ContainerDefinition = {
  bindings: [LevelEditorSceneService, LevelEditorSelectionService],
};
