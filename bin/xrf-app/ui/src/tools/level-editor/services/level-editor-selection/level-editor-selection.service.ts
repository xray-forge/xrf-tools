import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, Observable, RefObservable } from "@wirestate/mobx";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { SceneObjectDescription, SceneObjectId, SceneObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

import { findSceneObject } from "@/tools/level-editor/lib/tools/scene-objects";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/** The object chosen, with the opening of the scene it was chosen in. */
interface ILevelEditorSelection {
  sessionId: SessionId;
  id: SceneObjectId;
}

/** The objects hidden, with the opening of the scene they were hidden in. */
interface ILevelEditorHidden {
  sessionId: SessionId;
  ids: ReadonlyArray<SceneObjectId>;
}

/**
 * @param first - A scene object.
 * @param second - Another.
 * @returns Whether both name the same object.
 */
function isSameObject(first: SceneObjectId, second: SceneObjectId): boolean {
  return first.tool === second.tool && first.index === second.index;
}

/**
 * Which objects of the open scene are singled out, each by its tool and index: the one selected, the one under the
 * pointer, and those hidden one by one. The selection and the hidden objects are kept with the scene's opening they
 * were chosen in, so a scene opened or closed since selects and hides nothing; the selection is kept with the objects
 * listed of it too, so a listing read again without the object selects nothing.
 */
@Injectable()
export class LevelEditorSelectionService {
  /** The object under the pointer while helpers are drawn, whose helper is lifted; null for none. */
  @RefObservable()
  public hovered: Nullable<SceneObjectId> = null;

  @Observable()
  private selection: Nullable<ILevelEditorSelection> = null;

  @RefObservable()
  private hiding: Nullable<ILevelEditorHidden> = null;

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

    return findSceneObject(objects.tools, selection.id);
  }

  /** The selected object's id, or none while nothing the open scene lists is selected. */
  @Computed()
  public get selectedId(): Nullable<SceneObjectId> {
    return this.selected?.id ?? null;
  }

  /** The objects of the open scene hidden one by one, in the order they were hidden; none for another opening's. */
  @Computed()
  public get hidden(): ReadonlyArray<SceneObjectId> {
    const hiding: Nullable<ILevelEditorHidden> = this.hiding;

    return hiding && hiding.sessionId === this.sceneService.scene.sessionId ? hiding.ids : [];
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

  /**
   * @param id - The object under the pointer, or null for none.
   */
  @BoundAction()
  public noteHovered(id: Nullable<SceneObjectId>): void {
    if (this.hovered === null || id === null || !isSameObject(this.hovered, id)) {
      this.hovered = id;
    }
  }

  /**
   * @param id - An object of the open scene.
   * @returns Whether it is hidden.
   */
  public isHidden(id: SceneObjectId): boolean {
    return this.hidden.some((it: SceneObjectId) => isSameObject(it, id));
  }

  /**
   * Hides one object of the open scene, or shows it again where it is hidden, as an Outliner row's eye does.
   *
   * @param id - The object.
   */
  @BoundAction()
  public toggleHidden(id: SceneObjectId): void {
    const sessionId: Nullable<SessionId> = this.sceneService.scene.sessionId;

    if (!sessionId) {
      return;
    }

    const hidden: ReadonlyArray<SceneObjectId> = this.hidden;

    this.hiding = {
      ids: this.isHidden(id) ? hidden.filter((it: SceneObjectId) => !isSameObject(it, id)) : [...hidden, id],
      sessionId,
    };
  }

  /** Shows every object of the open scene hidden again. */
  @BoundAction()
  public revealAll(): void {
    this.hiding = null;
  }
}
