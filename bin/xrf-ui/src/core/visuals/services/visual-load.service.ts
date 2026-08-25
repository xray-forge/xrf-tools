import { Injectable } from "@wirestate/core";
import { BoundAction, Computed, Observable, runInAction } from "@wirestate/mobx";
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
  createDecodedTexture,
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
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable, Optional } from "@/lib/types/general";

/** A visual that is loaded: what it is, where it came from, and the views the scene draws. */
export interface IOpenVisual {
  selected: SelectedVisualDescription;
  views: IVisualModelViews;
}

/** One texture's bytes as read, or the reason there are none. Decoding happens later, and only if still wanted. */
interface IVisualTextureRead {
  texture: ILoadableTexture;
  bytes: Nullable<ArrayBuffer>;
  reason: Nullable<string>;
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

  @Observable()
  public visual: Loadable<Nullable<IOpenVisual>> = createLoadable(null);

  /**
   * Uploaded textures by submesh index, for a viewport to apply.
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

  /**
   * Load a visual and put it on screen.
   *
   * Records a failure as state rather than throwing: a caller that wants to report it reads the error, and a caller
   * that does not is not obliged to catch.
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

      const selected: SelectedVisualDescription = yield* call(visualsCommands.openModel(source, roots));

      this.log.info("Visual described in:", formatDuration(timer.lap()));

      yield* this.view(selected);

      this.log.info("Visual loaded in:", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Load error after:", formatDuration(timer.elapsed()), transformed);

      this.visual = this.visual.asFailed(transformed, null);
    }
  }

  /**
   * Put an already described visual on screen, for a selection the backend still holds.
   *
   * @param selected - Typed description and source the backend reported.
   */
  @LatestFlow("visual")
  public *restore(selected: SelectedVisualDescription): TFlow {
    yield* this.view(selected);
  }

  /** Drop whatever is loaded, releasing the textures it uploaded. */
  @BoundAction()
  public clear(): void {
    cancelFlow(this, "visual");

    runInAction(() => {
      this.visual = createLoadable(null);
      this.releaseTextures();
      this.textureStatuses = new Map();
    });
  }

  /**
   * Fetch and view the geometry of a described visual, then its textures.
   *
   * @param selected - Typed description and source returned by the backend.
   */
  private *view(selected: SelectedVisualDescription): TFlow {
    const timer: Timer = new Timer();

    // The roots the open used travels back with the description, so a geometry read after a reload searches what the
    // open searched rather than whatever the caller would name now.
    const buffer: ArrayBuffer = yield* call(visualsRawCommands.readGeometry(selected.source, selected.roots));

    this.log.info("Visual geometry read in:", formatDuration(timer.lap()));

    const views: IVisualModelViews = createVisualViews(selected.description, buffer);

    this.log.info("Visual views built in:", formatDuration(timer.lap()));

    const reads: Array<IVisualTextureRead> = yield* call(this.readTextures(selected));
    const loaded: IVisualTextureLoad = this.uploadTextures(selected, reads);

    this.log.info(`Loaded ${reads.length} textures in:`, formatDuration(timer.lap()));

    // A second pass, and only for what the renderer's loader declined: those files come back decoded by the backend.
    // Kept apart from the pass above so the common path stays one read and one synchronous upload.
    const declined: Array<number> = [...loaded.statuses.values()]
      .filter((status) => status.state === EVisualTextureState.UNSUPPORTED_FORMAT)
      .map((status) => status.submeshIndex);

    if (declined.length) {
      yield* call(this.decodeTextures(selected, declined, loaded));

      this.log.info(`Decoded ${declined.length} textures in:`, formatDuration(timer.lap()));
    }

    // Geometry, textures and their statuses land together, so the scene builds a mesh and dresses it in the same
    // commit. Published separately, a model showed untextured for as long as its textures took to arrive - brief, and
    // exactly long enough to read as grey plastic.
    this.releaseTextures();

    this.visual = this.visual.asReady({ selected, views });
    this.textures = loaded.textures;
    this.textureStatuses = loaded.statuses;
  }

