import { Maybe } from "@xrf/types";

import { IRendererMotion, IRendererPose, IRendererSkeleton } from "#/contract/scene/renderer-skeleton";
import { BIND_POSE, RendererSkeletonEntry } from "#/scene/skeleton/renderer-skeleton-entry";

/**
 * The skeletons, motions and poses a consumer put, by key.
 */
export class RendererSkeletons {
  private readonly skeletons: Map<string, RendererSkeletonEntry> = new Map();
  private readonly motions: Map<string, IRendererMotion> = new Map();
  private readonly poses: Map<string, IRendererPose> = new Map();
  private readonly onReplaced: (key: string) => void;

  /**
   * @param onReplaced - Told when a key's skeleton is put or released, so whatever binds to it can rebind.
   */
  public constructor(onReplaced: (key: string) => void) {
    this.onReplaced = onReplaced;
  }

  public get(key: string): Maybe<RendererSkeletonEntry> {
    return this.skeletons.get(key);
  }

  public putSkeleton(key: string, skeleton: IRendererSkeleton): void {
    this.skeletons.get(key)?.dispose();

    const entry: RendererSkeletonEntry = new RendererSkeletonEntry(skeleton);

    this.skeletons.set(key, entry);
    this.apply(key);
    this.onReplaced(key);
  }

  public releaseSkeleton(key: string): void {
    this.skeletons.get(key)?.dispose();
    this.skeletons.delete(key);
    this.poses.delete(key);
    this.onReplaced(key);
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

  public dispose(): void {
    this.skeletons.forEach((entry: RendererSkeletonEntry) => entry.dispose());
    this.skeletons.clear();
    this.motions.clear();
    this.poses.clear();
  }

  private reposeNaming(motion: string): void {
    this.poses.forEach((pose: IRendererPose, key: string) => {
      if (pose.motion === motion) {
        this.apply(key);
      }
    });
  }

  private apply(key: string): void {
    const pose: IRendererPose = this.poses.get(key) ?? BIND_POSE;

    this.skeletons.get(key)?.pose(pose.motion ? (this.motions.get(pose.motion) ?? null) : null, pose);
  }
}
