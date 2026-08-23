import { Injectable } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { assetsRawCommands } from "@/core/bindings/commands/assets-raw";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { visualsRawCommands } from "@/core/bindings/commands/visuals-raw";
import { SelectedVisualDescription, VisualSource } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import {
  createDdsTexture,
  EVisualTextureState,
  ILoadableTexture,
  IVisualTextureStatus,
  toInitialTextureState,
  toLoadableTextures,
} from "@/core/visuals/lib/visual-texture";
import { createVisualViews, IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { formatDuration } from "@/lib/format/duration";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger, Timer } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** A visual that is loaded: what it is, where it came from, and the views the scene draws. */
export interface IOpenVisual {
  selected: SelectedVisualDescription;
  views: IVisualModelViews;
}

/** One texture's outcome, held until the model it dresses is the one on screen. */
interface IVisualTextureResult {
  texture: Nullable<Texture>;
  status: IVisualTextureStatus;
}

/** Every texture of one visual, ready to publish beside its geometry. */
interface IVisualTextureLoad {
  textures: Map<number, Texture>;
  statuses: Map<number, IVisualTextureStatus>;
}

/**
 * Turning a named visual into something a scene can draw.
 *
 * Two calls by design - the description is typed and the geometry is raw bytes, and a tauri command returns one or the
 * other, never both - followed by the textures, which arrive one at a time so a model shows its first without waiting
 * for its last. Everything is addressed by source and roots rather than by what is currently loaded, so a response that
 * arrives after the caller moved on is discardable instead of being paired with the wrong model.
 */
