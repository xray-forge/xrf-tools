import { BoundAction, comparer, makeObservable, reaction } from "@wirestate/mobx";
import { call, cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { RenderLevelHit, RenderSelection } from "@xrf/ipc/types/xrf-renderer";
import { SessionId } from "@xrf/ipc/types/xrf-session";
import { Nullable } from "@xrf/types";

import { LevelHoverPick } from "@/core/level/lib/pick/level-hover-pick";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";
import { INativeViewportClick } from "@/core/render/lib/native/viewport/native-viewport-target";
import { toLevelEditorPick, toLevelEditorSelection } from "@/tools/level-editor/lib/pick/level-editor-pick";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

/** What the picks read from the render service that composes them. */
export interface ILevelEditorRenderPicksContext {
  getViewport: () => Nullable<NativeViewport>;
  /** The scene session shown, which every hit belongs to. */
  getSessionId: () => Nullable<SessionId>;
}

/** The services the picks read and change. */
export interface ILevelEditorRenderPicksSources {
  /** The open scene's objects, which name what a pick hits. */
  sceneService: LevelEditorSceneService;
  /** Whether helpers are drawn, which hovering needs. */
  helpersService: LevelEditorHelpersService;
  selectionService: LevelEditorSelectionService;
}

/** A viewport pick. */
export interface ILevelEditorPickRequest {
  /** In css pixels from the viewport's top left corner. */
  point: IRenderViewPoint;
  /** Whether the pick passes through glass to what stands behind it. */
  isThrough: boolean;
}

/**
 * Viewport clicks and hovers on the open scene: selection, its outline, and the hovered object, all let go with the
 * scene they were asked of.
 */
export class LevelEditorRenderPicks {
  /** The pointer's picks, one in flight at a time. */
  private readonly hovering: LevelHoverPick = new LevelHoverPick();

  public constructor(
    private readonly context: ILevelEditorRenderPicksContext,
    private readonly sources: ILevelEditorRenderPicksSources
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /**
   * Selects the listed scene object under a viewport point, or nothing. Letting go of the scene meanwhile cancels it.
   */
  @LatestFlow()
  public *pick(request: ILevelEditorPickRequest): TFlow {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();
    const { point, isThrough } = request;

    if (!viewport || !this.context.getSessionId()) {
      return;
    }

    const hit: Nullable<RenderLevelHit> = yield* call(viewport.pick({ x: point.x, y: point.y, isThrough }));
    const { sceneService, selectionService } = this.sources;

    selectionService.select(toLevelEditorPick(hit, sceneService.objects.value));
  }

  /**
   * Notes the scene object under a viewport point as hovered, one pick in flight at a time ({@link LevelHoverPick}).
   *
   * @param point - In css pixels from the viewport's top left corner; null for nothing hovered.
   */
  @LatestFlow()
  public *hover(point: Nullable<IRenderViewPoint>): TFlow {
    const { sceneService, selectionService } = this.sources;

    if (!point || !this.context.getViewport() || !this.context.getSessionId()) {
      selectionService.noteHovered(null);

      return;
    }

    // The pick in flight may have outlived the viewport or the scene it was asked of.
    const hit: Nullable<RenderLevelHit> = yield* this.hovering.pick(point, () =>
      this.context.getSessionId() ? this.context.getViewport() : null
    );

    selectionService.noteHovered(toLevelEditorPick(hit, sceneService.objects.value));
  }

  /**
   * Follows the selection's outline in a just-attached viewport, and clears the hover while helpers are off.
   *
   * @returns The disposers that stop following.
   */
  public start(viewport: NativeViewport): Array<() => void> {
    const { helpersService, selectionService } = this.sources;

    return [
      reaction(
        (): Nullable<RenderSelection> => toLevelEditorSelection(selectionService.selectedId),
        (selection: Nullable<RenderSelection>) => viewport.setSelection(selection),
        { equals: comparer.structural, fireImmediately: true }
      ),
      // Nothing is hovered while helpers are off, so none is lifted the moment they come back.
      reaction(
        () => helpersService.helpers.isShown,
        (isShown: boolean) => {
          if (!isShown) {
            void this.hover(null);
          }
        }
      ),
    ];
  }

  public click(click: INativeViewportClick): void {
    void this.pick({ isThrough: click.isThrough, point: click.point });
  }

  /**
   * Hovers where the pointer rests while helpers are drawn, and nothing otherwise.
   *
   * @param point - Null once the pointer leaves.
   */
  public rest(point: Nullable<IRenderViewPoint>): void {
    void this.hover(this.sources.helpersService.helpers.isShown ? point : null);
  }

  /** Cancels the picks asked of the scene shown, which name it alone, and clears its hover. */
  @BoundAction()
  public leave(): void {
    cancelFlow(this, "pick");
    cancelFlow(this, "hover");
    this.hovering.reset();
    this.sources.selectionService.noteHovered(null);
  }
}
