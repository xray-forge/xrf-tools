import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { Computed, Observable, runInAction } from "@wirestate/mobx";

import { createRoots } from "@/core/assets/lib";
import { texturesCommands } from "@/core/bindings/commands/textures";
import { texturesRawCommands } from "@/core/bindings/commands/textures-raw";
import { TextureDescription, TextureSource } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { configuredAssetRoots } from "@/core/settings/lib/path/role";
import { PathsService } from "@/core/settings/services/paths/paths.service";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * The one texture on screen: what it is, what its descriptor declares, and the picture of it.
 */
@Injectable()
export class TextureSelectionService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The texture being looked at, or null when none has been chosen. */
  @Observable()
  public selected: Loadable<Nullable<TextureDescription>> = Loadable.idle(null);

  /** The selected texture decoded to png, or null when it is a descriptor with no texture to show. */
  @Observable()
  public preview: Loadable<Nullable<ArrayBuffer>> = Loadable.idle(null);

  /**
   * What the last inspection asked for, so a failed one can be asked for again.
   *
   * Not observable: what offers the retry is the failure already on screen, so nothing renders this.
   */
  private attempt: Nullable<TextureSource> = null;

  /**
   * @returns The reference of the texture on screen, or null when none is.
   */
  @Computed()
  public get reference(): Nullable<string> {
    return this.selected.value?.reference ?? null;
  }

  /**
   * @returns The roots this texture was resolved in, so a later read searches what the describe searched.
   */
  @Computed()
  public get roots(): Nullable<XrayRoots> {
    return this.selected.value?.roots ?? null;
  }

  public constructor(private readonly pathsService: PathsService = inject(PathsService)) {}

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Drop whatever texture was on screen.
   */
  public clear(): void {
    runInAction(() => {
      this.selected = this.selected.asIdle();
      this.preview = this.preview.asIdle();
      this.attempt = null;
    });
  }

  /**
   * Open a loose texture or descriptor from disk.
   *
   * Centred on the file, so its own root and installation are searched for its descriptor and its pair, with the
   * configured roots behind them. That is what lets a bump name written here be resolved against real game data even
   * though the file itself sits outside one.
   *
   * @param path - Filesystem path of the `.dds` or `.thm`.
   */
  @LatestFlow("selected")
  public *openFile(path: string): TFlow {
    yield* this.describe({ kind: "file", path }, this.toFileRoots(path));
  }

  /**
   * Open a texture of an already browsed root set, loose or archived alike.
   *
   * @param reference - Engine reference of the texture, as a listing reported it.
   * @param roots - Roots that listing came from, so both read the same world.
   */
  @LatestFlow("selected")
  public *openReference(reference: string, roots: XrayRoots): TFlow {
    yield* this.describe({ kind: "asset", reference }, roots);
  }

  /**
   * Open whatever a listing addressed, however it addressed it.
   *
   * @param source - What to inspect, as the listing reported it.
   * @param roots - Roots that listing came from, so both read the same world.
   */
  @LatestFlow("selected")
  public *open(source: TextureSource, roots: XrayRoots): TFlow {
    yield* this.describe(source, source.kind === "file" ? this.toFileRoots(source.path) : roots);
  }

  /**
   * Ask again for whatever the last inspection asked for, or do nothing when nothing has been asked for yet.
   */
  @LatestFlow("selected")
  public *retry(): TFlow {
    const attempt: Nullable<TextureSource> = this.attempt;

    if (!attempt) {
      return;
    }

    const roots: Nullable<XrayRoots> = attempt.kind === "file" ? this.toFileRoots(attempt.path) : this.roots;

    if (roots) {
      yield* this.describe(attempt, roots);
    }
  }

  /**
   * The roots a loose file is resolved in: its own tree first, the configured ones behind it.
   *
   * @param path - Filesystem path of the file.
   * @returns Roots centred on it.
   */
  private toFileRoots(path: string): XrayRoots {
    return createRoots(configuredAssetRoots(this.pathsService.paths), path);
  }

  /**
   * Resolve one texture and put it on screen.
   *
   * @param source - What to inspect, as the backend names it.
   * @param roots - Roots the source is resolved in.
   */
  private *describe(source: TextureSource, roots: XrayRoots): TFlow {
    this.attempt = source;
    this.selected = this.selected.asLoading();

    try {
      const description: TextureDescription = yield* call(texturesCommands.describe(source, roots));

      this.selected = this.selected.asReady(description);

      this.log.info("Inspecting texture:", description.reference);

      yield* this.decode(description);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to inspect texture:", transformed);

      this.selected = this.selected.asFailed(transformed, null);
      this.preview = this.preview.asIdle();
    }
  }

  /**
   * Decode the selected texture into the png a webview can show.
   *
   * Inside the describe rather than beside it, so choosing another texture abandons this read with the description it
   * belonged to. A layout the backend cannot decode fails here alone and leaves the descriptor on screen, which is the
   * half of the answer that does not depend on the picture.
   *
   * @param description - The texture just resolved.
   */
  private *decode(description: TextureDescription): TFlow {
    const logicalPath: Nullable<string> = description.texture?.logicalPath ?? null;

    if (!logicalPath) {
      this.preview = this.preview.asIdle();

      return;
    }

    this.preview = this.preview.asLoading(null);

    try {
      const bytes: ArrayBuffer = yield* call(texturesRawCommands.readTexture(description.roots, logicalPath));

      this.preview = this.preview.asReady(bytes);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to decode texture:", transformed);

      this.preview = this.preview.asFailed(transformed, null);
    }
  }
}
