import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { visualsCommands } from "@/core/bindings/commands/visuals";
import { AssetWorldSpec, SelectedVisualDescription, VisualSource } from "@/core/bindings/types/xrf-app";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { getProjectGamedataPath } from "@/core/settings/lib/path/project";
import { ProjectService } from "@/core/settings/services/project/project.service";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * The visual the explorer has open, and everything about choosing it.
 *
 * Loading itself belongs to `VisualLoadService`, which the archives preview uses too. What is here is what only this
 * application decides: which world a source is searched in, that a failure is worth a notification, and that leaving
 * drops the backend's selection.
 */
@Injectable()
export class VisualsService {
  public readonly log: Logger = new Logger(this.constructor.name);

  @Observable()
  public isReady: boolean = false;

  /**
   * @returns The visual being shown, straight from the loader.
   *
   * Forwarded rather than mirrored: two copies of one state is how a screen ends up disagreeing with itself.
   */
  @Computed()
  public get visual(): Loadable<Nullable<IOpenVisual>> {
    return this.loadService.visual;
  }

  @Computed()
  public get textures(): ReadonlyMap<number, Texture> {
    return this.loadService.textures;
  }

  @Computed()
  public get textureStatuses(): ReadonlyMap<number, IVisualTextureStatus> {
    return this.loadService.textureStatuses;
  }

  @Computed()
  public get sourceLabel(): Nullable<string> {
    return this.loadService.sourceLabel;
  }

  @Computed()
  public get hasMotions(): boolean {
    return this.loadService.hasMotions;
  }

  /**
   * @returns The directory the open model sits in, or null when there is nothing to browse from.
   *
   * Only a loose file has one: an asset is already being browsed, and its bytes may sit inside a volume that no
   * directory contains.
   */
  @Computed()
  public get containingRoot(): Nullable<string> {
    const source: Nullable<VisualSource> = this.visual.value?.selected.source ?? null;

    if (source?.kind !== "file") {
      return null;
    }

    const separatorAt: number = Math.max(source.path.lastIndexOf("\\"), source.path.lastIndexOf("/"));

    return separatorAt > 0 ? source.path.slice(0, separatorAt) : null;
  }

  public constructor(
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly projectService: ProjectService = inject(ProjectService),
    private readonly loadService: VisualLoadService = inject(VisualLoadService)
  ) {
    makeObservable(this);
  }

  /**
   * Restore whatever the backend still has selected.
   *
   * A reload re-provisions this service, and the backend keeps the selection for exactly this reason, so the viewer
   * comes back showing the same model rather than an empty picker.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      const selected: Nullable<SelectedVisualDescription> = await visualsCommands.getModel();

      if (selected) {
        this.log.info("Restoring selected visual:", describeVisualSource(selected.source));

        await this.loadService.restore(selected);
      }
    } catch (error) {
      this.log.error("Failed to restore selected visual:", error);
    } finally {
      runInAction(() => {
        this.isReady = true;
      });
    }
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.loadService.clear();
    releaseEditorProject(visualsCommands.closeModel);
  }

  /**
   * Open a loose visual from disk.
   *
   * @param path - Filesystem path of the `.ogf` file.
   */
  @BoundAction()
  public async openFile(path: string): Promise<void> {
    // Centred on the file, so its own tree and installation are searched for its textures - and searched again when
    // those textures are read, because the world travels with the description.
    await this.open({ kind: "file", path }, [], path);
  }

  /**
   * Open a visual of a browsed world, loose or archived alike.
   *
   * The roots come from the caller because the browsed root is what makes the asset addressable at all: opening
   * `meshes\wpn\wpn_ak74.ogf` means nothing without the world it names.
   *
   * @param logicalPath - Engine identity of the visual, as the listing reported it.
   * @param roots - Roots searched ahead of the project's own, usually the browsed one.
   */
  @BoundAction()
  public async openAsset(logicalPath: string, roots: Array<string>): Promise<void> {
    await this.open({ kind: "asset", logicalPath }, roots);
  }

  /** Close what is open, on screen and in the backend. */
  @BoundAction()
  public async close(): Promise<void> {
    this.loadService.clear();

    try {
      await visualsCommands.closeModel();
    } catch (error) {
      this.log.error("Failed to close visual:", error);
    }
  }

  /**
   * Load a visual in the world this application composes, and report a failure the way this application reports one.
   *
   * @param source - Visual source to open.
   * @param roots - Roots searched ahead of the project's own.
   * @param asset - Asset the world is centred on, whose own tree is searched first.
   */
  private async open(source: VisualSource, roots: Array<string> = [], asset: Nullable<string> = null): Promise<void> {
    await this.loadService.load(source, await this.getWorld(roots, asset));

    const error: Nullable<Error> = this.visual.error;

    if (error) {
      emitNotification(this.eventBus, {
        details: `${describeVisualSource(source)}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.VISUALS_EXPLORER,
        title: "Could not open visual",
      });
    }
  }

  /**
   * The world a visual's references are searched in, after the visual's own tree.
   *
   * Only the frontend knows which project is configured, which is why the world is named on every call rather than
   * derived by the backend: it can derive the roots implied by an asset, but not an ambient one. Naming it rather than
   * holding a handle is also what lets a reload pick up where it left off, and another surface address the same assets.
   *
   * @param roots - Roots searched ahead of the project's own.
   * @param asset - Asset the world is centred on.
   * @returns The world spec to open with.
   */
  private async getWorld(roots: Array<string> = [], asset: Nullable<string> = null): Promise<AssetWorldSpec> {
    const projectPath: Nullable<string> = this.projectService.xrfProjectPath;
    const project: Array<string> = projectPath ? [await getProjectGamedataPath(projectPath)] : [];

    // The caller's roots come first: a browsed tree is the nearer answer, and the project is the fallback behind it.
    return { asset, roots: [...roots, ...project] };
  }
}
