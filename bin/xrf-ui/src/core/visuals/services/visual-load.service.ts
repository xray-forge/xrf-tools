import { Injectable } from "@wirestate/core";
import { BoundAction, Computed, Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { assetsRawCommands } from "@/core/bindings/commands/assets-raw";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { visualsRawCommands } from "@/core/bindings/commands/visuals-raw";
import { SelectedVisualDescription, VisualSource } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { ILoadableBump, IVisualBumpStatus, IVisualBumpTextures, toLoadableBumps } from "@/core/visuals/lib/visual-bump";
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

/** One file's bytes as read, or the reason there are none. Decoding happens later, and only if still wanted. */
interface IVisualTextureRead {
  bytes: Nullable<ArrayBuffer>;
  reason: Nullable<string>;
}

/**
 * Every texture of one visual, ready to publish beside its geometry.
 *
 * Submeshes sharing a file share the one `Texture`: `textures` maps several indices onto the same upload, which is why
 * anything freeing them has to go through the distinct values rather than the entries.
 */
/** One half of a bump pair after upload: the texture when it made it, and what to report either way. */
interface IVisualBumpHalf {
  texture: Nullable<Texture>;
  state: EVisualTextureState;
  reason: Nullable<string>;
}

interface IVisualTextureLoad {
  textures: Map<number, Texture>;
  statuses: Map<number, IVisualTextureStatus>;
  /** The bump pair of every submesh whose material bound one and whose two files both uploaded. */
  bumps: Map<number, IVisualBumpTextures>;
  bumpStatuses: Map<number, IVisualBumpStatus>;
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
   * Uploaded bump pairs by submesh index, for a viewport to shade with.
   *
   * Only complete pairs: the engine's sampler reads both every texel, so one half is nothing to shade with. A `dummy`
   * outcome is a complete pair too, of the real dummy files, so the preview shows the flat surface the game shows.
   */
  @Observable()
  public bumps: ReadonlyMap<number, IVisualBumpTextures> = new Map();

  /** What became of each submesh's bump inputs, each half on its own. */
  @Observable()
  public bumpStatuses: ReadonlyMap<number, IVisualBumpStatus> = new Map();

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
      this.visual = this.visual.asLoading(null);
      this.releaseTextures();
      this.textureStatuses = new Map();
      this.bumpStatuses = new Map();

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
      this.bumpStatuses = new Map();
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

    const reads: Map<string, IVisualTextureRead> = yield* call(this.readTextures(selected));
    const loaded: IVisualTextureLoad = this.uploadTextures(selected, reads);

    this.log.info(`Loaded ${reads.size} texture files in:`, formatDuration(timer.lap()));

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
    this.bumps = loaded.bumps;
    this.bumpStatuses = loaded.bumpStatuses;
  }

  /**
   * Read every located texture file of a visual, in parallel, without decoding any of them.
   *
   * Once per **file** rather than once per submesh that names it: a model whose submeshes share a texture used to read
   * and upload it once each, which costs an archive read and a gpu upload for a file already in hand.
   *
   * A failure is a returned reason rather than a throw: one texture that cannot be read is a submesh drawn plain, not
   * a model that fails to open.
   *
   * @param selected - Visual whose textures should be read.
   * @returns Each distinct file's bytes, or the reason there are none, by logical path.
   */
  private async readTextures(selected: SelectedVisualDescription): Promise<Map<string, IVisualTextureRead>> {
    // Base textures and bump pairs in one pass: a dummy pair shared by every degraded submesh is one read either way.
    const paths: Array<string> = [
      ...new Set([
        ...toLoadableTextures(selected.dependencies.textures).map((it: ILoadableTexture) => it.logicalPath),
        ...toLoadableBumps(selected.dependencies.textures, selected.materials).flatMap((it: ILoadableBump) => [
          it.bump,
          it.companion,
        ]),
      ]),
    ];

    if (!paths.length) {
      return new Map();
    }

    this.log.info(`Reading ${paths.length} textures for:`, describeVisualSource(selected.source));

    const reads: Array<[string, IVisualTextureRead]> = await Promise.all(
      paths.map(async (logicalPath: string): Promise<[string, IVisualTextureRead]> => {
        try {
          // Read by the logical path the open already resolved, so the bytes come from the file the description named
          // - a substituted dummy included - rather than from a second lookup that could answer differently.
          const bytes: ArrayBuffer = await assetsRawCommands.readAsset(selected.roots, logicalPath);

          return [logicalPath, { bytes, reason: null }];
        } catch (error: unknown) {
          const transformed: Error = transformError(error);

          this.log.error(`Failed to load texture '${logicalPath}':`, transformed);

          return [logicalPath, { bytes: null, reason: transformed.message }];
        }
      })
    );

    return new Map(reads);
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
  private uploadTextures(
    selected: SelectedVisualDescription,
    reads: Map<string, IVisualTextureRead>
  ): IVisualTextureLoad {
    const statuses: Map<number, IVisualTextureStatus> = new Map(
      selected.dependencies.textures.map((texture) => [
        texture.submeshIndex,
        { reason: null, state: toInitialTextureState(texture.resolution), submeshIndex: texture.submeshIndex },
      ])
    );
    const textures: Map<number, Texture> = new Map();
    // One upload per file, shared by every submesh naming it.
    const uploads: Map<string, Nullable<Texture>> = new Map();

    for (const { submeshIndex, logicalPath } of toLoadableTextures(selected.dependencies.textures)) {
      const read: Optional<IVisualTextureRead> = reads.get(logicalPath);

      if (!read || read.bytes === null) {
        statuses.set(submeshIndex, { reason: read?.reason ?? null, state: EVisualTextureState.FAILED, submeshIndex });

        continue;
      }

      if (!uploads.has(logicalPath)) {
        uploads.set(logicalPath, createDdsTexture(read.bytes));
      }

      const uploaded: Nullable<Texture> = uploads.get(logicalPath) ?? null;

      if (uploaded) {
        textures.set(submeshIndex, uploaded);
      }

      statuses.set(submeshIndex, {
        reason: null,
        state: uploaded ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
        submeshIndex,
      });
    }

    const { bumps, bumpStatuses } = this.uploadBumps(selected, reads, uploads);

    return { statuses, textures, bumps, bumpStatuses };
  }

  /**
   * Upload every complete bump pair, sharing uploads with the base textures and between submeshes.
   *
   * A pair lands only when both halves uploaded: the engine samples both every texel, so half a pair shades nothing,
   * while each half still reports its own outcome so the panel can say which one failed. The renderer's own loader is
   * the only decoder here; a bump in a layout it refuses stays unshaded rather than going through the png fallback,
   * whose single image would lose the mip chain a bump relies on at distance.
   *
   * @param selected - Visual the materials belong to.
   * @param reads - What each texture read produced.
   * @param uploads - Uploads so far by logical path, shared so a file read once is uploaded once.
   * @returns Complete pairs by submesh index, and every submesh's two outcomes.
   */
  private uploadBumps(
    selected: SelectedVisualDescription,
    reads: Map<string, IVisualTextureRead>,
    uploads: Map<string, Nullable<Texture>>
  ): Pick<IVisualTextureLoad, "bumps" | "bumpStatuses"> {
    const bumps: Map<number, IVisualBumpTextures> = new Map();
    const bumpStatuses: Map<number, IVisualBumpStatus> = new Map();

    for (const loadable of toLoadableBumps(selected.dependencies.textures, selected.materials)) {
      const bump: IVisualBumpHalf = this.uploadBumpHalf(loadable.bump, reads, uploads);
      const companion: IVisualBumpHalf = this.uploadBumpHalf(loadable.companion, reads, uploads);

      if (bump.texture && companion.texture) {
        bumps.set(loadable.submeshIndex, { bump: bump.texture, companion: companion.texture });
      }

      bumpStatuses.set(loadable.submeshIndex, {
        submeshIndex: loadable.submeshIndex,
        bump: bump.state,
        companion: companion.state,
        reason: bump.reason ?? companion.reason,
      });
    }

    return { bumps, bumpStatuses };
  }

  /**
   * Upload one half of a pair, or say why it is not on the gpu.
   *
   * @param logicalPath - The located file.
   * @param reads - What each texture read produced.
   * @param uploads - Uploads so far by logical path, shared so a file read once is uploaded once.
   * @returns The texture when it uploaded, and the state and reason either way.
   */
  private uploadBumpHalf(
    logicalPath: string,
    reads: Map<string, IVisualTextureRead>,
    uploads: Map<string, Nullable<Texture>>
  ): IVisualBumpHalf {
    const read: Optional<IVisualTextureRead> = reads.get(logicalPath);

    if (!read || read.bytes === null) {
      return { texture: null, state: EVisualTextureState.FAILED, reason: read?.reason ?? null };
    }

    if (!uploads.has(logicalPath)) {
      uploads.set(logicalPath, createDdsTexture(read.bytes));
    }

    const texture: Nullable<Texture> = uploads.get(logicalPath) ?? null;

    return {
      texture,
      state: texture ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
      reason: null,
    };
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
            state: EVisualTextureState.DECODED,
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
   * The one owner: a scene borrows what it is handed and never frees it, because one upload can be drawn by several
   * submeshes and, once a level places several models, by several models.
   */
  private releaseTextures(): void {
    // Through the distinct values: submeshes sharing a file share one upload, and disposing per entry would free it
    // once per submesh that named it. The bump pairs join the same set, since a dummy pair is shared the same way.
    const uploaded: Set<Texture> = new Set(this.textures.values());

    for (const pair of this.bumps.values()) {
      uploaded.add(pair.bump);
      uploaded.add(pair.companion);
    }

    for (const texture of uploaded) {
      texture.dispose();
    }

    this.textures = new Map();
    this.bumps = new Map();
  }
}
