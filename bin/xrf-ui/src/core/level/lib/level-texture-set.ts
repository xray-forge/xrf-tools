import { Texture } from "three";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { createDdsTexture, createDecodedTexture } from "@/core/visuals/lib/visual-texture";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** What became of one reference, so a surface it dresses can say why it is untextured. */
export interface ILevelTexture {
  texture: Nullable<Texture>;
  reason: Nullable<string>;
}

/**
 * Owns a level's uploaded textures, keyed by the reference the shader table spells.
 */
export class LevelTextureSet {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly loaded: Map<string, ILevelTexture> = new Map();
  private readonly pending: Map<string, Promise<ILevelTexture>> = new Map();
  private readonly users: Map<string, Set<number>> = new Map();

  /** Where each reference resolved to at open, so a load is a read rather than a second search. */
  private paths: ReadonlyMap<string, string> = new Map();

  private roots: Nullable<XrayRoots> = null;

  /**
   * Takes the level a later load reads against.
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
   * @param reference - Texture reference as a section spells it.
   * @returns What became of it, or null while it has never been asked for.
   */
  public get(reference: string): Nullable<ILevelTexture> {
    return this.loaded.get(reference) ?? null;
  }

  public get size(): number {
    return this.loaded.size;
  }

  /**
   * Loads every reference one sector names, sharing whatever is already loaded.
   *
   * @param sector - Sector asking, which is what keeps the textures alive.
   * @param references - What its sections name, base textures and lightmaps alike.
   * @returns What became of each, once they have all settled.
   */
  public async acquire(sector: number, references: ReadonlyArray<string>): Promise<void> {
    const wanted: Set<string> = new Set(references.filter(Boolean));

    for (const reference of wanted) {
      this.users.set(reference, (this.users.get(reference) ?? new Set()).add(sector));
    }

    await Promise.all(Array.from(wanted, (reference: string) => this.load(reference)));
  }

  /**
   * Gives up one sector's claim, disposing whatever nothing else still names.
   *
   * @param sector - Sector going away.
   */
  public release(sector: number): void {
    for (const [reference, users] of this.users) {
      if (!users.delete(sector) || users.size) {
        continue;
      }

      this.users.delete(reference);
      this.loaded.get(reference)?.texture?.dispose();
      this.loaded.delete(reference);
    }
  }

  /** Releases every texture, for teardown and for swapping levels. */
  public dispose(): void {
    for (const loaded of this.loaded.values()) {
      loaded.texture?.dispose();
    }

    this.loaded.clear();
    this.pending.clear();
    this.users.clear();
  }

  /**
   * Reads and uploads one reference, or joins the read already in flight for it.
   */
  private async load(reference: string): Promise<ILevelTexture> {
    const held: ILevelTexture | undefined = this.loaded.get(reference);

    if (held) {
      return held;
    }

    // Two sectors arriving together name the same ground texture, and reading it twice would upload it twice.
    const inFlight: Promise<ILevelTexture> | undefined = this.pending.get(reference);

    if (inFlight) {
      return inFlight;
    }

    const reading: Promise<ILevelTexture> = this.read(reference);

    this.pending.set(reference, reading);

    try {
      const loaded: ILevelTexture = await reading;

      this.loaded.set(reference, loaded);

      return loaded;
    } finally {
      this.pending.delete(reference);
    }
  }

  private async read(reference: string): Promise<ILevelTexture> {
    const logicalPath: string | undefined = this.paths.get(reference);

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
