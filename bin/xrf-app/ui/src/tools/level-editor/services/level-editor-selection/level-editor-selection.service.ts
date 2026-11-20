import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, Observable, RefObservable } from "@wirestate/mobx";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { SceneObjectDescription, SceneObjectId, SceneObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

import { findSceneObject } from "@/tools/level-editor/lib/tools/scene-objects";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/** The selected object, with the session it was selected in. */
interface ILevelEditorSelection {
  sessionId: SessionId;
  id: SceneObjectId;
}

/** The hidden objects, with the session they were hidden in. */
interface ILevelEditorHidden {
  sessionId: SessionId;
  ids: ReadonlyArray<SceneObjectId>;
}

function isSameObject(first: SceneObjectId, second: SceneObjectId): boolean {
  return first.tool === second.tool && first.index === second.index;
}

/**
 * The open scene's selected, hovered and hidden objects.
 *
 * Selection and hiding are tied to the session they were made in, so another open drops them; the selection also
 * drops once a re-read listing lacks the object.
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

  /** The selected object as the open scene's listing describes it. */
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

  @Computed()
  public get selectedId(): Nullable<SceneObjectId> {
    return this.selected?.id ?? null;
  }

  /** The open scene's hidden objects, in the order they were hidden. */
  @Computed()
  public get hidden(): ReadonlyArray<SceneObjectId> {
    const hiding: Nullable<ILevelEditorHidden> = this.hiding;

    return hiding && hiding.sessionId === this.sceneService.scene.sessionId ? hiding.ids : [];
  }

  /**
   * Selects one object of the open scene; null selects nothing.
   */
  @BoundAction()
  public select(id: Nullable<SceneObjectId>): void {
    const sessionId: Nullable<SessionId> = this.sceneService.scene.sessionId;

    this.selection = id && sessionId ? { id, sessionId } : null;
  }

  @BoundAction()
  public noteHovered(id: Nullable<SceneObjectId>): void {
    if (this.hovered === null || id === null || !isSameObject(this.hovered, id)) {
      this.hovered = id;
    }
  }

  public isHidden(id: SceneObjectId): boolean {
    return this.hidden.some((it: SceneObjectId) => isSameObject(it, id));
  }

  /**
   * Hides or shows one object of the open scene, as an Outliner row's eye does.
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

  /** Shows every hidden object again. */
  @BoundAction()
  public revealAll(): void {
    this.hiding = null;
  }
}
