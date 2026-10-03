import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, Observable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { visualsCommands } from "@/core/ipc/commands/visuals";
import { Session } from "@/core/ipc/session";
import { SelectedVisualDescription, SessionSnapshot, VisualSource } from "@/core/ipc/types/xrf-app";
import { RenderTextureReport } from "@/core/ipc/types/xrf-renderer";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { IRenderSurfaceDraw } from "@/core/render/lib/surface/render-surface-draw";
import { IVisualRenderSource } from "@/core/visuals/lib/render/visual-render-source";
import { IVisualBumpStatus, toLoadableBumps } from "@/core/visuals/lib/visual-bump";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { createVisualSurfaces } from "@/core/visuals/lib/visual-surface";
import { IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { toVisualBumpStatuses, toVisualTextureStatuses } from "@/core/visuals/lib/visual-texture-report";
import { createVisualViews, IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

/** A visual that is loaded: what it is, where it came from, and the views the scene draws. */
export interface IOpenVisual {
  selected: SessionSnapshot<SelectedVisualDescription>;
  views: IVisualModelViews;
}

/**
 * Opens a visual's session and publishes what the viewer reads of it; the renderer reads its geometry and textures
 * through the session itself, and says what became of each texture once it has.
 */
@Injectable()
export class VisualLoadService implements IVisualRenderSource {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(visualsCommands.closeModel);

  @Observable()
  public visual: AsyncState<IOpenVisual> = AsyncState.idle();

  /**
   * @returns The model on screen, or null while nothing is open. What a viewport draws, as opposed to what the
   *   open itself is doing.
   */
  @Computed()
  public get model(): Nullable<IVisualModelViews> {
    return this.visual.value?.views ?? null;
  }

  /** What became of each submesh's texture, so a panel can report it rather than leaving a submesh unexplained. */
  @Observable()
  public textureStatuses: ReadonlyMap<number, IVisualTextureStatus> = new Map();

  /** What became of each submesh's bump inputs, each half on its own. */
  @Observable()
  public bumpStatuses: ReadonlyMap<number, IVisualBumpStatus> = new Map();

  /**
   * @returns The open visual's session, which the renderer reads it through, or null while nothing is open.
   */
  @Computed()
  public get sessionId(): Nullable<string> {
    return this.visual.value?.selected.sessionId ?? null;
  }

  /**
   * @returns Whether any submesh binds a bump pair with both halves located; a dummy pair counts, since it is shaded.
   */
  @Computed()
  public get hasBump(): boolean {
    const selected: Nullable<SelectedVisualDescription> = this.visual.value?.selected.value ?? null;

    return Boolean(selected && toLoadableBumps(selected.dependencies.textures, selected.materials).length);
  }

  /**
   * @returns The path or entry the loaded visual was read from, or null when nothing is loaded.
   */
  @Computed()
  public get sourceLabel(): Nullable<string> {
    const source: Nullable<VisualSource> = this.visual.value?.selected.value.source ?? null;

    return source ? describeVisualSource(source) : null;
  }

  /**
   * @returns Whether the loaded visual animates from anything, referenced or embedded.
   */
  @Computed()
  public get hasMotions(): boolean {
    const selected: Nullable<SelectedVisualDescription> = this.visual.value?.selected.value ?? null;

    return Boolean(selected && (selected.dependencies.motions.length || selected.description.embeddedMotions.length));
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Load a visual and put it on screen.
   *
   * @param source - Visual source to open.
   * @param roots - Roots the source and its references are searched in.
   */
  @LatestFlow("visual")
  public *load(source: VisualSource, roots: XrayRoots): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Loading visual:", describeVisualSource(source));

    try {
      this.visual = this.visual.asLoading();

      const selected: SessionSnapshot<SelectedVisualDescription> = yield* call(
        this.session.open(visualsCommands.openModel, source, roots)
      );

      this.log.info("Visual described in:", formatDuration(timer.lap()));

      this.view(selected);

      this.log.info("Visual loaded:", describeVisualSource(source), "in", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to load visual:",
        describeVisualSource(source),
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.visual = this.visual.asFailed(transformed);
    }
  }

  /**
   * Restores the native selection unless a user action has taken the visual flow.
   */
  @ExclusiveFlow("visual")
  public *restore(): TFlow {
    const timer: Timer = new Timer();
    const snapshot = yield* call(visualsCommands.getModel());

    this.session.adopt(snapshot);

    if (snapshot) {
      this.log.info("Restoring visual:", describeVisualSource(snapshot.value.source));
      this.view(snapshot);

      this.log.info(
        "Visual restored:",
        describeVisualSource(snapshot.value.source),
        "in",
        formatDuration(timer.elapsed())
      );
    }
  }

  /**
   * Closes this loader's document without clearing a newer view.
   */
  @LatestFlow("visual")
  public *close(): TFlow {
    const selected = this.visual.value?.selected;

    yield* call(this.session.close());

    this.clearView();

    if (selected) {
      this.log.info("Visual closed:", describeVisualSource(selected.value.source));
    }
  }

  /**
   * Abandons pending loads and releases the current document and textures.
   */
  @BoundAction()
  public clear(): void {
    cancelFlow(this, "visual");
    this.session.release();

    this.clearView();
  }

  /**
   * Notes what the renderer made of each texture the open visual samples.
   *
   * @param sessionId - The session the renderer drew, so a report on a visual since replaced changes nothing.
   * @param reports - What it said of every texture.
   */
  @BoundAction()
  public noteTextures(sessionId: string, reports: Array<RenderTextureReport>): void {
    const selected: Nullable<SessionSnapshot<SelectedVisualDescription>> = this.visual.value?.selected ?? null;

    if (selected?.sessionId === sessionId) {
      this.textureStatuses = toVisualTextureStatuses(selected.value, reports);
      this.bumpStatuses = toVisualBumpStatuses(selected.value, reports);
    }
  }

  private clearView(): void {
    runInAction(() => {
      this.visual = this.visual.asIdle();
      this.textureStatuses = new Map();
      this.bumpStatuses = new Map();
    });
  }

  /**
   * Publish what the viewer reads of a described visual, every located texture loading until the renderer says.
   *
   * @param snapshot - Native visual description and its session identity.
   */
  private view(snapshot: SessionSnapshot<SelectedVisualDescription>): void {
    const selected: SelectedVisualDescription = snapshot.value;
    // Joined once, for the toolbar and the panels to read how each submesh is drawn.
    const surfaces: Map<number, IRenderSurfaceDraw> = createVisualSurfaces(
      selected.description.submeshes,
      selected.surfaces
    );

    this.visual = this.visual.asReady({ selected: snapshot, views: createVisualViews(selected.description, surfaces) });
    this.textureStatuses = toVisualTextureStatuses(selected, []);
    this.bumpStatuses = toVisualBumpStatuses(selected, []);
  }
}
