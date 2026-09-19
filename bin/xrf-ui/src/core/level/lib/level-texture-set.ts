import { Texture } from "three";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { createDdsTexture, createDecodedTexture } from "@/core/visuals/lib/visual-texture";
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
   * @param references - What a sector's surfaces name, base textures and lightmaps alike.
   */
  public async load(references: ReadonlyArray<string>): Promise<void> {
    const wanted: Set<string> = new Set(references.filter(Boolean));

    await Promise.all(Array.from(wanted, (reference: string) => this.read(reference)));
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
  private async read(reference: string): Promise<ILevelTexture> {
    const held: ILevelTexture | undefined = this.loaded.get(reference);

    if (held) {
      return held;
    }

    // Two sectors arriving together name the same ground texture, and reading it twice would upload it twice.
    const inFlight: Promise<ILevelTexture> | undefined = this.pending.get(reference);

    if (inFlight) {
      return inFlight;
    }

    const reading: Promise<ILevelTexture> = this.upload(reference);

    this.pending.set(reference, reading);

    try {
      const loaded: ILevelTexture = await reading;

      this.loaded.set(reference, loaded);

      return loaded;
    } finally {
      this.pending.delete(reference);
    }
  }

  private async upload(reference: string): Promise<ILevelTexture> {
    const logicalPath: Maybe<string> = this.paths.get(reference);

    if (!this.roots || !logicalPath) {
      return { reason: `Nothing in the mounted roots answers to '${reference}'`, texture: null };
    }

    try {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(this.roots, logicalPath);
      const compressed: Nullable<Texture> = createDdsTexture(bytes);

      // The renderer refuses some layouts the game ships; the backend decodes those to png instead.
      return {
        reason: null,
        texture:
          compressed ?? (await createDecodedTexture(await texturesRawCommands.readTexture(this.roots, logicalPath))),
      };
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to load level texture '${reference}':`, transformed);

      return { reason: transformed.message, texture: null };
    }
  }
}
