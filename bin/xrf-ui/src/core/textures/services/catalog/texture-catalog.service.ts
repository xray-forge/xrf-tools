import { inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { createRoots, describeRoots } from "@/core/assets/lib";
import { transformError } from "@/core/error/lib";
import { texturesCommands } from "@/core/ipc/commands/textures";
import { Session } from "@/core/ipc/session";
import {
  SessionSnapshot,
  TextureCatalog,
  TextureCatalogMode,
  TextureMaterialSummary,
  TextureSource,
} from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { buildTextureNodes, ITextureNode } from "@/core/textures/lib/texture-catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * A browsed root set: every texture the engine would find in it, and what each descriptor makes of its own.
 *
 * The explorer's, and only the explorer's. A tool that works one texture at a time binds
 * [`TextureSelectionService`] alone and never lists anything, which is what keeps the two applications different tools
 * rather than one tool with a longer menu.
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
   *
   * A reload loses the tree but not the session. The selection is not restored with it, because the backend parks none:
   * every read is addressed by roots and reference, so re-choosing a row costs one describe.
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
    yield* this.list(createRoots([root, assetRoot]), "roots");
  }

  /**
   * List a plain directory of textures, addressed by path rather than by engine reference.
   *
   * @param directory - Filesystem path of the folder to list.
   */
  @LatestFlow("catalog")
  public *openLooseDirectory(directory: string): TFlow {
    yield* this.list(createRoots([directory]), "looseDirectory");
  }

  /**
   * End the session: stop browsing, and drop whatever texture was being inspected.
   *
   * One ending rather than two, because nothing here wants a texture on screen that the tree beside it no longer
   * contains, which is the disagreement this explorer exists to prevent.
   */
  @LatestFlow("catalog")
  public *close(): TFlow {
    try {
      yield* call(this.session.close());

      this.catalogState = this.catalogState.asIdle();
      this.summaries = this.summaries.asIdle([]);
      this.selectionService.clear();
    } catch (error) {
      this.log.error("Failed to close browsed texture roots:", error);
    }
  }

  /**
   * Inspect a texture of the browsed session.
   *
   * Addressed by the source the listing put on the row rather than by what it is labelled: a loose file has no
   * reference to resolve back into, and the row is the only thing that knows which of the two it is.
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
   *
   */
  @ExclusiveFlow("catalog")
  private *restore(): TFlow {
    try {
      const session = yield* call(texturesCommands.getSession());

      this.session.adopt(session);

      if (session) {
        yield* this.list(session.value.roots, session.value.mode);
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
   * The sweep runs inside this flow rather than beside it, so a superseding open cancels it along with the listing it
   * belongs to, and a sweep that fails leaves the tree browsable with no badges rather than closing it.
   *
   * @param roots - Roots to list and sweep.
   * @param mode - How to address what is found, which also decides whether a sweep can say anything.
   */
  private *list(roots: XrayRoots, mode: TextureCatalogMode): TFlow {
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

      this.log.info(`Listed ${catalog.value.entries.length} textures in:`, describeRoots(catalog.value.roots));

      // The sweep reads descriptors by engine reference, which a loose listing has none of. Skipped rather than run
      // and ignored, so a folder of one's own textures lists at once instead of waiting on a sweep with nothing to say.
      if (mode === "roots") {
        yield* this.sweep(catalog.value.roots);
      }
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to list textures:", transformed);

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
    this.summaries = this.summaries.asLoading();

    try {
      const summaries: Array<TextureMaterialSummary> = yield* call(texturesCommands.describeCatalog(roots));

      this.summaries = this.summaries.asReady(summaries);

      this.log.info(`Described ${summaries.length} texture descriptors`);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to describe texture descriptors:", transformed);

      this.summaries = this.summaries.asFailed(transformed, []);
    }
  }
}
