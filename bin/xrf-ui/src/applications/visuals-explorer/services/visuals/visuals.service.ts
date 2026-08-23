import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { createRoots } from "@/core/assets/lib";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { SelectedVisualDescription, VisualSource } from "@/core/bindings/types/xrf-app";
import { Vector3d } from "@/core/bindings/types/xrf-db";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { VisualBone } from "@/core/bindings/types/xrf-visual";
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
import { Nullable, Optional } from "@/lib/types/general";

/**
 * The visual the explorer has open, and everything about choosing it.
 *
 * Loading itself belongs to `VisualLoadService`, which the archives preview uses too. What is here is what only this
 * application decides: which roots a source is searched in, that a failure is worth a notification, and that leaving
 * drops the backend's selection.
 */
@Injectable()
export class VisualsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  /**
   * Bone the viewport marks, by name, or null when none is selected.
   */
  @Observable()
  public highlightedBone: Nullable<string> = null;

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
   * @returns Where the highlighted bone sits, or null when none is selected or the open model has no such bone.
   *
   * Resolved against the open model rather than remembered, which is what makes a selection left over from the
   * previous model harmless: the name simply matches nothing.
   */
  @Computed()
  public get highlightedJoint(): Nullable<[number, number, number]> {
    const bones: Array<VisualBone> = this.visual.value?.selected.description.bones ?? [];
    const bone: Optional<VisualBone> = bones.find((it: VisualBone) => it.name === this.highlightedBone);
    const position: Nullable<Vector3d> = bone?.bindPosition ?? null;

    if (position === null || position.x === null || position.y === null || position.z === null) {
      return null;
    }

    return [position.x, position.y, position.z];
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
  public async onProvision(/* todo: Respect strict mode and races here. */): Promise<void> {
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
    this.log.info("Deactivating and disposing visuals");

    this.loadService.clear();
    releaseEditorProject(visualsCommands.closeModel);
  }

  /**
   * Marks one bone in the viewport, or clears the mark.
   *
   * @param name - Bone name to mark, or null to clear it.
   */
  @BoundAction()
  public highlightBone(name: Nullable<string>): void {
    this.highlightedBone = name;
  }

  /**
   * Open a loose visual from disk.
   *
   * @param path - Filesystem path of the `.ogf` file.
   */
  @BoundAction()
  public async openFile(path: string): Promise<void> {
    // Centred on the file, so its own tree and installation are searched for its textures - and searched again when
    // those textures are read, because the roots travels with the description.
    await this.open({ kind: "file", path }, [], path);
  }

  /**
   * Open a visual of a browsed roots, loose or archived alike.
   *
   * The roots come from the caller because the browsed root is what makes the asset addressable at all: opening
   * `meshes\wpn\wpn_ak74.ogf` means nothing without the roots it names.
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
   * Load a visual in the roots this application composes, and report a failure the way this application reports one.
   *
   * @param source - Visual source to open.
   * @param roots - Roots searched ahead of the project's own.
   * @param asset - Asset the roots is centred on, whose own tree is searched first.
   */
  private async open(source: VisualSource, roots: Array<string> = [], asset: Nullable<string> = null): Promise<void> {
    await this.loadService.load(source, await this.getRoots(roots, asset));

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
   * The roots a visual's references are searched in, after the visual's own tree.
   *
   * Only the frontend knows which project is configured, which is why the roots is named on every call rather than
   * derived by the backend: it can derive the roots implied by an asset, but not an ambient one. Naming it rather than
   * holding a handle is also what lets a reload pick up where it left off, and another surface address the same assets.
   *
   * @param roots - Roots searched ahead of the project's own.
   * @param asset - Asset the roots is centred on.
   * @returns The roots spec to open with.
   */
  private async getRoots(roots: Array<string> = [], asset: Nullable<string> = null): Promise<XrayRoots> {
    const projectPath: Nullable<string> = this.projectService.xrfProjectPath;
    const project: Array<string> = projectPath ? [await getProjectGamedataPath(projectPath)] : [];

    // The caller's roots come first: a browsed tree is the nearer answer, and the project is the fallback behind it.
    return createRoots([...roots, ...project], asset);
  }
}
