import { Maybe, Nullable } from "@xrf/types";

import { IRendererMotion } from "#/contract/scene/renderer-motion";
import { IRendererPose } from "#/contract/scene/renderer-pose";
import { IRendererSkeleton } from "#/contract/scene/renderer-skeleton";
import { BIND_POSE, RendererSkeletonEntry } from "#/scene/skeleton/renderer-skeleton-entry";

/**
 * The skeletons, motions and poses a consumer put, by key. A skeleton put again or released stays posed and advanced
 * until the change rebuilding its users lets it go, so they never draw a skeleton frozen or gone.
 */
export class RendererSkeletons {
  private readonly skeletons: Map<string, RendererSkeletonEntry> = new Map();
  /** Skeletons put again or released that objects may still draw, by the key they were put under. */
  private readonly retiring: Map<RendererSkeletonEntry, string> = new Map();
  private readonly motions: Map<string, IRendererMotion> = new Map();
  private readonly poses: Map<string, IRendererPose> = new Map();
  private readonly onReplaced: (key: string, release: Nullable<() => void>) => void;

  /**
   * @param onReplaced - Told when a key's skeleton is put or released, so whatever binds to it can rebind, with what
   *   lets the skeleton it replaced go once nothing draws it, or null for none.
   */
  public constructor(onReplaced: (key: string, release: Nullable<() => void>) => void) {
    this.onReplaced = onReplaced;
  }

  public get(key: string): Maybe<RendererSkeletonEntry> {
    return this.skeletons.get(key);
  }

  public putSkeleton(key: string, skeleton: IRendererSkeleton): void {
    const replaced: Maybe<RendererSkeletonEntry> = this.skeletons.get(key);

    this.skeletons.set(key, new RendererSkeletonEntry(skeleton));
    this.apply(key);
    this.replace(key, replaced);
  }

  public releaseSkeleton(key: string): void {
    const replaced: Maybe<RendererSkeletonEntry> = this.skeletons.get(key);

    this.skeletons.delete(key);
    this.poses.delete(key);
    this.replace(key, replaced);
  }

  public putMotion(key: string, motion: IRendererMotion): void {
    this.motions.set(key, motion);
    this.reposeNaming(key);
  }

  public releaseMotion(key: string): void {
    this.motions.delete(key);
    this.reposeNaming(key);
  }

  public pose(key: string, pose: IRendererPose): void {
    this.poses.set(key, pose);
    this.apply(key);
  }

  /** Keeps every skeleton's bone matrices of the frame before, once each drawn frame, before it draws. */
  public advance(): void {
    this.skeletons.forEach((entry: RendererSkeletonEntry) => entry.advance());
    this.retiring.forEach((_: string, entry: RendererSkeletonEntry) => entry.advance());
  }

  public dispose(): void {
    this.skeletons.forEach((entry: RendererSkeletonEntry) => entry.dispose());
    this.retiring.forEach((_: string, entry: RendererSkeletonEntry) => entry.dispose());
    this.skeletons.clear();
    this.retiring.clear();
    this.motions.clear();
    this.poses.clear();
  }

  private replace(key: string, replaced: Maybe<RendererSkeletonEntry>): void {
    if (replaced) {
      this.retiring.set(replaced, key);
    }

    this.onReplaced(key, replaced ? () => this.retire(replaced) : null);
  }

  private retire(entry: RendererSkeletonEntry): void {
    if (this.retiring.delete(entry)) {
      entry.dispose();
    }
  }

  private reposeNaming(motion: string): void {
    this.poses.forEach((pose: IRendererPose, key: string) => {
      if (pose.motion === motion) {
        this.apply(key);
      }
    });
  }

  /** Poses a key's skeleton, and whichever it replaced that still draws, as it is posed now. */
  private apply(key: string): void {
    const pose: IRendererPose = this.poses.get(key) ?? BIND_POSE;
    const motion: Nullable<IRendererMotion> = pose.motion ? (this.motions.get(pose.motion) ?? null) : null;

    this.skeletons.get(key)?.pose(motion, pose);
    this.retiring.forEach((retiring: string, entry: RendererSkeletonEntry) => {
      if (retiring === key) {
        entry.pose(motion, pose);
      }
    });
  }
}