@Injectable()
export class VisualLoadService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Distinguishes a response for the visual being asked about from one the caller already moved past. */
  private requestId: number = 0;

  @Observable()
  public visual: Loadable<Nullable<IOpenVisual>> = createLoadable(null);

  /**
   * Uploaded textures by submesh index, for a viewport to apply.
   *
   * Textures reach the scene through state rather than by handing this service a scene reference: the scene is owned by
   * the component that mounts it, and a store that reached into webgl would make two owners for one context.
   */
  @Observable()
  public textures: ReadonlyMap<number, Texture> = new Map();

  /** What became of each submesh's texture, so a panel can report it rather than leaving a submesh unexplained. */
  @Observable()
  public textureStatuses: ReadonlyMap<number, IVisualTextureStatus> = new Map();

  /**
   * @returns The path or entry the loaded visual was read from, or null when nothing is loaded.
   */
  @Computed()
  public get sourceLabel(): Nullable<string> {
    const source: Nullable<VisualSource> = this.visual.value?.selected.source ?? null;

    return source ? describeVisualSource(source) : null;
  }

  /**
   * @returns Whether the loaded visual animates from anything, referenced or embedded.
   */
  @Computed()
  public get hasMotions(): boolean {
    const selected: Nullable<SelectedVisualDescription> = this.visual.value?.selected ?? null;

    return Boolean(selected && (selected.dependencies.motions.length || selected.description.embeddedMotions.length));
  }

  public constructor() {
    makeObservable(this);
  }

  /**
   * Load a visual and put it on screen.
   *
   * Records a failure as state rather than throwing: a caller that wants to report it reads the error, and a caller
   * that does not is not obliged to catch.
   *
   * @param source - Visual source to open.
   * @param roots - Roots the source and its references are searched in.
   */
  @BoundAction()
  public async load(source: VisualSource, roots: XrayRoots): Promise<void> {
    const timer: Timer = new Timer();

    this.log.info("Loading visual:", describeVisualSource(source));

    try {
      const request: number = runInAction(() => {
        this.requestId += 1;
        this.visual = this.visual.asLoading();

        return this.requestId;
      });

      const selected: SelectedVisualDescription = await visualsCommands.openModel(source, roots);

      this.log.info("Visual described in:", formatDuration(timer.lap()));

      await this.view(selected, request);

      // Geometry only: the textures this started keep loading and report their own duration.
      this.log.info("Visual loaded in:", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Load error after:", formatDuration(timer.elapsed()), transformed);

      runInAction(() => {
        this.visual = this.visual.asFailed(transformed, null);
      });
    }
  }

  /**
   * Put an already described visual on screen, for a selection the backend still holds.
   *
   * @param selected - Typed description and source the backend reported.
   */
  @BoundAction()
  public async restore(selected: SelectedVisualDescription): Promise<void> {
    await this.view(selected, ++this.requestId);
  }

  /** Drop whatever is loaded, releasing the textures it uploaded. */
  @BoundAction()
  public clear(): void {
    runInAction(() => {
      this.requestId += 1;
      this.visual = createLoadable(null);
      this.releaseTextures();
      this.textureStatuses = new Map();
    });
  }

  /**
   * Fetch and view the geometry of a described visual, then its textures.
   *
   * @param selected - Typed description and source returned by the backend.
   * @param request - Request identity used to discard stale geometry.
   */
  private async view(selected: SelectedVisualDescription, request: number): Promise<void> {
    const timer: Timer = new Timer();

    // The roots the open used travels back with the description, so a geometry read after a reload searches what the
    // open searched rather than whatever the caller would name now.
    const buffer: ArrayBuffer = await visualsRawCommands.readGeometry(selected.source, selected.roots);

    if (request !== this.requestId) {
      this.log.info(
        "Discarding geometry for a visual already moved past after:",
        formatDuration(timer.elapsed()),
        describeVisualSource(selected.source)
      );

      return;
    }

    this.log.info("Visual geometry read in:", formatDuration(timer.lap()));

    const views: IVisualModelViews = createVisualViews(selected.description, buffer);

    this.log.info("Visual views built in:", formatDuration(timer.lap()));

    const loaded: IVisualTextureLoad = await this.loadTextures(selected, request);

    if (request !== this.requestId) {
      for (const texture of loaded.textures.values()) {
        texture.dispose();
      }

      return this.log.info("Discarding textures for a visual already moved past");
    }

    // Geometry, textures and their statuses land together, so the scene builds a mesh and dresses it in the same
    // commit. Published separately, a model showed untextured for as long as its textures took to arrive - brief, and
    // exactly long enough to read as grey plastic.
    runInAction(() => {
      this.releaseTextures();
      this.visual = this.visual.asReady({ selected, views });
      this.textures = loaded.textures;
      this.textureStatuses = loaded.statuses;
    });
  }

  /**
   * Fetch every located texture of a visual, without publishing any of them.
   *
   * Held back rather than applied as they land because they are addressed by submesh index, and the model those indices
   * belong to is not on screen yet: writing them to state early would dress the model being replaced.
   *
   * @param selected - Visual whose textures should be loaded.
   * @param request - Request identity used to discard stale textures.
   * @returns Uploaded textures by submesh index, and what became of every submesh's reference.
   */
  private async loadTextures(selected: SelectedVisualDescription, request: number): Promise<IVisualTextureLoad> {
    const statuses: Map<number, IVisualTextureStatus> = new Map(
      selected.dependencies.textures.map((texture) => [
        texture.submeshIndex,
        { reason: null, state: toInitialTextureState(texture.resolution), submeshIndex: texture.submeshIndex },
      ])
    );
    const loadable: Array<ILoadableTexture> = toLoadableTextures(selected.dependencies.textures);

    if (!loadable.length) {
      return { statuses, textures: new Map() };
    }

    const timer: Timer = new Timer();
    const textures: Map<number, Texture> = new Map();

    this.log.info(`Loading ${loadable.length} textures for:`, describeVisualSource(selected.source));

    await Promise.all(
      loadable.map(async (texture) => {
        const loaded: IVisualTextureResult = await this.loadTexture(texture, selected.roots, request);

        if (loaded.texture) {
          textures.set(texture.submeshIndex, loaded.texture);
        }

        statuses.set(texture.submeshIndex, loaded.status);
      })
    );

    this.log.info(`Loaded ${loadable.length} textures in:`, formatDuration(timer.elapsed()));

    return { statuses, textures };
  }

  /**
   * One texture, from bytes to an uploaded texture or to a stated reason it is not one.
   *
   * Read by the logical path the open already resolved, so the bytes come from the file the description named — a
   * substituted dummy included — rather than from a second lookup that could answer differently.
   *
   * A failure is a returned status rather than a throw: one texture that cannot be read is a submesh drawn plain, not
   * a model that fails to open.
   *
   * @param texture - Submesh identity and the logical path resolution located.
   * @param roots - The mounted roots the asset is read from.
   * @param request - Request identity used to discard a late response.
   * @returns The uploaded texture when there is one, and what became of the reference either way.
   */
  private async loadTexture(
    texture: ILoadableTexture,
    roots: XrayRoots,
    request: number
  ): Promise<IVisualTextureResult> {
    try {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(roots, texture.logicalPath);

      if (request !== this.requestId) {
        return { status: this.toPendingStatus(texture), texture: null };
      }

      const uploaded: Nullable<Texture> = createDdsTexture(bytes);

      return {
        status: {
          reason: null,
          state: uploaded ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
          submeshIndex: texture.submeshIndex,
        },
        texture: uploaded,
      };
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to load texture '${texture.logicalPath}':`, transformed);

      return {
        status: {
          reason: transformed.message,
          state: EVisualTextureState.FAILED,
          submeshIndex: texture.submeshIndex,
        },
        texture: null,
      };
    }
  }

  /** The status a texture keeps when its read is abandoned, which is the one it started with. */
  private toPendingStatus(texture: ILoadableTexture): IVisualTextureStatus {
    return { reason: null, state: EVisualTextureState.LOADING, submeshIndex: texture.submeshIndex };
  }

  /**
   * Free the uploaded textures of a visual being replaced.
   *
   * The scene disposes what it was handed when its model changes, and this disposes what the service still holds, so a
   * texture is freed by whichever side outlives the other.
   */
  private releaseTextures(): void {
    for (const texture of this.textures.values()) {
      texture.dispose();
    }

    this.textures = new Map();
  }
}
