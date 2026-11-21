import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { IContainerDefinition } from "@/lib/container/container-definition";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorObjectsService } from "@/tools/level-editor/services/level-editor-objects";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

export { LevelEditorTool as Component } from "./LevelEditorTool";

export const container: IContainerDefinition = {
  bindings: [
    LevelEditorSceneService,
    LevelEditorSelectionService,
    LevelEditorHelpersService,
    LevelViewService,
    LevelViewportService,
    LevelEditorRenderService,
    LevelEditorObjectsService,
  ],
};
