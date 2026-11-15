import { BoundAction, comparer, makeObservable, reaction } from "@wirestate/mobx";
import { call, cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { RenderLevelHit, RenderSelection } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { LevelHoverPick } from "@/core/level/lib/pick/level-hover-pick";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";
import { INativeViewportClick } from "@/core/render/lib/native/viewport/native-viewport-target";
import { toLevelEditorPick, toLevelEditorSelection } from "@/tools/level-editor/lib/pick/level-editor-pick";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

/** What the scene's picks ask of the render service that composes them. */
export interface ILevelEditorRenderPicksContext {
  /** The viewport attached, or null for none. */
  getViewport: () => Nullable<NativeViewport>;
  /** The scene session shown, which what a pick hits is one of, or null for none. */
  getSessionId: () => Nullable<SessionId>;
}

/** What the scene's picks name and change. */
export interface ILevelEditorRenderPicksSources {
  /** The open scene's objects, which name what a pick hits. */
  sceneService: LevelEditorSceneService;
  /** Whether helpers are drawn, which hovering needs. */
  helpersService: LevelEditorHelpersService;
  /** What is selected and hovered. */
  selectionService: LevelEditorSelectionService;
}

/** A pick asked of the viewport: where, and whether glass is passed by. */
export interface ILevelEditorPickRequest {
  /** In css pixels from the viewport's top left corner. */
  point: IRenderViewPoint;
  /** Whether glass is passed by, picking what stands behind it. */
  isThrough: boolean;
}

/**
 * What of the open scene is drawn where the viewport is clicked or hovered, and what it changes: the scene object
 * selected, outlined where it is drawn, and the one hovered; let go with the scene they were asked of.
 */
export class LevelEditorRenderPicks {
  /** The pointer's picks, one in flight at a time. */
  private readonly hovering: LevelHoverPick = new LevelHoverPick();

  /**
   * @param context - The viewport and the scene shown.
   * @param sources - What the picks name and change.
   */
  public constructor(
    private readonly context: ILevelEditorRenderPicksContext,
    private readonly sources: ILevelEditorRenderPicksSources
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /**
   * Selects the scene object drawn under a point of the viewport, or nothing where none the scene lists is. The scene
   * shown let go meanwhile cancels it, as what it hit names something else now.
   *
   * @param request - Where, and whether glass is passed by.
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
   * Notes the scene object under a point of the viewport as hovered, with one hover pick in flight at a time
   * ({@link LevelHoverPick}).
   *
   * @param point - Where, in css pixels from the viewport's top left corner; null for nothing hovered.
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
   * Outlines what is selected in a viewport just attached, and lets go of what is hovered while helpers are off.
   *
   * @param viewport - The viewport just attached.
   * @returns What stops following them.
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

  /**
   * Picks what a click meets.
   *
   * @param click - Where it landed, and whether glass is passed by.
   */
  public click(click: INativeViewportClick): void {
    void this.pick({ isThrough: click.isThrough, point: click.point });
  }

  /**
   * Hovers where the pointer rests while helpers are drawn, and nothing otherwise.
   *
   * @param point - Where, in css pixels from the viewport's top left corner; null once it leaves.
   */
  public rest(point: Nullable<IRenderViewPoint>): void {
    void this.hover(this.sources.helpersService.helpers.isShown ? point : null);
  }

  /** Lets go of the picks asked of the scene shown, which name it alone, and of what is hovered in it. */
  @BoundAction()
  public leave(): void {
    cancelFlow(this, "pick");
    cancelFlow(this, "hover");
    this.hovering.reset();
    this.sources.selectionService.noteHovered(null);
  }
}
