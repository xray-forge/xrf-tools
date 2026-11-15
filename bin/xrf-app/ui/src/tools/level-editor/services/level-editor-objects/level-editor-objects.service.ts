import { inject, Injectable } from "@wirestate/core";
import { transformError } from "@xrf/errors";
import { call, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { SceneObjectId } from "@xrf/ipc/types/xrf-app-levels";
import { Sphere } from "@xrf/ipc/types/xrf-math";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import { FRAME_LEVEL_SELECTION_KEYBIND_COMMAND } from "@/core/level/commands";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

/** What the editor does with the scene object chosen: framing it with the camera, by where the backend says it stands. */
@Injectable()
export class LevelEditorObjectsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public constructor(
    private readonly sceneService: LevelEditorSceneService = inject(LevelEditorSceneService),
    private readonly selectionService: LevelEditorSelectionService = inject(LevelEditorSelectionService),
    private readonly renderService: LevelEditorRenderService = inject(LevelEditorRenderService)
  ) {}

  /**
   * Stands the camera framing one object of the open scene, by the sphere it is drawn in: a scene object's model or a
   * helped object's helper. An object that draws nothing, a sector or a group, leaves the camera where it is.
   *
   * @param id - The object, by its tool and index.
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

    // Another scene opened while the backend answered: the object is not where the camera now is.
    if (sphere && this.sceneService.scene.sessionId === sessionId) {
      this.renderService.frameSphere(sphere);
    }
  }

  /**
   * Selects one object of the open scene and frames it, as activating its Outliner row does.
   *
   * @param id - The object, by its tool and index.
   */
  public focus(id: SceneObjectId): void {
    this.selectionService.select(id);
    void this.goTo(id);
  }

  /** Stands the camera framing what is selected, as Unreal's `F` does. */
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
