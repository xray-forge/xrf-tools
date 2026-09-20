import { Texture } from "three";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ISectorTextureRequest } from "@/core/level/lib/sector/level-sector-textures";
import {
  createCheckerTexture,
  createDdsTexture,
  createDecodedTexture,
  IRenderTextureOptions,
  IRenderTextureUpload,
} from "@/core/render/lib/texture/render-texture";
import { Logger } from "@/lib/logging";
import { Maybe, Nullable } from "@/lib/types/general";

/** What became of one reference, so a surface it dresses can say why it is untextured or why it looks wrong. */
export interface ILevelTexture {
  texture: Nullable<Texture>;
  reason: Nullable<string>;
  /** Whether it was uploaded in a layout that keeps its alpha, which is not the file's answer but its callers'. */
  isAlphaRead: boolean;
}

/** One reference the set has something to say about, for a viewer reporting what a level is missing. */
export interface ILevelTextureProblem {
  reference: string;
  reason: string;
}

/**
 * Reading a level's uploaded textures, which is all a surface being dressed needs.
 */
export interface ILevelTextureLookup {
  readonly size: number;
  get(reference: string): Nullable<ILevelTexture>;
  /** Every reference the set could not answer for properly, in the order they were read. */
  listProblems(): ReadonlyArray<ILevelTextureProblem>;
}

/**
 * Owns a level's uploaded textures, keyed by the reference the shader table spells.
 *
 * Keyed by reference rather than by sector, because a level's surfaces are shared: one ground texture dresses dozens
 * of sectors, and uploading it once per sector would spend the memory the streaming budget is there to save.
 */
export class LevelTextureSet implements ILevelTextureLookup {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly loaded: Map<string, ILevelTexture> = new Map();
  private readonly pending: Map<string, Promise<ILevelTexture>> = new Map();

  /** Where each reference resolved to at open, so a load is a read rather than a second search. */
  private paths: ReadonlyMap<string, string> = new Map();

  private roots: Nullable<XrayRoots> = null;

  /**
   * Takes the level a later load reads against, releasing whatever the last one held.
   *
   * @param roots - Roots the level was opened in.
   * @param references - What each texture reference came to, from the open.
   */
  public open(roots: XrayRoots, references: ReadonlyArray<LevelTextureReference>): void {
    this.dispose();

    this.roots = roots;
    this.paths = new Map(
      references
        .filter((it: LevelTextureReference): it is LevelTextureReference & { logicalPath: string } =>
          Boolean(it.logicalPath)
        )
        .map((it) => [it.reference, it.logicalPath])
    );
  }

  /**
   * @param reference - Texture reference as a surface spells it.
   * @returns What became of it, or null while it has never been asked for.
   */
  public get(reference: string): Nullable<ILevelTexture> {
    return this.loaded.get(reference) ?? null;
  }

  public get size(): number {
    return this.loaded.size;
  }

  /**
   * @returns Every reference this set has something to say about, which is what a viewer reports.
   */
  public listProblems(): ReadonlyArray<ILevelTextureProblem> {
    return Array.from(this.loaded)
      .filter(([, loaded]) => loaded.reason)
      .map(([reference, loaded]) => ({ reason: loaded.reason as string, reference }));
  }

  /**
   * Loads every reference given, sharing whatever is already loaded or already being read.
   *
   * @param requests - What a sector's surfaces name, base textures and lightmaps alike.
   */
  public async load(requests: ReadonlyArray<ISectorTextureRequest>): Promise<void> {
    await Promise.all(requests.filter((it) => it.reference).map((request) => this.read(request)));
  }

  /**
   * Disposes every texture outside the given set.
   *
   * Called after any change to what is resident, with the references those sectors name. Idempotent, and self
   * healing: a texture orphaned by a cancelled read goes on the next call rather than lingering for the level's life.
   *
   * @param references - Everything the resident sectors still name.
   */
  public retain(references: ReadonlySet<string>): void {
    for (const [reference, loaded] of Array.from(this.loaded)) {
      if (!references.has(reference)) {
        loaded.texture?.dispose();
        this.loaded.delete(reference);
      }
    }
  }

  /** Releases every texture, for teardown and for swapping levels. */
  public dispose(): void {
    for (const loaded of this.loaded.values()) {
      loaded.texture?.dispose();
    }

    this.loaded.clear();
    this.pending.clear();
  }

  /**
   * Reads and uploads one reference, or joins the read already in flight for it.
   */
  private async read(request: ISectorTextureRequest): Promise<ILevelTexture> {
    const reference: string = request.reference;
    const held: Maybe<ILevelTexture> = this.loaded.get(reference);

    // Held unless it was uploaded without the alpha this caller needs. Whether a file keeps its alpha is decided by
    // the surfaces drawn with it, and a texture is uploaded once for the whole level by whichever sector asked first:
    // a sector of opaque surfaces uploading a cut-out file as `RGB_S3TC_DXT1` left every cut-out surface reached
    // later testing an alpha channel that is not there, which draws the file's transparent black as solid black.
    if (held && (!request.isAlphaRead || held.isAlphaRead)) {
      return held;
    }

    if (held) {
      this.log.info(`Texture '${reference}' is uploaded again, for a surface that reads its alpha`);

      held.texture?.dispose();
      this.loaded.delete(reference);
    }

    // Two sectors arriving together name the same ground texture, and reading it twice would upload it twice.
    const inFlight: Maybe<Promise<ILevelTexture>> = this.pending.get(reference);

    if (inFlight) {
      return inFlight;
    }

    const reading: Promise<ILevelTexture> = this.upload(request);

    this.pending.set(reference, reading);

    try {
      const loaded: ILevelTexture = await reading;

      this.loaded.set(reference, loaded);

      return loaded;
    } finally {
      this.pending.delete(reference);
    }
  }

  private async upload(request: ISectorTextureRequest): Promise<ILevelTexture> {
    const reference: string = request.reference;
    const logicalPath: Maybe<string> = this.paths.get(reference);

    const isAlphaRead: boolean = request.isAlphaRead;

    if (!this.roots || !logicalPath) {
      return faulty(isAlphaRead, `Nothing in the mounted roots answers to '${reference}'`);
    }

    try {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(this.roots, logicalPath);
      // Every file a level's shader table names is a picture - a base texture or a lightmap - so both are decoded from
      // sRGB. Only whether the alpha survives varies, and that is the surfaces' answer rather than the file's.
      const options: IRenderTextureOptions = { isAlphaRead: request.isAlphaRead, isColor: true };
      const upload: IRenderTextureUpload = createDdsTexture(bytes, options);

      if (upload.texture) {
        return { isAlphaRead, reason: null, texture: upload.texture };
      }

      // A layout the reader does not model; the backend expands those to png instead. The refusal is kept rather
      // than dropped, so a surface drawn from a decoded png can say which layout put it on that path.
      this.log.info(`Texture '${reference}' is decoded rather than uploaded:`, upload.refusal);

      return {
        isAlphaRead,
        reason: null,
        texture: await createDecodedTexture(await texturesRawCommands.readTexture(this.roots, logicalPath), options),
      };
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to load level texture '${reference}':`, transformed);

      return faulty(isAlphaRead, transformed.message);
    }
  }
}

/**
 * What a reference comes to when it cannot come to its own texture: the reason, and a checker to draw instead.
 *
 * @param isAlphaRead - What the upload was asked for, kept so a later caller can tell whether to ask again.
 * @param reason - Why there is no texture.
 * @returns The stand-in.
 */
function faulty(isAlphaRead: boolean, reason: string): ILevelTexture {
  return { isAlphaRead, reason, texture: createCheckerTexture() };
}
