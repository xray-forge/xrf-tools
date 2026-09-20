import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, Observable, runInAction } from "@wirestate/mobx";

import { createRoots } from "@/core/assets/lib";
import { transformError } from "@/core/error/lib";
import { texturesCommands } from "@/core/ipc/commands/textures";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { ETextureSource, TextureDescription, TextureSource } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** A texture inspection and the search roots needed to repeat it. */
interface ITextureSelectionRequest {
  readonly source: TextureSource;
  readonly roots: XrayRoots;
}

/**
 * The one texture on screen: what it is, what its descriptor declares, and the picture of it.
 */
@Injectable()
export class TextureSelectionService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The texture being looked at, or null when none has been chosen. */
  @Observable()
  public selected: AsyncState<TextureDescription> = AsyncState.idle();

  /** The selected texture decoded to png, or null when it is a descriptor with no texture to show. */
  @Observable()
  public preview: AsyncState<ArrayBuffer> = AsyncState.idle();

  /**
   * The complete request, retained even when describing the texture fails.
   * Not observable: what offers the retry is the failure already on screen, so nothing renders this.
   */
  private attempt: Nullable<ITextureSelectionRequest> = null;

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

  /**
   * The extra tree used when opening a loose file.
   * Each inspection captures its roots so later retries keep the same search context.
   */
  private assetRoot: Nullable<string> = null;

  /**
   * Say which further tree a loose file is resolved against.
   * Applies to subsequent opens; an existing request retains its own roots for retries.
   *
   * @param path - The tree, or null to resolve a file in its own neighbourhood alone.
   */
  @BoundAction()
  public setAssetRoot(path: Nullable<string>): void {
    this.assetRoot = path;
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Abandons active loading and drops the selected texture, preview, and retry request.
   */
  public clear(): void {
    cancelFlow(this, "selected");

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
    yield* this.describe({ kind: ETextureSource.FILE, path }, this.toFileRoots(path));
  }

  /**
   * Open whatever a listing addressed, however it addressed it.
   *
   * @param source - What to inspect, as the listing reported it.
   * @param roots - Roots that listing came from, so both read the same world.
   */
  @LatestFlow("selected")
  public *open(source: TextureSource, roots: XrayRoots): TFlow {
    yield* this.describe(source, source.kind === ETextureSource.FILE ? this.toFileRoots(source.path) : roots);
  }

  /**
   * Repeats the last inspection with its original source and roots.
   * Does nothing before an inspection or after the selection is cleared.
   */
  @LatestFlow("selected")
  public *retry(): TFlow {
    const attempt: Nullable<ITextureSelectionRequest> = this.attempt;

    if (!attempt) {
      return;
    }

    yield* this.describe(attempt.source, attempt.roots);
  }

  /**
   * The roots a loose file is resolved in: its own tree first, the configured ones behind it.
   *
   * @param path - Filesystem path of the file.
   * @returns Roots centred on it.
   */
  private toFileRoots(path: string): XrayRoots {
    return createRoots([this.assetRoot], path);
  }

  /**
   * Resolve one texture and put it on screen.
   *
   * @param source - What to inspect, as the backend names it.
   * @param roots - Roots the source is resolved in.
   */
  private *describe(source: TextureSource, roots: XrayRoots): TFlow {
    this.attempt = { source, roots };
    this.selected = this.selected.asLoading();

    try {
      const description: TextureDescription = yield* call(texturesCommands.describe(source, roots));

      this.selected = this.selected.asReady(description);

      this.log.info("Inspecting texture:", description.reference);

      yield* this.decode(description);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to inspect texture:",
        source.kind === ETextureSource.FILE ? source.path : source.reference,
        transformed
      );

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

      this.log.error("Failed to decode texture:", description.reference, logicalPath, transformed);

      this.preview = this.preview.asFailed(transformed, null);
    }
  }
}
