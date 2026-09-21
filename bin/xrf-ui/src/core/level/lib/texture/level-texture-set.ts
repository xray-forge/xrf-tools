import { Texture } from "three";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ISectorTextureRequest } from "@/core/level/lib/sector/level-sector-textures";
import {
  ELevelSurfaceDressing,
  ILevelSurfaceDressing,
  ILevelTextureProblem,
  ILevelTextureReport,
} from "@/core/level/lib/surface/level-surface-dressing";
import {
  createCheckerTexture,
  createDdsTexture,
  createDecodedTexture,
  describeTextureUpload,
  IRenderTextureOptions,
  IRenderTextureUpload,
} from "@/core/render/lib/texture/render-texture";
import { Logger } from "@/lib/logging";
import { Maybe, Nullable } from "@/lib/types/general";

/**
 * What a reference comes to when it cannot come to its own texture: the reason, and a checker to draw instead.
 *
 * @param isAlphaRead - What the upload was asked for, kept so a later caller can tell whether to ask again.
 * @param reason - Why there is no texture.
 * @returns The stand-in.
 */
function faulty(isAlphaRead: boolean, reason: string): ILevelTexture {
  return { isAlphaRead, isMipped: true, reason, texture: createCheckerTexture(), upload: null };
}

/** What became of one reference, so a surface it dresses can say why it is untextured or why it looks wrong. */
export interface ILevelTexture {
  texture: Nullable<Texture>;
  reason: Nullable<string>;
  /** Whether it was uploaded in a layout that keeps its alpha, which is not the file's answer but its callers'. */
  isAlphaRead: boolean;
  /** Whether it was uploaded with the mip chain its callers sample, which a wall mark's texture is not. */
  isMipped: boolean;
  /** How it was uploaded, described here so nothing reporting on it has to hold the texture. */
  upload: Nullable<string>;
}

/**
 * Reading a level's uploaded textures, which is all a surface being dressed needs.
 */
export interface ILevelTextureLookup {
  readonly size: number;
  get(reference: string): Nullable<ILevelTexture>;
}

/** References whose upload changed, or `null` where the whole set went, which is a level opening or closing. */
export type TLevelTextureChange = Nullable<ReadonlySet<string>>;

/** Told what changed, so whatever draws from the set re-dresses what the change names and nothing else. */
export type TLevelTextureListener = (changed: TLevelTextureChange) => void;

/**
 * A lookup that says when it changes.
 */
export interface ILevelTextureSource extends ILevelTextureLookup {
  /**
   * @param listener - Told what changed, from inside the call that changed it.
   * @returns Stops the telling.
   */
  subscribe(listener: TLevelTextureListener): () => void;
  /**
   * @returns What the set came to, as data, for everything that reports on it and holds no texture of its own.
   */
  describe(): ILevelTextureReport;
}

/**
 * Owns a level's uploaded textures, keyed by the reference the shader table spells.
 *
 * Keyed by reference rather than by sector, because a level's surfaces are shared: one ground texture dresses dozens
 * of sectors, and uploading it once per sector would spend the memory the streaming budget is there to save.
 */
export class LevelTextureSet implements ILevelTextureSource {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly loaded: Map<string, ILevelTexture> = new Map();
  private readonly pending: Map<string, Promise<ILevelTexture>> = new Map();
  private readonly listeners: Set<TLevelTextureListener> = new Set();

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
    this.release();

    this.roots = roots;
    this.paths = new Map(
      references
        .filter((it: LevelTextureReference): it is LevelTextureReference & { logicalPath: string } =>
          Boolean(it.logicalPath)
        )
        .map((it) => [it.reference, it.logicalPath])
    );

