import { inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";
import { AssetSource, EAssetSource } from "@xrf/ipc/types/xrf-app-core";
import { Vector3d } from "@xrf/ipc/types/xrf-math";
import { VisualBone } from "@xrf/ipc/types/xrf-visual";
import { Logger } from "@xrf/logging";
import { findLastSeparator } from "@xrf/path";
import { Nullable, Optional } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { IVisualBoneControls } from "@/core/visuals/components/panels/visual-inspection";
import { IVisualPose } from "@/core/visuals/lib/render";
import { selectAddonBones, selectHiddenBoneIndices } from "@/core/visuals/lib/visual-bones";
import { VisualInspectionService } from "@/core/visuals/services/visual-inspection";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualMotionService } from "@/core/visuals/services/visual-motion";

/**
 * What one open asked for, kept so the same request can be made again.
 */
interface IVisualOpenAttempt {
  source: AssetSource;
  roots: Array<Nullable<string>>;
  asset: Nullable<string>;
}

/**
 * The visual the explorer has open, and everything about choosing it.
 */
@Injectable()
export class VisualsService extends VisualInspectionService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

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
   * @returns How the model stands, which here is whichever motion the motions panel is playing.
   */
  @Computed()
  public get pose(): IVisualPose {
    return { frame: this.motionService.frame, motion: this.motionService.posed.value?.name ?? null };
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
   */
  @Computed()
  public get containingRoot(): Nullable<string> {
    const source: Nullable<AssetSource> = this.selected?.source ?? null;

    if (source?.kind !== EAssetSource.FILE) {
      return null;
    }

    const separatorAt: number = findLastSeparator(source.path);

    return separatorAt > 0 ? source.path.slice(0, separatorAt) : null;
  }

  /**
   * The open a retry repeats, or null before anything has been asked for.
   * Not observable: what offers the retry is the failure already on screen, so nothing renders this.
   */
  private attempt: Nullable<IVisualOpenAttempt> = null;

  public constructor(
    loadService: VisualLoadService = inject(VisualLoadService),
    private readonly motionService: VisualMotionService = inject(VisualMotionService)
  ) {
    super(loadService);
  }

  /**
   * Restore whatever the backend still has selected.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await this.loadService.restore();
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
    await this.open({ kind: EAssetSource.FILE, path }, [assetRoot], path);
  }

  /**
   * Open a visual of a browsed roots, loose or archived alike.
   *
   * @param logicalPath - Engine identity of the visual, as the listing reported it.
   * @param roots - Roots searched ahead of the project's own, usually the browsed one.
   */
  @BoundAction()
  public async openAsset(logicalPath: string, roots: Array<Nullable<string>>): Promise<void> {
    await this.open({ kind: EAssetSource.ASSET, logicalPath }, roots);
  }

  /**
   * Open again whatever the last attempt asked for, or do nothing when nothing has been asked for yet.
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

    await this.loadService.close();
  }

  /**
   * Load a visual in the roots this tool composes.
   *
   * @param source - Visual source to open.
   * @param roots - Roots searched ahead of the project's own.
   * @param asset - Asset the roots is centred on, whose own tree is searched first.
   */
  private async open(
    source: AssetSource,
    roots: Array<Nullable<string>> = [],
    asset: Nullable<string> = null
  ): Promise<void> {
    // Recorded before the read rather than after it, so a retry repeats the request even when the read never returns.
    this.attempt = { asset, roots, source };

    // A motion belongs to the skeleton it was baked against, and the backend has just parked a different selection.
    this.motionService.clear();

    // Whatever the caller named, in the order it named them, and nothing else: a browsed session already carries the
    // extra root it was opened with, so there is no ambient set to fall in behind them.
    await this.loadService.load(source, createRoots(roots, asset));
  }
}
