import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { createRoots } from "@/core/assets/lib";
import { transformError } from "@/core/error/lib";
import { SelectedVisualDescription, VisualSource } from "@/core/ipc/types/xrf-app";
import { Vector3d } from "@/core/ipc/types/xrf-db";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { VisualBone } from "@/core/ipc/types/xrf-visual";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { IVisualBoneControls, IVisualInspection } from "@/core/visuals/components/panels/visual-inspection";
import { selectAddonBones, selectHiddenBoneIndices } from "@/core/visuals/lib/visual-bones";
import { IVisualBumpStatus, IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { findLastSeparator } from "@/lib/path/separator";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * What one open asked for, kept so the same request can be made again.
 *
 * The arguments rather than the roots they resolved to: a retry re-derives them, so a root corrected after
 * the failure is picked up instead of being repeated as it was wrong.
 */
interface IVisualOpenAttempt {
  source: VisualSource;
  roots: Array<Nullable<string>>;
  asset: Nullable<string>;
}

/**
 * The visual the explorer has open, and everything about choosing it.
 *
 * Loading itself belongs to `VisualLoadService`, which the archives preview uses too. What is here is what only this
 * application decides: which roots a source is searched in, that a failure is worth a notification, and that leaving
 * drops the backend's selection.
 */
@Injectable()
export class VisualsService implements IVisualInspection {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  /**
   * Bone the viewport marks, by name, or null when none is selected.
   */
  @Observable()
  public highlightedBone: Nullable<string> = null;

  /**
   * Bones the viewport collapses, by name.
   */
  @Observable()
  public hiddenBones: ReadonlySet<string> = new Set();

  /**
   * @returns The visual being shown, straight from the loader.
   *
   * Forwarded rather than mirrored: two copies of one state is how a screen ends up disagreeing with itself.
   */
  @Computed()
  public get visual(): AsyncState<IOpenVisual> {
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
  public get bumps(): ReadonlyMap<number, IVisualBumpTextures> {
    return this.loadService.bumps;
  }

  @Computed()
  public get bumpStatuses(): ReadonlyMap<number, IVisualBumpStatus> {
    return this.loadService.bumpStatuses;
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
   * @returns What the backend reported about the open visual, or null when nothing is open.
   *
   * The one place the async state is unwrapped for its contents, so a panel asking what the model contains does not also
   * acquire an opinion about whether it is still arriving.
   */
  @Computed()
  public get selected(): Nullable<SelectedVisualDescription> {
    return this.visual.value?.selected.value ?? null;
  }

  /**
   * @returns The open model's skeleton, or no bones at all when nothing is open.
   */
  @Computed()
  public get bones(): Array<VisualBone> {
    return this.selected?.description.bones ?? [];
  }

  /**
   * @returns Where the highlighted bone sits, or null when none is selected or the open model has no such bone.
   *
   * Resolved against the open model rather than remembered, which is what makes a selection left over from the
   * previous model harmless: the name simply matches nothing.
   */
  @Computed()
  public get highlightedJoint(): Nullable<[number, number, number]> {
    const bone: Optional<VisualBone> = this.bones.find((it: VisualBone) => it.name === this.highlightedBone);
    const position: Nullable<Vector3d> = bone?.bindTransform?.c ?? null;

    if (position === null || position.x === null || position.y === null || position.z === null) {
      return null;
    }

    return [position.x, position.y, position.z];
  }

  /**
   * @returns Every bone the viewport should collapse, by index, descendants included.
   */
  @Computed()
  public get hiddenBoneIndices(): ReadonlySet<number> {
    return selectHiddenBoneIndices(this.bones, this.hiddenBones);
  }

  /**
   * @returns The addon bones this visual carries, which are the ones worth a control of their own.
   */
  @Computed()
  public get addonBones(): Array<string> {
    return selectAddonBones(this.bones);
  }

  /**
   * @returns Itself, because the explorer is a viewer: it marks and hides bones.
   *
   * The panels ask for this rather than for the service, so the same panels serve a surface that only inspects.
   */
  public get boneControls(): IVisualBoneControls {
    return this;
  }

  /**
   * @returns The directory the open model sits in, or null when there is nothing to browse from.
   *
   * Only a loose file has one: an asset is already being browsed, and its bytes may sit inside a volume that no
   * directory contains.
   */
  @Computed()
  public get containingRoot(): Nullable<string> {
    const source: Nullable<VisualSource> = this.selected?.source ?? null;

    if (source?.kind !== "file") {
      return null;
    }

    const separatorAt: number = findLastSeparator(source.path);

    return separatorAt > 0 ? source.path.slice(0, separatorAt) : null;
  }

  /**
   * The open a retry repeats, or null before anything has been asked for.
   *
   * Not observable: what offers the retry is the failure already on screen, so nothing renders this.
   */
  private attempt: Nullable<IVisualOpenAttempt> = null;

  public constructor(
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly loadService: VisualLoadService = inject(VisualLoadService),
    private readonly motionService: VisualMotionService = inject(VisualMotionService)
  ) {}

  /**
   * Restore whatever the backend still has selected.
   *
   * A reload re-provisions this service, and the backend keeps the selection for exactly this reason, so the viewer
   * comes back showing the same model rather than an empty picker.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      await flowResult(this.loadService.restore());
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
   * Collapses one bone in the viewport, or brings it back.
   *
   * @param name - Bone name to toggle.
   */
  @BoundAction()
  public toggleBoneVisibility(name: string): void {
    const hidden: Set<string> = new Set(this.hiddenBones);

    if (!hidden.delete(name)) {
      hidden.add(name);
    }

    this.hiddenBones = hidden;
  }

  /** Brings every collapsed bone back, which is what a model looks like before anything is turned off. */
  @BoundAction()
  public showAllBones(): void {
    this.hiddenBones = new Set();
  }

  /**
   * Open a loose visual from disk.
   *
   * @param path - Filesystem path of the `.ogf` file.
   * @param assetRoot - A further tree its references are searched in, or null to search only its own.
   */
  @BoundAction()
  public async openFile(path: string, assetRoot: Nullable<string> = null): Promise<void> {
    // Centred on the file, so its own tree is searched for its textures - and searched again when those textures are
    // read, because the roots travel with the description. The named root falls in behind it.
    await this.open({ kind: "file", path }, [assetRoot], path);
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
  public async openAsset(logicalPath: string, roots: Array<Nullable<string>>): Promise<void> {
    await this.open({ kind: "asset", logicalPath }, roots);
  }

  /**
   * Open again whatever the last attempt asked for, or do nothing when nothing has been asked for yet.
   *
   * The way out of a failed open. Re-activating the row works too - selection survives a failure by design - but that
   * is a navigation gesture standing in for a recovery action, and a single-model session has no row to activate.
   */
  @BoundAction()
  public async retryOpen(): Promise<void> {
    const attempt: Nullable<IVisualOpenAttempt> = this.attempt;

    if (attempt) {
      await this.open(attempt.source, attempt.roots, attempt.asset);
    }
  }

  /** Close what is open, on screen and in the backend. */
  @BoundAction()
  public async close(): Promise<void> {
    this.motionService.clear();

    try {
      await flowResult(this.loadService.close());
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
  private async open(
    source: VisualSource,
    roots: Array<Nullable<string>> = [],
    asset: Nullable<string> = null
  ): Promise<void> {
    // Recorded before the read rather than after it, so a retry repeats the request even when the read never returns.
    this.attempt = { asset, roots, source };

    // A motion belongs to the skeleton it was baked against, and the backend has just parked a different selection.
    this.motionService.clear();

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
  private async getRoots(roots: Array<Nullable<string>> = [], asset: Nullable<string> = null): Promise<XrayRoots> {
    // Whatever the caller named, in the order it named them, and nothing else: a browsed session already carries the
    // extra root it was opened with, so there is no ambient set to fall in behind them.
    return createRoots([...roots], asset);
  }
}