  /**
   * Read every located texture of a visual, in parallel, without decoding any of them.
   *
   * A failure is a returned reason rather than a throw: one texture that cannot be read is a submesh drawn plain, not
   * a model that fails to open.
   *
   * @param selected - Visual whose textures should be read.
   * @returns Each texture's bytes, or the reason there are none.
   */
  private async readTextures(selected: SelectedVisualDescription): Promise<Array<IVisualTextureRead>> {
    const loadable: Array<ILoadableTexture> = toLoadableTextures(selected.dependencies.textures);

    if (!loadable.length) {
      return [];
    }

    this.log.info(`Reading ${loadable.length} textures for:`, describeVisualSource(selected.source));

    return await Promise.all(
      loadable.map(async (texture) => {
        try {
          // Read by the logical path the open already resolved, so the bytes come from the file the description named
          // - a substituted dummy included - rather than from a second lookup that could answer differently.
          const bytes: ArrayBuffer = await assetsRawCommands.readAsset(selected.roots, texture.logicalPath);

          return { texture, bytes, reason: null };
        } catch (error: unknown) {
          const transformed: Error = transformError(error);

          this.log.error(`Failed to load texture '${texture.logicalPath}':`, transformed);

          return { texture, bytes: null, reason: transformed.message };
        }
      })
    );
  }

  /**
   * Decode and upload what was read, and say what became of every submesh's reference.
   *
   * Synchronous, and deliberately so: this is the expensive half, and keeping it out of the awaiting part is what lets
   * a cancelled load skip it entirely.
   *
   * @param selected - Visual the textures belong to.
   * @param reads - What each texture read produced.
   * @returns Uploaded textures by submesh index, and every submesh's outcome.
   */
  private uploadTextures(selected: SelectedVisualDescription, reads: Array<IVisualTextureRead>): IVisualTextureLoad {
    const statuses: Map<number, IVisualTextureStatus> = new Map(
      selected.dependencies.textures.map((texture) => [
        texture.submeshIndex,
        { reason: null, state: toInitialTextureState(texture.resolution), submeshIndex: texture.submeshIndex },
      ])
    );
    const textures: Map<number, Texture> = new Map();

    for (const read of reads) {
      const submeshIndex: number = read.texture.submeshIndex;

      if (read.bytes === null) {
        statuses.set(submeshIndex, { reason: read.reason, state: EVisualTextureState.FAILED, submeshIndex });

        continue;
      }

      const uploaded: Nullable<Texture> = createDdsTexture(read.bytes);

      if (uploaded) {
        textures.set(submeshIndex, uploaded);
      }

      statuses.set(submeshIndex, {
        reason: null,
        state: uploaded ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
        submeshIndex,
      });
    }

    return { statuses, textures };
  }

  /**
   * Ask the backend to decode the textures three.js declined, and fold them into what will be published.
   *
   * Reached for `BC7`, RGBA-ordered `A8B8G8R8` and `BC5`, which `DDSLoader` has no branch for - 97 files of the 26,145
   * measured across the reference trees. Eight bit luminance and `R5G6B5` decode nowhere, so those keep the status they
   * already have and the materials panel goes on saying the format is unsupported.
   *
   * @param selected - Visual the textures belong to, whose roots address the read.
   * @param declined - Submesh indices whose texture the renderer's own loader refused.
   * @param loaded - Textures and statuses to fold the decoded ones into.
   */
  private async decodeTextures(
    selected: SelectedVisualDescription,
    declined: Array<number>,
    loaded: IVisualTextureLoad
  ): Promise<void> {
    const references: Map<number, string> = new Map(
      toLoadableTextures(selected.dependencies.textures).map((texture) => [texture.submeshIndex, texture.logicalPath])
    );

    await Promise.all(
      declined.map(async (submeshIndex) => {
        const logicalPath: Optional<string> = references.get(submeshIndex);

        if (!logicalPath) {
          return;
        }

        try {
          const png: ArrayBuffer = await visualsRawCommands.readTexture(selected.roots, logicalPath);

          loaded.textures.set(submeshIndex, await createDecodedTexture(png));
          loaded.statuses.set(submeshIndex, {
            reason: null,
            state: EVisualTextureState.APPLIED,
            submeshIndex,
          });
        } catch (error: unknown) {
          // Left as unsupported rather than failed: nothing broke, the format simply decodes nowhere.
          this.log.info(`Texture '${logicalPath}' decodes nowhere:`, transformError(error).message);
        }
      })
    );
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
