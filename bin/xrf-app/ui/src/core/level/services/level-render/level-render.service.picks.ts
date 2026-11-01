import { BoundAction, comparer, makeObservable, reaction } from "@wirestate/mobx";
import { call, cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { toLevelFocusDistance } from "@/core/level/lib/features/level-feature-options";
import {
  ELevelPick,
  ILevelPickParts,
  relocateLevelObjects,
  relocateLevelPick,
  TLevelPick,
  toLevelPick,
} from "@/core/level/lib/pick/level-pick";
import { toLevelPickSelection } from "@/core/level/lib/pick/level-pick-selection";
import { LevelLoadService } from "@/core/level/services/level-load";
import { ILevelRenderContext } from "@/core/level/services/level-render/level-render.service.context";
import { LevelViewService } from "@/core/level/services/level-view";
import { ELevelSelectionMode, LevelViewportService } from "@/core/level/services/level-viewport";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";
import { INativeViewportClick } from "@/core/render/lib/native/viewport/native-viewport-target";

/** A pick asked of the viewport: where, whether glass is passed by, and how what it picks changes the selection. */
export interface ILevelPickRequest {
  /** In css pixels from the viewport's top left corner. */
  point: IRenderViewPoint;
  /** Whether glass is passed by, picking what stands behind it. */
  isThrough: boolean;
  /** Replacing the selection where not given. */
  mode?: ELevelSelectionMode;
}

/** What the level's picks name and change. */
export interface ILevelRenderPicksSources {
  /** The open level's parts, which name what a pick hits. */
  loadService: LevelLoadService;
  /** Whether helpers are drawn, which hovering needs, and the lens a focus pick moves. */
  viewService: LevelViewService;
  /** What is picked, selected, hidden and hovered. */
  viewportService: LevelViewportService;
}

/**
 * What of the level is drawn where the viewport is clicked or hovered, and what it changes: the selection, the hovered
 * object, or the lens's focus; kept across the level's parts read anew, and let go with the scene they were asked of.
 */
export class LevelRenderPicks {
  /** The hover pick in flight, settled either way, which the next hover waits for; null while none is. */
  private hovering: Nullable<Promise<void>> = null;

  /**
   * @param context - The viewport and the level open.
   * @param sources - What the picks name and change.
   */
  public constructor(
    private readonly context: ILevelRenderContext,
    private readonly sources: ILevelRenderPicksSources
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /**
   * Says what of the open level is drawn under a point of the viewport, which is then what is selected. The scene shown
   * let go meanwhile cancels it, as what it hit names something else now.
   *
   * @param request - Where, whether glass is passed by, and how what it picks changes the selection.
   */
  @LatestFlow()
  public *pick(request: ILevelPickRequest): TFlow {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();
    const { point, isThrough, mode = ELevelSelectionMode.REPLACE } = request;

    if (!viewport || !this.context.getLevel()) {
      return;
    }

    const hit: Nullable<RenderLevelHit> = yield* call(viewport.pick({ x: point.x, y: point.y, isThrough }));

    this.sources.viewportService.notePicked(hit ? toLevelPick(hit, this.sources.loadService.objects) : null, mode);
  }

  /**
   * Notes the level object under a point of the viewport as hovered, picking at once with one hover pick in flight at a
   * time: a point arriving meanwhile waits for it and is picked next, and a newer one drops the point waiting, so the
   * pointer is answered as fast as a pick comes back and never queues the renderer's readbacks.
   *
   * @param point - Where, in css pixels from the viewport's top left corner; null for nothing hovered.
   */
  @LatestFlow()
  public *hover(point: Nullable<IRenderViewPoint>): TFlow {
    if (!point || !this.context.getViewport() || !this.context.getLevel()) {
      this.sources.viewportService.noteHovered(null);

      return;
    }

    if (this.hovering) {
      yield* call(this.hovering);
    }

    // The pick in flight may have outlived the viewport or the level it was asked of.
    const viewport: Nullable<NativeViewport> = this.context.getViewport();

    if (!viewport || !this.context.getLevel()) {
      return;
    }

    const picking: Promise<Nullable<RenderLevelHit>> = viewport.pick({ x: point.x, y: point.y, isThrough: false });

    const settled: Promise<void> = picking
      .then(
        () => undefined,
        () => undefined
      )
      .then(() => {
        if (this.hovering === settled) {
          this.hovering = null;
        }
      });

    this.hovering = settled;

    const hit: Nullable<RenderLevelHit> = yield* call(picking);
    const pick: Nullable<TLevelPick> = hit ? toLevelPick(hit, this.sources.loadService.objects) : null;

    this.sources.viewportService.noteHovered(pick?.kind === ELevelPick.OBJECT ? pick.object : null);
  }

  /**
   * Focuses the view's lens on what is drawn under a point of the viewport, its depth of field turned on, and stops
   * picking the focus; a click meeting nothing of the level leaves the focus as it was.
   *
   * @param request - Where, and whether glass is passed by, focusing on what stands behind it.
   */
  @LatestFlow()
  public *focusAt(request: Omit<ILevelPickRequest, "mode">): TFlow {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();
    const { point, isThrough } = request;
    const { viewService, viewportService } = this.sources;

    viewportService.setFocusPicking(false);

    if (!viewport || !this.context.getLevel()) {
      return;
    }

    const hit: Nullable<RenderLevelHit> = yield* call(viewport.pick({ x: point.x, y: point.y, isThrough }));
    const depth: Nullable<number> = hit?.depth ?? null;

    if (depth !== null && depth > 0) {
      viewService.setFeatures({
        ...viewService.features,
        camera: { ...viewService.features.camera, focusDistance: toLevelFocusDistance(depth), isDefocused: true },
      });
    }
  }

  /**
   * @param object - A spawned object, by its index among the level's.
   * @returns Its bounding sphere in renderer space, centre then radius; null until its model is drawn or with no
   *   viewport attached.
   */
  public async locateSpawnObject(object: number): Promise<Nullable<[number, number, number, number]>> {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();

    return viewport ? viewport.locateSpawnObject(object) : null;
  }

  /**
   * Tells a viewport just attached what is selected, keeps the selection across the level's parts read anew, and lets
   * go of what is hovered while helpers are off.
   *
   * @param viewport - The viewport just attached.
   * @returns What stops following them.
   */
  public start(viewport: NativeViewport): Array<() => void> {
    const { loadService, viewService, viewportService } = this.sources;

    return [
      reaction(
        (): ILevelPickParts => ({ objects: loadService.objects, surfaces: loadService.surfaces.value }),
        this.relocate,
        { equals: comparer.shallow }
      ),
      reaction(
        () => viewportService.picked,
        (picked: Nullable<TLevelPick>) => viewport.setSelection(toLevelPickSelection(picked)),
        { fireImmediately: true }
      ),
      // Nothing is hovered while helpers are off, so none is lifted the moment they come back.
      reaction(
        () => viewService.helpers.isShown,
        (isShown: boolean) => {
          if (!isShown) {
            void this.hover(null);
          }
        }
      ),
    ];
  }

  /**
   * Focuses the lens where a click lands while the focus is picked, and otherwise picks what it meets.
   *
   * @param click - Where it landed, and the keys held with it.
   */
  public click(click: INativeViewportClick): void {
    const { point, isThrough, isToggling } = click;

    if (this.sources.viewportService.isFocusPicking) {
      void this.focusAt({ isThrough, point });
    } else {
      void this.pick({
        isThrough,
        mode: isToggling ? ELevelSelectionMode.TOGGLE : ELevelSelectionMode.REPLACE,
        point,
      });
    }
  }

  /**
   * Hovers where the pointer rests while helpers are drawn, and nothing otherwise.
   *
   * @param point - Where, in css pixels from the viewport's top left corner; null once it leaves.
   */
  public rest(point: Nullable<IRenderViewPoint>): void {
    void this.hover(this.sources.viewService.helpers.isShown ? point : null);
  }

  /** Lets go of the picks asked of the scene shown, which name it alone, and of what is hovered in it. */
  @BoundAction()
  public leave(): void {
    cancelFlow(this, "pick");
    cancelFlow(this, "focusAt");
    cancelFlow(this, "hover");
    this.hovering = null;
    this.sources.viewportService.noteHovered(null);
  }

  /**
   * Keeps what is picked, selected and hidden across the open level's parts read anew, each found by what it is, or
   * drops it with what it named.
   *
   * @param after - The parts as read now.
   * @param before - The parts it was picked among.
   */
  @BoundAction()
  private relocate(after: ILevelPickParts, before: ILevelPickParts): void {
    const { viewportService } = this.sources;
    const { hidden, picked, selected } = viewportService;
    const relocated: Nullable<TLevelPick> = picked ? relocateLevelPick(picked, before, after) : null;

    if (after.objects === before.objects) {
      if (relocated !== picked) {
        viewportService.notePickRelocated(relocated);
      }

      return;
    }

    viewportService.notePickRelocated(
      relocated,
      relocateLevelObjects(selected, before.objects, after.objects),
      relocateLevelObjects(hidden, before.objects, after.objects)
    );
  }
}
