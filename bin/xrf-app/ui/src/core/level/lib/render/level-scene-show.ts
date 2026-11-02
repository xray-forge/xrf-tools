import { BoundAction, makeObservable, RefObservable } from "@wirestate/mobx";
import { RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { ILevelScene } from "@/core/level/lib/load/level-scene";

/** What the owner of a level's viewport does as the scene it shows changes hands. */
export interface ILevelSceneShowHooks {
  /** The scene shown is let go: another level opened, the level streamed afresh, or the viewport gone. */
  onLeft: () => void;
  /** The level is resident whole and revealed: once per scene it is drawn as. */
  onResident: () => void;
}

/**
 * Which scene of a native viewport shows the open level, and whether it is revealed: a level is shown only once its
 * scene is resident whole, so it never assembles in front of whoever opened it. A level read again in its session moves
 * to the scene the backend streams it as afresh, and is revealed again once that scene is ready.
 *
 * Load reports name the scene they count, which may be one shown before: only those of the scene followed reveal.
 */
export class LevelSceneShow {
  /** Whether the open level is revealed, drawn with everything it opens with. */
  @RefObservable()
  public isRevealed: boolean = false;

  /** The session of the level open, whose later scenes the backend streams it as by itself. */
  private sessionId: Nullable<string> = null;
  /** The scene the level open is drawn as, once the renderer has named it. */
  private scene: Nullable<number> = null;
  /** The scene shown before the level was streamed afresh, until the scene it streams as is ready; null otherwise. */
  private followed: Nullable<number> = null;
  /** The scene of the open level last followed, whose next move is waited for anew. */
  private shownScene: Nullable<ILevelScene> = null;
  /** The last load report, of whichever scene the viewport drew. */
  private loaded: Nullable<RenderLoadReport> = null;

  /**
   * @param hooks - What the owner does as the scene shown is let go and as the level is resident.
   */
  public constructor(private readonly hooks: ILevelSceneShowHooks) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /**
   * Opens a level, concealed until the scene the renderer names for it (`noteShown`) is resident whole. A level read
   * again in the session shown stays shown: the scene it streams as moves apart (`move`).
   *
   * @param sessionId - The level's session, or null for none open.
   * @param scene - The scene that session streams as now, whose next move is the one followed.
   * @returns Whether the level opened anew, which the owner then shows; false for the session shown already.
   */
  @BoundAction()
  public open(sessionId: Nullable<string>, scene: Nullable<ILevelScene>): boolean {
    if (sessionId !== null && this.isShowing(sessionId)) {
      return false;
    }

    this.hooks.onLeft();
    this.sessionId = sessionId;
    this.shownScene = scene;
    this.scene = null;
    this.followed = null;
    this.isRevealed = false;

    return true;
  }

  /**
   * @param scene - The scene the renderer draws the level opened as, or null for nothing shown.
   */
  @BoundAction()
  public noteShown(scene: Nullable<number>): void {
    this.scene = scene;
    this.revealWhenReady();
  }

  /**
   * Follows the level shown into the scene the backend streams it as afresh, in its session, concealed until that scene
   * is ready. A scene of another level, or the one the level opened as, is the opening's own.
   *
   * @param scene - The scene the open level streams as now.
   */
  @BoundAction()
  public move(scene: Nullable<ILevelScene>): void {
    if (!scene || scene === this.shownScene || !this.isShowing(scene.sessionId)) {
      return;
    }

    this.hooks.onLeft();
    this.shownScene = scene;
    this.followed = this.scene ?? this.followed;
    this.scene = null;
    this.isRevealed = false;

    // The renderer reports the scene it streams as from the moment the backend moved it, which can be before now.
    if (this.loaded) {
      this.noteLoad(this.loaded);
    }
  }

  /**
   * @param report - How far the viewport has loaded the scene it draws.
   */
  @BoundAction()
  public noteLoad(report: RenderLoadReport): void {
    this.loaded = report;

    // A level read again streams as a scene of its own, which the viewport's reports name once it shows it.
    if (this.followed !== null && report.scene !== this.followed) {
      this.scene = report.scene;
    }

    this.revealWhenReady();
  }

  /** Forgets the level shown, with the viewport that showed it. */
  @BoundAction()
  public release(): void {
    this.hooks.onLeft();
    this.sessionId = null;
    this.shownScene = null;
    this.scene = null;
    this.followed = null;
    this.loaded = null;
    this.isRevealed = false;
  }

  /**
   * @param sessionId - The level session a snapshot belongs to.
   * @returns Whether that session is shown already, its scene named or a level read again on its way.
   */
  private isShowing(sessionId: string): boolean {
    return this.sessionId === sessionId && (this.scene !== null || this.followed !== null);
  }

  /**
   * Reveals the level once its scene is resident whole; a report of the level drawn before it, still loading when this
   * one was asked for, reveals nothing, and a scene revealed stays so whatever its later reports count.
   */
  private revealWhenReady(): void {
    const report: Nullable<RenderLoadReport> = this.loaded;

    if (
      !this.isRevealed &&
      report?.isReady &&
      this.sessionId !== null &&
      this.scene !== null &&
      report.scene === this.scene
    ) {
      this.isRevealed = true;
      this.followed = null;
      this.hooks.onResident();
    }
  }
}
