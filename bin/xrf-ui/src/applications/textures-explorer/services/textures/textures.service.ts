import { inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { buildTextureNodes, ITextureNode } from "@/applications/textures-explorer/lib/texture-catalog";
import { createRoots, describeRoots } from "@/core/assets/lib";
import { texturesCommands } from "@/core/bindings/commands/textures";
import { texturesRawCommands } from "@/core/bindings/commands/textures-raw";
import {
  TextureCatalog,
  TextureDescription,
  TextureMaterialSummary,
  TextureSource,
} from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { configuredAssetRoots } from "@/core/settings/lib/path/role";
import { PathsService } from "@/core/settings/services/paths/paths.service";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * The textures the explorer is browsing, and the one it is inspecting.
 */
@Injectable()
export class TexturesService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /**
   * Whether the backend has been asked what it still has open.
   */
  @Observable()
  public isReady: boolean = false;

  /** Every texture of the browsed roots, or null when a single texture was opened directly. */
  @Observable()
  public catalog: Loadable<Nullable<TextureCatalog>> = createLoadable(null);

  /** What each descriptor of those roots declares, empty until the sweep behind the listing finishes. */
  @Observable()
  public summaries: Loadable<Array<TextureMaterialSummary>> = createLoadable([]);

  /** The texture being inspected, or null when none has been chosen. */
  @Observable()
  public selected: Loadable<Nullable<TextureDescription>> = createLoadable(null);

  /** The selected texture decoded to png, or null when it is a descriptor with no texture to show. */
  @Observable()
  public preview: Loadable<Nullable<ArrayBuffer>> = createLoadable(null);

  /**
   * What the last inspection asked for, so a failed one can be asked for again.
   *
   * Not observable: what offers the retry is the failure already on screen, so nothing renders this.
   */
  private attempt: Nullable<TextureSource> = null;

  /**
   * @returns Whether a root set is open, which is what publishes the tree.
   */
  @Computed()
  public get isBrowsing(): boolean {
    return this.catalog.value !== null;
  }

  /**
   * @returns The roots every read of this session searches, as the backend reported them.
   *
   * Taken from whichever of the two sessions is open, so a single-texture session reads through the roots its own
   * description was resolved in rather than through none at all.
   */
  @Computed()
  public get roots(): Nullable<XrayRoots> {
    return this.catalog.value?.roots ?? this.selected.value?.roots ?? null;
  }

  /**
   * @returns One node per texture, with each declared bump pair folded under the texture declaring it.
   */
  @Computed()
  public get nodes(): Array<ITextureNode> {
    return buildTextureNodes(this.catalog.value?.entries ?? [], this.summaries.value ?? []);
  }

  /**
   * @returns The reference of the texture on screen, or null when none is.
   */
  @Computed()
  public get selectedReference(): Nullable<string> {
    return this.selected.value?.reference ?? null;
  }

  public constructor(private readonly pathsService: PathsService = inject(PathsService)) {}

  /**
   * Restore whatever roots the backend is still browsing.
   *
   * A reload loses the tree but not the session. The selection is not restored with it, because the backend parks none:
   * every read is addressed by roots and reference, so re-choosing a row costs one describe.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      const roots: Nullable<XrayRoots> = await texturesCommands.getRoots();

      if (roots) {
        this.log.info("Restoring browsed texture roots:", describeRoots(roots));

        // Through the lane rather than around it, so a root the user picks while this is still restoring wins.
        //
        // The listing announces readiness itself rather than this method announcing it afterwards. React's strict mode
        // provisions twice, and the second restore cancels the first, so a `finally` here would report a ready screen
        // while the catalog is still empty - which is the picker, opened for a quarter of a second over a session that
        // was already coming back.
        await flowResult(this.restore(roots));

        return;
      }
    } catch (error) {
      this.log.error("Failed to restore browsed texture roots:", error);
    }

    runInAction(() => {
      // Nothing to restore, or nothing that could be asked: the picker is the answer either way. A restore still in
      // flight owns the announcement, so this speaks only when nothing is arriving.
      if (!this.catalog.isLoading) {
        this.isReady = true;
      }
    });
  }

  /**
   * Drop the browsed roots on the way out of the application.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating and releasing the browsed roots");

    runInAction(() => {
      this.catalog = createLoadable(null);
      this.summaries = createLoadable([]);
      this.selected = createLoadable(null);
      this.preview = createLoadable(null);
    });

    releaseEditorProject(texturesCommands.close);
  }

  /**
   * Browse a root and list every texture the engine would find in it.
   *
   * @param root - Filesystem path of the directory or installation to browse.
   */
  @LatestFlow("catalog")
  public *openRoot(root: string): TFlow {
    yield* this.list(createRoots([root, ...configuredAssetRoots(this.pathsService.paths)]));
  }

  /**
   * End the session: stop browsing, and drop whatever texture was being inspected.
   *
   * One ending rather than two, because nothing here wants a texture on screen that the tree beside it no longer
   * contains, which is the disagreement this explorer exists to prevent.
   */
  @LatestFlow("catalog")
  public *close(): TFlow {
    this.catalog = createLoadable(null);
    this.summaries = createLoadable([]);
    this.selected = createLoadable(null);
    this.preview = createLoadable(null);
    this.attempt = null;

    try {
      yield* call(texturesCommands.close());
    } catch (error) {
      this.log.error("Failed to close browsed texture roots:", error);
    }
  }

  /**
   * Inspect a loose texture or descriptor from disk, without browsing anything.
   *
   * @param path - Filesystem path of the `.dds` or `.thm`.
   */
  @LatestFlow("selected")
  public *openFile(path: string): TFlow {
    // Centred on the file, so its own root and installation are searched for its descriptor and its pair, with the
    // configured roots behind them.
    yield* this.describe({ kind: "file", path }, createRoots(configuredAssetRoots(this.pathsService.paths), path));
  }

  /**
   * Inspect a texture of the browsed roots, loose or archived alike.
   *
   * @param reference - Engine reference of the texture, as the listing reported it.
   */
  @LatestFlow("selected")
  public *select(reference: string): TFlow {
    const roots: Nullable<XrayRoots> = this.roots;

    if (!roots) {
      this.log.info("Cannot inspect a texture with nothing open:", reference);

      return;
    }

    yield* this.describe({ kind: "asset", reference }, roots);
  }

  /**
   * Ask again for whatever the last inspection asked for, or do nothing when nothing has been asked for yet.
   */
  @LatestFlow("selected")
  public *retrySelected(): TFlow {
    const attempt: Nullable<TextureSource> = this.attempt;

    if (!attempt) {
      return;
    }

    if (attempt.kind === "file") {
      yield* this.describe(attempt, createRoots(configuredAssetRoots(this.pathsService.paths), attempt.path));

      return;
    }

    const roots: Nullable<XrayRoots> = this.roots;

    if (roots) {
      yield* this.describe(attempt, roots);
    }
  }

  /**
   * Puts an already browsed root set back on screen, for a session the backend still holds.
   *
   * @param roots - Roots the backend reported as browsed.
   */
  @LatestFlow("catalog")
  private *restore(roots: XrayRoots): TFlow {
    yield* this.list(roots);
  }

  /**
   * Lists a root set, then reads every descriptor it holds.
   *
   * The sweep runs inside this flow rather than beside it, so a superseding open cancels it along with the listing it
   * belongs to, and a sweep that fails leaves the tree browsable with no badges rather than closing it.
   *
   * @param roots - Roots to list and sweep.
   */
  private *list(roots: XrayRoots): TFlow {
    this.catalog = this.catalog.asLoading();
    this.summaries = createLoadable([]);

    try {
      const catalog: TextureCatalog = yield* call(texturesCommands.open(roots));

      this.catalog = this.catalog.asReady(catalog);
      // Announced here rather than after the sweep: the tree can be drawn from the listing alone, and waiting for
      // thousands of descriptors would hold a loader over a screen that is ready to browse. A cancelled flow never
      // reaches this line, which is what keeps a superseded restore from announcing a screen it no longer owns.
      this.isReady = true;

      this.log.info(`Listed ${catalog.entries.length} textures in:`, describeRoots(catalog.roots));

      yield* this.sweep(catalog.roots);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to list textures:", transformed);

      this.catalog = this.catalog.asFailed(transformed, null);
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

  /**
   * Resolves one texture and puts it on screen.
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

      yield* this.decode(description, roots);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to inspect texture:", transformed);

      this.selected = this.selected.asFailed(transformed, null);
      this.preview = createLoadable(null);
    }
  }

  /**
   * Decodes the selected texture into the png a webview can show.
   *
   * Inside the describe rather than beside it, so choosing another texture abandons this read with the description it
   * belonged to. A layout the backend cannot decode fails here alone and leaves the descriptor on screen, which is the
   * half of the answer that does not depend on the picture.
   *
   * @param description - The texture just resolved.
   * @param roots - Roots it was resolved in, so the read reaches the same file.
   */
  private *decode(description: TextureDescription, roots: XrayRoots): TFlow {
    const logicalPath: Nullable<string> = description.texture?.logicalPath ?? null;

    if (!logicalPath) {
      this.preview = createLoadable(null);

      return;
    }

    this.preview = this.preview.asLoading(null);

    try {
      const bytes: ArrayBuffer = yield* call(texturesRawCommands.readTexture(roots, logicalPath));

      this.preview = this.preview.asReady(bytes);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to decode texture:", transformed);

      this.preview = this.preview.asFailed(transformed, null);
    }
  }
}
