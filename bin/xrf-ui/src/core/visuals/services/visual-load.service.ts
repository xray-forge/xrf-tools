import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, Observable, RefObservable, runInAction } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { visualsCommands } from "@/core/ipc/commands/visuals";
import { visualsRawCommands } from "@/core/ipc/commands/visuals-raw";
import { Session } from "@/core/ipc/session";
import { SelectedVisualDescription, SessionSnapshot, VisualSource } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { IRenderSurface } from "@/core/render/lib/surface/render-surface";
import { IVisualBumpFiles, IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { createVisualSurfaces } from "@/core/visuals/lib/visual-surface";
import { IVisualTextureFile, IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { VisualTextureSet } from "@/core/visuals/lib/visual-texture-set";
import { createVisualViews, IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** A visual that is loaded: what it is, where it came from, and the views the scene draws. */
export interface IOpenVisual {
  selected: SessionSnapshot<SelectedVisualDescription>;
  views: IVisualModelViews;
}

/**
 * Loads a native geometry snapshot and its resolved textures, publishing them together.
 *
 * Geometry uses the opening's identity; texture reads use the roots and paths resolved by that opening.
 * A delayed response can therefore be discarded without pairing geometry with another model's description.
 */
@Injectable()
export class VisualLoadService {
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

  /**
   * Texture files by submesh index, for whichever side draws to upload.
   */
  @RefObservable()
  public textures: ReadonlyMap<number, IVisualTextureFile> = new Map();

  /** What became of each submesh's texture, so a panel can report it rather than leaving a submesh unexplained. */
  @Observable()
  public textureStatuses: ReadonlyMap<number, IVisualTextureStatus> = new Map();

  /**
   * Bump pairs by submesh index, for whichever side draws to shade with.
   */
  @RefObservable()
  public bumps: ReadonlyMap<number, IVisualBumpFiles> = new Map();

  /** What became of each submesh's bump inputs, each half on its own. */
  @Observable()
  public bumpStatuses: ReadonlyMap<number, IVisualBumpStatus> = new Map();

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

      yield* this.view(selected);

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
      yield* this.view(snapshot);

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

  private clearView(): void {
    runInAction(() => {
      this.visual = this.visual.asIdle();
      this.releaseTextures();
      this.textureStatuses = new Map();
      this.bumpStatuses = new Map();
    });
  }

  /**
   * Fetch and view the geometry of a described visual, then its textures.
   *
   * @param snapshot - Native visual description and its session identity.
   */
  private *view(snapshot: SessionSnapshot<SelectedVisualDescription>): TFlow {
    const selected: SelectedVisualDescription = snapshot.value;
    const timer: Timer = new Timer();

    // Geometry belongs to this parse, even if the same path has since been opened with different roots.
    const buffer: ArrayBuffer = yield* call(visualsRawCommands.readGeometry(snapshot.sessionId));

    this.log.info("Visual geometry read in:", formatDuration(timer.lap()));

    // Joined once, and read by both the meshes that draw the surfaces and the uploads that have to carry their alpha.
    const surfaces: Map<number, IRenderSurface> = createVisualSurfaces(
      selected.description.submeshes,
      selected.surfaces
    );
    const views: IVisualModelViews = createVisualViews(selected.description, buffer, surfaces);

    this.log.info("Visual views built in:", formatDuration(timer.lap()));

    const loaded: VisualTextureSet = yield* VisualTextureSet.load(selected, surfaces);

    // Geometry, textures and their statuses land together, so the scene builds a mesh and dresses it in the same
    // commit. Published separately, a model showed untextured for as long as its textures took to arrive - brief,
    // and exactly long enough to read as grey plastic.
    this.releaseTextures();

    this.visual = this.visual.asReady({ selected: snapshot, views });
    this.textures = loaded.textures;
    this.textureStatuses = loaded.statuses;
    this.bumps = loaded.bumps;
    this.bumpStatuses = loaded.bumpStatuses;
  }

  /**
   * Drops what the current view was drawn from.
   */
  private releaseTextures(): void {
    this.textures = new Map();
    this.bumps = new Map();
  }
}
