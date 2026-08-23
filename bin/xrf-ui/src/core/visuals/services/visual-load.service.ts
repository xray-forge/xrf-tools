import { Injectable } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { assetsRawCommands } from "@/core/bindings/commands/assets-raw";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { visualsRawCommands } from "@/core/bindings/commands/visuals-raw";
import { SelectedVisualDescription, VisualSource } from "@/core/bindings/types/xrf-app";
import { XrayWorldSpec } from "@/core/bindings/types/xrf-vfs";
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

/**
 * Turning a named visual into something a scene can draw.
 *
 * Two calls by design - the description is typed and the geometry is raw bytes, and a tauri command returns one or the
 * other, never both - followed by the textures, which arrive one at a time so a model shows its first without waiting
 * for its last. Everything is addressed by source and world rather than by what is currently loaded, so a response that
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
   * @param world - World the source and its references are searched in.
   */
  @BoundAction()
  public async load(source: VisualSource, world: XrayWorldSpec): Promise<void> {
    const timer: Timer = new Timer();

    this.log.info("Loading visual:", describeVisualSource(source));

    try {
      const request: number = runInAction(() => {
        this.requestId += 1;
        this.visual = this.visual.asLoading();

        return this.requestId;
      });

      const selected: SelectedVisualDescription = await visualsCommands.openModel(source, world);

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

    // The world the open used travels back with the description, so a geometry read after a reload searches what the
    // open searched rather than whatever the caller would name now.
    const buffer: ArrayBuffer = await visualsRawCommands.readGeometry(selected.source, selected.world);

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

    runInAction(() => {
      this.visual = this.visual.asReady({ selected, views });
      this.releaseTextures();
      this.textureStatuses = new Map(
        selected.dependencies.textures.map((texture) => [
          texture.submeshIndex,
          { reason: null, state: toInitialTextureState(texture.resolution), submeshIndex: texture.submeshIndex },
        ])
      );
    });

    void this.loadTextures(selected, request);
  }

  /**
   * Fetch each located texture and apply it as it lands.
   *
   * @param selected - Visual whose textures should be loaded.
   * @param request - Request identity used to discard stale textures.
   */
  private async loadTextures(selected: SelectedVisualDescription, request: number): Promise<void> {
    const loadable: Array<ILoadableTexture> = toLoadableTextures(selected.dependencies.textures);

    if (!loadable.length) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info(`Loading ${loadable.length} textures for:`, describeVisualSource(selected.source));

    await Promise.all(loadable.map((texture) => this.loadTexture(texture, selected.world, request)));

    this.log.info(`Loaded ${loadable.length} textures in:`, formatDuration(timer.elapsed()));
  }

  /**
   * One texture, from bytes to an uploaded texture or to a stated reason it is not one.
   *
   * Read by the logical path the open already resolved, so the bytes come from the file the description named — a
   * substituted dummy included — rather than from a second lookup that could answer differently.
   *
   * @param texture - Submesh identity and the logical path resolution located.
   * @param world - The mounted world the asset is read from.
   * @param request - Request identity used to discard a late response.
   */
  private async loadTexture(texture: ILoadableTexture, world: XrayWorldSpec, request: number): Promise<void> {
    try {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(world, texture.logicalPath);

      if (request !== this.requestId) {
        return;
      }

      const uploaded: Nullable<Texture> = createDdsTexture(bytes);

      runInAction(() => {
        if (uploaded) {
          this.textures = new Map(this.textures).set(texture.submeshIndex, uploaded);
        }

        this.setTextureStatus(texture.submeshIndex, {
          reason: null,
          state: uploaded ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
          submeshIndex: texture.submeshIndex,
        });
      });
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to load texture '${texture.logicalPath}':`, transformed);

      if (request !== this.requestId) {
        return;
      }

      runInAction(() => {
        this.setTextureStatus(texture.submeshIndex, {
          reason: transformed.message,
          state: EVisualTextureState.FAILED,
          submeshIndex: texture.submeshIndex,
        });
      });
    }
  }

  private setTextureStatus(submeshIndex: number, status: IVisualTextureStatus): void {
    this.textureStatuses = new Map(this.textureStatuses).set(submeshIndex, status);
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
