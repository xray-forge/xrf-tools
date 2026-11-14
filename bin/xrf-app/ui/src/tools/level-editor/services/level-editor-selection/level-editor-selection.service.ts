import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import {
  SceneObjectDescription,
  SceneObjectId,
  SceneObjectsDescription,
  SceneObjectToolDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Maybe, Nullable } from "@xrf/types";

import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/** The object chosen, with the opening of the scene it was chosen in. */
interface ILevelEditorSelection {
  sessionId: SessionId;
  id: SceneObjectId;
}

/**
 * Which object of the open scene is selected, by its tool and index. Kept with the scene it was chosen in and with the
 * objects listed of it, so a scene opened or closed since, or a listing read again without the object, selects
 * nothing.
 */
@Injectable()
export class LevelEditorSelectionService {
  @Observable()
  private selection: Nullable<ILevelEditorSelection> = null;

  public constructor(private readonly sceneService: LevelEditorSceneService = inject(LevelEditorSceneService)) {}

  /** The selected object as the open scene's listing describes it, or none. */
  @Computed()
  public get selected(): Nullable<SceneObjectDescription> {
    const selection: Nullable<ILevelEditorSelection> = this.selection;
    const objects: Nullable<SceneObjectsDescription> = this.sceneService.objects.value;

    if (
      !selection ||
      !objects ||
      selection.sessionId !== this.sceneService.scene.sessionId ||
      selection.sessionId !== this.sceneService.objects.snapshot?.sessionId
    ) {
      return null;
    }

    const tool: Maybe<SceneObjectToolDescription> = objects.tools.find(
      (it: SceneObjectToolDescription) => it.tool === selection.id.tool
    );

    return tool?.objects.find((it: SceneObjectDescription) => it.id.index === selection.id.index) ?? null;
  }

  /** The selected object's id, or none while nothing the open scene lists is selected. */
  @Computed()
  public get selectedId(): Nullable<SceneObjectId> {
    return this.selected?.id ?? null;
  }

  /**
   * Selects one object of the open scene, or nothing.
   *
   * @param id - The object, by its tool and index; none to select nothing.
   */
  @BoundAction()
  public select(id: Nullable<SceneObjectId>): void {
    const sessionId: Nullable<SessionId> = this.sceneService.scene.sessionId;

    this.selection = id && sessionId ? { id, sessionId } : null;
  }
}
