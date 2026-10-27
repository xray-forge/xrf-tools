import { inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult } from "@wirestate/mobx";
import { TFlow } from "@xrf/flow";
import { toEnumMember } from "@xrf/ipc";
import { texturesCommands } from "@xrf/ipc/commands/textures";
import { SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import {
  ETextureCatalogMode,
  TextureBrowse,
  TextureCatalog,
  TextureMaterialSummary,
  TextureSource,
} from "@xrf/ipc/types/xrf-app-textures";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { createRoots, describeRoots } from "@/core/assets/lib";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { ISessionRootOpening, SessionRootResource } from "@/core/session/lib/session-root-resource";
import { buildTextureNodes, ITextureNode } from "@/core/textures/lib/texture-catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";

/**
 * A browsed root set: every texture the engine would find in it, and what each descriptor makes of its own.
 */
@Injectable()
export class TextureCatalogService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Every texture of the browsed roots, or none when nothing is open. */
  public readonly catalog: SessionRootResource<TextureCatalog, TextureBrowse> = new SessionRootResource(this.log, {
    subject: "texture catalog",
    close: texturesCommands.closeBrowse,
    // A restore answers what was browsed, not its listing, so the roots are listed again.
    reopen: {
      restore: texturesCommands.restoreBrowse,
      open: (browse: TextureBrowse) =>
        TextureCatalogService.toOpening(browse.roots, toEnumMember(ETextureCatalogMode, browse.mode)),
    },
    adopted: (snapshot: SessionSnapshot<TextureCatalog>) => this.sweep(snapshot.value),
  });

  /** What each descriptor of those roots declares, empty until the sweep behind the listing finishes. */
  public readonly summaries: CommandOperation<Array<TextureMaterialSummary>> = new CommandOperation(this.log);

  public constructor(private readonly selectionService: TextureSelectionService = inject(TextureSelectionService)) {}

  /**
   * @returns Whether a root set is open, which is what publishes the tree.
   */
  @Computed()
  public get isBrowsing(): boolean {
    return this.catalog.value !== null;
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

  /**
   * Restore whatever roots the backend is still browsing.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.catalog.restore());
  }

  /**
   * Browse a root and list every texture the engine would find in it.
   *
   * @param root - Filesystem path of the directory or installation to browse.
   * @param assetRoot - A further tree listed and searched behind it, or null to list only the root itself.
   */
  @BoundAction()
  public async openRoot(root: string, assetRoot: Nullable<string> = null): Promise<void> {
    // The named root joins the listing, not just the resolution: a mod tree carrying only what it changed folds with
    // the tree behind it, which is what makes one row per engine reference the right shape.
    await this.open(TextureCatalogService.toOpening(createRoots([root, assetRoot]), ETextureCatalogMode.ROOTS));
  }

  /**
   * List a plain directory of textures, addressed by path rather than by engine reference.
   *
   * @param directory - Filesystem path of the folder to list.
   */
  @BoundAction()
  public async openLooseDirectory(directory: string): Promise<void> {
    await this.open(TextureCatalogService.toOpening(createRoots([directory]), ETextureCatalogMode.LOOSE_DIRECTORY));
  }

  /**
   * End the session: stop browsing, and drop whatever texture was being inspected.
   */
  @BoundAction()
  public async close(): Promise<void> {
    if (await flowResult(this.catalog.close())) {
      this.summaries.reset();
      this.selectionService.clear();
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
   * @param roots - Roots to list.
   * @param mode - How to address what is found, which also decides whether a sweep can say anything.
   * @returns The open listing them.
   */
  private static toOpening(roots: XrayRoots, mode: ETextureCatalogMode): ISessionRootOpening<TextureCatalog> {
    return {
      label: describeRoots(roots),
      command: (sessionId: SessionId) => texturesCommands.openBrowse(sessionId, roots, mode),
    };
  }

  private async open(opening: ISessionRootOpening<TextureCatalog>): Promise<void> {
    await flowResult(this.catalog.open(opening.label, opening.command));
  }

  /**
   * Reads every descriptor of a listing, which is what badges the rows already on screen. The tree is drawn from the
   * listing alone, so the sweep runs after it is shown rather than holding a loader over it.
   *
   * @param catalog - The listing, so both read the same world.
   */
  private *sweep(catalog: TextureCatalog): TFlow {
    this.summaries.reset();

    // The sweep reads descriptors by engine reference, which a loose listing has none of. Skipped rather than run and
    // ignored, so a folder of one's own textures lists at once instead of waiting on a sweep with nothing to say.
    if (toEnumMember(ETextureCatalogMode, catalog.mode) !== ETextureCatalogMode.ROOTS) {
      return;
    }

    yield* this.summaries.run({
      name: "Read texture descriptors",
      subject: describeRoots(catalog.roots),
      command: () => texturesCommands.describeCatalog(catalog.roots),
      failure: "Could not read the texture descriptors",
    });
  }
}
