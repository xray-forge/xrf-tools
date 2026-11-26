import { inject, Injectable } from "@wirestate/core";
import { transformError } from "@xrf/errors";
import { call, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SceneObjectId } from "@xrf/ipc/types/xrf-app-levels";
import { Sphere } from "@xrf/ipc/types/xrf-math";
import { SessionId } from "@xrf/ipc/types/xrf-session";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import { FRAME_LEVEL_SELECTION_KEYBIND_COMMAND } from "@/core/level/commands";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

/** Frames scene objects with the camera, by where the backend says they stand. */
@Injectable()
export class LevelEditorObjectsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public constructor(
    private readonly sceneService: LevelEditorSceneService = inject(LevelEditorSceneService),
    private readonly selectionService: LevelEditorSelectionService = inject(LevelEditorSelectionService),
    private readonly renderService: LevelEditorRenderService = inject(LevelEditorRenderService)
  ) {}

  /**
   * Frames one object of the open scene by its model's or helper's sphere. An object drawing nothing, such as a sector
   * or a group, leaves the camera where it is.
   */
  @LatestFlow()
  public *goTo(id: SceneObjectId): TFlow {
    const sessionId: Nullable<SessionId> = this.sceneService.scene.sessionId;

    if (sessionId === null) {
      return;
    }

    const sphere: Nullable<Sphere> = yield* call(
      levelsCommands.locateSceneObject(sessionId, id).then(
        ({ value }) => value,
        (error: unknown) => {
          this.log.warn("The scene object is not framed:", transformError(error).message);

          return null;
        }
      )
    );

    // Another scene may have opened while the backend answered.
    if (sphere && this.sceneService.scene.sessionId === sessionId) {
      this.renderService.frameSphere(sphere);
    }
  }

  /**
   * Selects and frames one object of the open scene, as activating its Outliner row does.
   */
  public focus(id: SceneObjectId): void {
    this.selectionService.select(id);
    void this.goTo(id);
  }

  /** Frames the selection, as Unreal's `F` does. */
  @KeybindCommand(FRAME_LEVEL_SELECTION_KEYBIND_COMMAND, {
    isEnabled: (service: LevelEditorObjectsService) => service.selectionService.selectedId !== null,
  })
  public frameSelection(): void {
    const id: Nullable<SceneObjectId> = this.selectionService.selectedId;

    if (id) {
      void this.goTo(id);
    }
  }
}
