import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { ContainerDefinition } from "@/lib/container/container-definition";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

export { LevelEditorTool as Component } from "./LevelEditorTool";

export const container: ContainerDefinition = {
  bindings: [
    LevelEditorSceneService,
    LevelEditorSelectionService,
    LevelEditorHelpersService,
    LevelViewService,
    LevelViewportService,
    LevelEditorRenderService,
  ],
};
