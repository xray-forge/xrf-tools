import { Texture } from "three";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ISectorTextureRequest } from "@/core/level/lib/level-sector-textures";
import {
  createDdsTexture,
  createDecodedTexture,
  IRenderTextureOptions,
  IRenderTextureUpload,
} from "@/core/render/lib/render-texture";
import { Logger } from "@/lib/logging";
import { Maybe, Nullable } from "@/lib/types/general";

/** What became of one reference, so a surface it dresses can say why it is untextured. */
export interface ILevelTexture {
  texture: Nullable<Texture>;
  reason: Nullable<string>;
}

/**
 * Reading a level's uploaded textures, which is all a surface being dressed needs.
 */
export interface ILevelTextureLookup {
  readonly size: number;
  get(reference: string): Nullable<ILevelTexture>;
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

    if (held) {
      return held;
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

    if (!this.roots || !logicalPath) {
      return { reason: `Nothing in the mounted roots answers to '${reference}'`, texture: null };
    }

    try {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(this.roots, logicalPath);
      // Every file a level's shader table names is a picture - a base texture or a lightmap - so both are decoded from
      // sRGB. Only whether the alpha survives varies, and that is the surfaces' answer rather than the file's.
      const options: IRenderTextureOptions = { isAlphaRead: request.isAlphaRead, isColor: true };
      const upload: IRenderTextureUpload = createDdsTexture(bytes, options);

      if (upload.texture) {
        return { reason: null, texture: upload.texture };
      }

      // A layout the reader does not model; the backend expands those to png instead. The refusal is kept rather
      // than dropped, so a surface drawn from a decoded png can say which layout put it on that path.
      this.log.info(`Texture '${reference}' is decoded rather than uploaded:`, upload.refusal);

      return {
        reason: null,
        texture: await createDecodedTexture(await texturesRawCommands.readTexture(this.roots, logicalPath), options),
      };
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to load level texture '${reference}':`, transformed);

      return { reason: transformed.message, texture: null };
    }
  }
}