    // Everything the last level held has gone, so everything drawn from the set is now dressed in nothing.
    this.notify(null);
  }

  /**
   * Takes a listener for what changes here.
   *
   * @param listener - Told what changed, from inside the call that changed it.
   * @returns Stops the telling.
   */
  public subscribe(listener: TLevelTextureListener): () => void {
    this.listeners.add(listener);

    return (): void => {
      this.listeners.delete(listener);
    };
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
   * @returns What this set came to: how much of it there is, what it could not answer for, and what became of
   *   each reference. Built rather than exposed, so a reader of it holds no texture.
   */
  public describe(): ILevelTextureReport {
    const dressing: Map<string, ILevelSurfaceDressing> = new Map();
    const problems: Array<ILevelTextureProblem> = [];

    for (const [reference, loaded] of this.loaded) {
      if (loaded.reason) {
        problems.push({ reason: loaded.reason, reference });
      }

      dressing.set(reference, toSurfaceDressing(reference, loaded));
    }

    return { dressing, problems, uploaded: this.loaded.size };
  }

  /**
   * Loads every reference given, sharing whatever is already loaded or already being read.
   *
   * @param requests - What a sector's surfaces name, base textures and lightmaps alike.
   */
  public async load(requests: ReadonlyArray<ISectorTextureRequest>): Promise<void> {
    const changed: Set<string> = new Set();

    await Promise.all(
      requests
        .filter((it) => it.reference)
        .map(async (request: ISectorTextureRequest): Promise<void> => {
          if (await this.read(request)) {
            changed.add(request.reference);
          }
        })
    );

    // Once for the sector rather than once per texture: a sector arriving is one change to whatever draws from here,
    // and the references it brought are what that change touched.
    if (changed.size) {
      this.notify(changed);
    }
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
    const released: Set<string> = new Set();

    for (const [reference, loaded] of Array.from(this.loaded)) {
      if (!references.has(reference)) {
        loaded.texture?.dispose();
        this.loaded.delete(reference);
        released.add(reference);
      }
    }

    // A surface still drawn by a material that outlived its texture has to stop sampling it, and a released
    // reference is the only thing that says which.
    if (released.size) {
      this.notify(released);
    }
  }

  /** Releases every texture, for teardown and for swapping levels. */
  public dispose(): void {
    this.release();
    this.notify(null);
  }

  /** Releases every texture without saying so, for the callers that are about to say something larger. */
  private release(): void {
    for (const loaded of this.loaded.values()) {
      loaded.texture?.dispose();
    }

    this.loaded.clear();
    this.pending.clear();
  }

  private notify(changed: TLevelTextureChange): void {
    for (const listener of Array.from(this.listeners)) {
      listener(changed);
    }
  }

  /**
   * Reads and uploads one reference, or joins the read already in flight for it.
   *
   * @returns Whether what the set holds for the reference is not what it held before the call, which is what makes
   *   the change a delta rather than a rumour.
   */
  private async read(request: ISectorTextureRequest): Promise<boolean> {
    const reference: string = request.reference;
    const held: Maybe<ILevelTexture> = this.loaded.get(reference);

    // Held unless it was uploaded without the alpha this caller needs. Whether a file keeps its alpha is decided by
    // the surfaces drawn with it, and a texture is uploaded once for the whole level by whichever sector asked first:
    // a sector of opaque surfaces uploading a cut-out file as `RGB_S3TC_DXT1` left every cut-out surface reached
    // later testing an alpha channel that is not there, which draws the file's transparent black as solid black.
    if (held && (!request.isAlphaRead || held.isAlphaRead) && (request.isMipped || !held.isMipped)) {
      return false;
    }

    if (held) {
      this.log.info(`Texture '${reference}' is uploaded again, for a surface that samples it differently`);

      held.texture?.dispose();
      this.loaded.delete(reference);
    }

    // Two sectors arriving together name the same ground texture, and reading it twice would upload it twice.
    const inFlight: Maybe<Promise<ILevelTexture>> = this.pending.get(reference);

    // A joiner reports the change too: it has no way to know whether whoever asked it has already seen the upload
    // the other read is making, and a second delta naming a reference that is already right costs one comparison.
    if (inFlight) {
      await inFlight;

      return true;
    }

    const reading: Promise<ILevelTexture> = this.upload(request);

    this.pending.set(reference, reading);

    try {
      this.loaded.set(reference, await reading);

      return true;
    } finally {
      this.pending.delete(reference);
    }
  }

  private async upload(request: ISectorTextureRequest): Promise<ILevelTexture> {
    const reference: string = request.reference;
    const logicalPath: Maybe<string> = this.paths.get(reference);

    const isAlphaRead: boolean = request.isAlphaRead;
    const isMipped: boolean = request.isMipped;

    if (!this.roots || !logicalPath) {
      return faulty(isAlphaRead, `Nothing in the mounted roots answers to '${reference}'`);
    }

    try {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(this.roots, logicalPath);
      // Every file a level's shader table names is a picture - a base texture or a lightmap - so both are decoded from
      // sRGB. Only whether the alpha survives varies, and that is the surfaces' answer rather than the file's.
      const options: IRenderTextureOptions = {
        isAlphaRead: request.isAlphaRead,
        isColor: true,
        isMipped: request.isMipped,
      };
      const upload: IRenderTextureUpload = createDdsTexture(bytes, options);

      if (upload.texture) {
        return {
          isAlphaRead,
          isMipped,
          reason: null,
          texture: upload.texture,
          upload: describeTextureUpload(upload.texture),
        };
      }

      // A layout the reader does not model; the backend expands those to png instead. The refusal is kept rather
      // than dropped, so a surface drawn from a decoded png can say which layout put it on that path.
      this.log.info(`Texture '${reference}' is decoded rather than uploaded:`, upload.refusal);

      const decoded: Texture = await createDecodedTexture(
        await texturesRawCommands.readTexture(this.roots, logicalPath),
        options
      );

      return { isAlphaRead, isMipped, reason: null, texture: decoded, upload: describeTextureUpload(decoded) };
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to load level texture '${reference}':`, transformed);

      return faulty(isAlphaRead, transformed.message);
    }
  }
}

/** What one reference came to, as a panel reads it. */
function toSurfaceDressing(reference: string, loaded: ILevelTexture): ILevelSurfaceDressing {
  // A stand-in carries its reason; one carrying neither a texture nor a reason is still a surface drawn from
  // nothing, and saying so is better than calling it uploaded.
  if (loaded.reason || !loaded.texture) {
    return {
      reason: loaded.reason ?? "Nothing was uploaded for it",
      reference,
      state: ELevelSurfaceDressing.STOOD_IN,
      upload: null,
    };
  }

  return { reason: null, reference, state: ELevelSurfaceDressing.UPLOADED, upload: loaded.upload };
}
