import { Maybe, Nullable } from "@xrf/types";
import { Bone, Matrix4, Skeleton } from "three/webgpu";

import {
  IRendererMotion,
  IRendererPose,
  IRendererSkeleton,
  RENDERER_FLOATS_PER_BONE,
} from "#/contract/scene/renderer-skeleton";

/** Where a bone's translation starts within its twelve floats. */
const TRANSLATION_OFFSET: number = 9;

/** The bind pose, which is what a skeleton stands in before anything poses it. */
const BIND_POSE: IRendererPose = { frame: 0, hiddenBones: [], motion: null };

/**
 * One skeleton: three's, posed from model space transforms, and its segments for the overlay.
 */
export class RendererSkeletonEntry {
  public readonly skeleton: Skeleton;
  /** Six floats a segment, child then parent, where the pose puts them; null without pairs. */
  public readonly segments: Nullable<Float32Array>;
  /** Bumped by every pose, so an overlay can tell its segments moved. */
  public version: number = 0;

  private readonly bones: Array<Bone>;
  private readonly binds: Float32Array;
  private readonly pairs: Nullable<Uint16Array>;

  public constructor({ binds, pairs }: IRendererSkeleton) {
    this.binds = binds;
    this.pairs = pairs ?? null;
    this.segments = pairs ? new Float32Array(pairs.length * 3) : null;
    this.bones = Array.from({ length: binds.length / RENDERER_FLOATS_PER_BONE }, () => {
      const bone: Bone = new Bone();

      // Posed in model space directly: nothing parents these bones, so nothing may recompose their world matrices.
      bone.matrixAutoUpdate = false;
      bone.matrixWorldAutoUpdate = false;

      return bone;
    });

    // The inverses are of the bind pose itself, never a motion or the hidden set: the inverse of a collapsed bone
    // would be the inverse of a zero matrix.
    const inverses: Array<Matrix4> = this.bones.map((_, bone: number) =>
      RendererSkeletonEntry.toMatrix(binds, bone * RENDERER_FLOATS_PER_BONE).invert()
    );

    this.skeleton = new Skeleton(this.bones, inverses);
    this.pose(null, BIND_POSE);
  }

  /**
   * @param motion - The motion the pose names, or null for the bind pose.
   * @param pose - The frame and the hidden bones.
   */
  public pose(motion: Nullable<IRendererMotion>, pose: IRendererPose): void {
    const count: number = this.bones.length;
    const stride: number = motion ? motion.floatsPerBone : RENDERER_FLOATS_PER_BONE;
    const base: number = pose.frame * count * stride;
    // A frame outside the motion shows the bind pose rather than reading past the buffer.
    const isInMotion: boolean =
      Boolean(motion) && stride >= RENDERER_FLOATS_PER_BONE && motion!.transforms.length >= base + count * stride;
    const source: Float32Array = isInMotion ? motion!.transforms : this.binds;
    const offset: number = isInMotion ? base : 0;
    const boneStride: number = isInMotion ? stride : RENDERER_FLOATS_PER_BONE;

    this.bones.forEach((bone: Bone, index: number) => {
      bone.matrix.copy(RendererSkeletonEntry.toMatrix(source, offset + index * boneStride));
      bone.matrixWorld.copy(bone.matrix);
    });

    for (const hidden of pose.hiddenBones) {
      const bone: Maybe<Bone> = this.bones[hidden];

      if (bone) {
        bone.matrixWorld.elements.fill(0);
        bone.matrixWorld.elements[15] = 1;
      }
    }

    this.poseSegments(source, offset, boneStride);
    this.version += 1;
  }

  public dispose(): void {
    this.skeleton.dispose();
  }

  /** The overlay keeps drawing hidden bones: it is the only thing left saying where a hidden part sits. */
  private poseSegments(source: Float32Array, offset: number, stride: number): void {
    if (!this.pairs || !this.segments) {
      return;
    }

    for (let segment = 0; segment < this.pairs.length / 2; segment += 1) {
      const child: number = offset + this.pairs[segment * 2] * stride + TRANSLATION_OFFSET;
      const parent: number = offset + this.pairs[segment * 2 + 1] * stride + TRANSLATION_OFFSET;

      this.segments.set(source.subarray(child, child + 3), segment * 6);
      this.segments.set(source.subarray(parent, parent + 3), segment * 6 + 3);
    }
  }

  /** A bone's 3x4 as a matrix: three rotation columns, then the translation. */
  private static toMatrix(source: Float32Array, at: number): Matrix4 {
    const matrix: Matrix4 = new Matrix4();
    const elements: Array<number> = matrix.elements;

    elements[0] = source[at];
    elements[1] = source[at + 1];
    elements[2] = source[at + 2];
    elements[4] = source[at + 3];
    elements[5] = source[at + 4];
    elements[6] = source[at + 5];
    elements[8] = source[at + 6];
    elements[9] = source[at + 7];
    elements[10] = source[at + 8];
    elements[12] = source[at + 9];
    elements[13] = source[at + 10];
    elements[14] = source[at + 11];

    return matrix;
  }
}

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
