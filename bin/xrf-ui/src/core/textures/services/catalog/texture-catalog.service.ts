import { inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { createRoots, describeRoots } from "@/core/assets/lib";
import { transformError } from "@/core/error/lib";
import { texturesCommands } from "@/core/ipc/commands/textures";
import { toEnumMember } from "@/core/ipc/enumeration";
import { Session } from "@/core/ipc/session";
import {
  ETextureCatalogMode,
  SessionSnapshot,
  TextureCatalog,
  TextureMaterialSummary,
  TextureSource,
} from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { buildTextureNodes, ITextureNode } from "@/core/textures/lib/texture-catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

/**
 * A browsed root set: every texture the engine would find in it, and what each descriptor makes of its own.
 */
@Injectable()
export class TextureCatalogService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(texturesCommands.close);

  /**
   * Whether the backend has been asked what it still has open.
   */
  @Observable()
  public isReady: boolean = false;

  /** Every texture of the browsed roots, or null when nothing is open. */
  @Observable()
  private catalogState: AsyncState<SessionSnapshot<TextureCatalog>> = AsyncState.idle();

  @Computed()
  public get catalog(): AsyncState<TextureCatalog> {
    return this.catalogState.map((snapshot) => snapshot.value);
  }

  /** What each descriptor of those roots declares, empty until the sweep behind the listing finishes. */
  @Observable()
  public summaries: AsyncState<Array<TextureMaterialSummary>> = AsyncState.idle([]);

  /**
   * @returns Whether a root set is open, which is what publishes the tree.
   */
  @Computed()
  public get isBrowsing(): boolean {
    return this.catalogState.value !== null;
  }

  /**
   * @returns The roots every read of this session searches, as the backend reported them.
   */
  @Computed()
  public get roots(): Nullable<XrayRoots> {
    return this.catalog.value?.roots ?? null;
  }

  /**
   * @returns One node per texture, with each declared bump pair folded under the texture declaring it.
   */
  @Computed()
  public get nodes(): Array<ITextureNode> {
    return buildTextureNodes(this.catalog.value?.entries ?? [], this.summaries.value ?? []);
  }

  public constructor(private readonly selectionService: TextureSelectionService = inject(TextureSelectionService)) {}

  /**
   * Restore whatever roots the backend is still browsing.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  /**
   * Drop the browsed roots on the way out of the application.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating, release");

    this.session.release();

    runInAction(() => {
      this.catalogState = this.catalogState.asIdle();
      this.summaries = this.summaries.asIdle([]);
    });
  }

  /**
   * Browse a root and list every texture the engine would find in it.
   *
   * @param root - Filesystem path of the directory or installation to browse.
   * @param assetRoot - A further tree listed and searched behind it, or null to list only the root itself.
   */
  @LatestFlow("catalog")
  public *openRoot(root: string, assetRoot: Nullable<string> = null): TFlow {
    // The named root joins the listing, not just the resolution: a mod tree carrying only what it changed folds with
    // the tree behind it, which is what makes one row per engine reference the right shape.
    yield* this.list(createRoots([root, assetRoot]), ETextureCatalogMode.ROOTS);
  }

  /**
   * List a plain directory of textures, addressed by path rather than by engine reference.
   *
   * @param directory - Filesystem path of the folder to list.
   */
  @LatestFlow("catalog")
  public *openLooseDirectory(directory: string): TFlow {
    yield* this.list(createRoots([directory]), ETextureCatalogMode.LOOSE_DIRECTORY);
  }

  /**
   * End the session: stop browsing, and drop whatever texture was being inspected.
   */
  @LatestFlow("catalog")
  public *close(): TFlow {
    const roots: Nullable<XrayRoots> = this.roots;

    try {
      yield* call(this.session.close());

      this.catalogState = this.catalogState.asIdle();
      this.summaries = this.summaries.asIdle([]);
      this.selectionService.clear();

      if (roots) {
        this.log.info("Texture catalog closed:", describeRoots(roots));
      }
    } catch (error) {
      this.log.error("Failed to close browsed texture roots:", error);
    }
  }

  /**
   * Inspect a texture of the browsed session.
   *
   * @param source - Where the row said its texture is.
   */
  public select(source: TextureSource): Promise<void> {
    const roots: Nullable<XrayRoots> = this.roots;

    if (!roots) {
      this.log.info("Cannot inspect a texture with nothing open:", source);

      return Promise.resolve();
    }

    return flowResult(this.selectionService.open(source, roots));
  }

  /**
   * Puts an already browsed session back on screen, for one the backend still holds.
   */
  @ExclusiveFlow("catalog")
  private *restore(): TFlow {
    try {
      const session = yield* call(texturesCommands.getSession());

      this.session.adopt(session);

      if (session) {
        this.log.info("Restoring texture catalog:", describeRoots(session.value.roots), session.value.mode);
        yield* this.list(session.value.roots, toEnumMember(ETextureCatalogMode, session.value.mode));
      }
    } catch (error) {
      this.log.error("Failed to restore browsed texture roots:", error);
    } finally {
      this.isReady = true;
    }
  }

  /**
   * Lists a root set, then reads every descriptor it holds.
   *
   * @param roots - Roots to list and sweep.
   * @param mode - How to address what is found, which also decides whether a sweep can say anything.
   */
  private *list(roots: XrayRoots, mode: ETextureCatalogMode): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Listing textures:", describeRoots(roots), mode);
    this.catalogState = this.catalogState.asLoading();
    this.summaries = this.summaries.asIdle([]);

    try {
      const catalog: SessionSnapshot<TextureCatalog> = yield* call(
        this.session.open(texturesCommands.open, roots, mode)
      );

      this.catalogState = this.catalogState.asReady(catalog);
      // Announced here rather than after the sweep: the tree can be drawn from the listing alone, and waiting for
      // thousands of descriptors would hold a loader over a screen that is ready to browse. A cancelled flow never
      // reaches this line, which is what keeps a superseded restore from announcing a screen it no longer owns.
      this.isReady = true;

      this.log.info(
        `Listed ${catalog.value.entries.length} textures:`,
        describeRoots(catalog.value.roots),
        "in",
        formatDuration(timer.elapsed())
      );

      // The sweep reads descriptors by engine reference, which a loose listing has none of. Skipped rather than run
      // and ignored, so a folder of one's own textures lists at once instead of waiting on a sweep with nothing to say.
      if (mode === ETextureCatalogMode.ROOTS) {
        yield* this.sweep(catalog.value.roots);
      }
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to list textures:",
        describeRoots(roots),
        mode,
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.catalogState = this.catalogState.asFailed(transformed);
      this.isReady = true;
    }
  }

  /**
   * Reads every descriptor of the listed roots, which is what badges the rows already on screen.
   *
   * @param roots - The roots the listing came from, so both read the same world.
   */
  private *sweep(roots: XrayRoots): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Reading texture descriptors:", describeRoots(roots));
    this.summaries = this.summaries.asLoading();

    try {
      const summaries: Array<TextureMaterialSummary> = yield* call(texturesCommands.describeCatalog(roots));

      this.summaries = this.summaries.asReady(summaries);

      this.log.info(
        `Described ${summaries.length} texture descriptors:`,
        describeRoots(roots),
        "in",
        formatDuration(timer.elapsed())
      );
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to describe texture descriptors:",
        describeRoots(roots),
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.summaries = this.summaries.asFailed(transformed, []);
    }
  }
}
