import {
  Bone,
  BufferAttribute,
  BufferGeometry,
  LineBasicMaterial,
  LineSegments,
  Matrix4,
  Object3D,
  Skeleton,
} from "three";

import { FLOATS_PER_BONE, IVisualModelViews, TRANSLATION_OFFSET } from "@/core/visuals/lib/visual-views";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * Where one frame's bone transforms are read from.
 */
interface IPosedFrame {
  source: Float32Array;
  /** Floats one bone occupies in `source`. */
  stride: number;
  /** Where this frame's first bone starts in `source`. */
  base: number;
}

/** How the bind pose overlay is drawn, which is the only thing this needs from the scene's configuration. */
export interface IVisualPreviewSkeletonConfig {
  skeletonColor: number;
}

/**
 * One model's skeleton: the bones skinning binds to, the overlay that draws them, and the pose both are showing.
 */
export class VisualPreviewSkeleton {
  /**
   * Builds a skeleton for a model, or reports that the model carries none.
   *
   * @param model - Model views, whose bind transforms decide whether there is a skeleton at all.
   * @param parent - Scene node the bones and the overlay attach to, so the renderer updates their world matrices.
   * @param config - How the overlay is drawn.
   * @returns The skeleton, or null when the model carries no bind data.
   */
  public static create(
    model: IVisualModelViews,
    parent: Object3D,
    config: IVisualPreviewSkeletonConfig
  ): Nullable<VisualPreviewSkeleton> {
    return model.skeletonBinds ? new VisualPreviewSkeleton(model, parent, config) : null;
  }

  /** One bone per bind transform, in bone order. */
  private readonly bones: Array<Bone> = [];
  private readonly skin: Skeleton;
  private readonly binds: Float32Array;
  private readonly parent: Object3D;
  /** Segment endpoints of the bind pose, absent for a skeleton whose bones never form a drawable segment. */
  private readonly overlay: Nullable<LineSegments<BufferGeometry, LineBasicMaterial>>;
  /** Which two bones each drawn segment joins, in the order the overlay lays them out. */
  private readonly pairs: Nullable<Uint16Array>;

  /** The pose last asked for, so hiding a bone can re-apply it without the caller sending it again. */
  private pose: { transforms: Nullable<Float32Array>; frame: number; floatsPerBone: number } = {
    floatsPerBone: 0,
    frame: 0,
    transforms: null,
  };
  /** Bones collapsed to nothing, by index, already including the descendants of each. */
  private hiddenBones: ReadonlySet<number> = new Set();

  private constructor(model: IVisualModelViews, parent: Object3D, config: IVisualPreviewSkeletonConfig) {
    this.parent = parent;
    this.binds = model.skeletonBinds as Float32Array;
    this.pairs = model.skeletonPairs;

    // The bind pose itself, never a motion or the hidden set: an inverse taken from a collapsed bone would be the
    // inverse of a zero matrix, and every vertex weighted to it would pose to nothing for as long as the model is open.
    const bindFrame: IPosedFrame = { base: 0, source: this.binds, stride: FLOATS_PER_BONE };
    const inverses: Array<Matrix4> = [];

    for (let bone: number = 0; bone < this.binds.length / FLOATS_PER_BONE; bone += 1) {
      const joint: Bone = new Bone();

      joint.matrixAutoUpdate = false;

      this.bones.push(joint);
      this.parent.add(joint);
      this.poseBone(bone, bindFrame);

      inverses.push(joint.matrix.clone().invert());
    }

    this.skin = new Skeleton(this.bones, inverses);
    this.overlay = this.createOverlay(model.skeleton, config);
  }

  /**
   * @returns What a skinned mesh binds to.
   */
  public getSkin(): Skeleton {
    return this.skin;
  }

  /**
   * Shows or hides the bind pose overlay.
   *
   * @param isVisible - Whether the overlay should draw.
   */
  public setOverlayVisible(isVisible: boolean): void {
    if (this.overlay) {
      this.overlay.visible = isVisible;
    }
  }

  /**
   * Poses every bone from one frame of a baked motion, or returns them to the bind pose.
   *
   * @param transforms - Every frame's bone transforms, frame major, or null to show the bind pose again.
   * @param frame - Which frame of that buffer to show.
   * @param floatsPerBone - Floats one bone occupies, as the bake reported it.
   */
  public setPose(transforms: Nullable<Float32Array>, frame: number, floatsPerBone: number): void {
    this.pose = { floatsPerBone, frame, transforms };

    this.applyPose();
  }

  /**
   * Collapses some of the bones, the way the engine hides a part that is not attached.
   *
   * @param bones - Indices of bones to collapse, already including their descendants.
   */
  public setHiddenBones(bones: ReadonlySet<number>): void {
    this.hiddenBones = bones;

    this.applyPose();
  }

  /** Detaches everything from the scene and releases what the renderer uploaded for it. */
  public dispose(): void {
    for (const bone of this.bones) {
      this.parent.remove(bone);
    }

    if (this.overlay) {
      this.parent.remove(this.overlay);
      this.overlay.geometry.dispose();
      this.overlay.material.dispose();
    }

    // Disposing releases the bone texture the renderer uploaded for it, which is one texture per model opened.
    this.skin.dispose();
  }

  /**
   * Builds the overlay, when the model's bones form a segment to draw at all.
   *
   * @param positions - Bind pose segment endpoints, or null when nothing can be drawn.
   * @param config - How the overlay is drawn.
   * @returns The overlay, already attached, or null when there is nothing to draw.
   */
  private createOverlay(
    positions: Nullable<Float32Array>,
    config: IVisualPreviewSkeletonConfig
  ): Nullable<LineSegments<BufferGeometry, LineBasicMaterial>> {
    if (!positions) {
      return null;
    }

    const geometry: BufferGeometry = new BufferGeometry();

    // A copy, not the model's own array: posing writes into this attribute every frame, and the model's bind positions
    // have to survive being posed so the overlay can go back to them.
    geometry.setAttribute("position", new BufferAttribute(new Float32Array(positions), 3));

    const overlay: LineSegments<BufferGeometry, LineBasicMaterial> = new LineSegments(
      geometry,
      new LineBasicMaterial({ color: config.skeletonColor, depthTest: false, transparent: true })
    );

    overlay.renderOrder = 1;
    overlay.visible = false;

    this.parent.add(overlay);

    return overlay;
  }

  /**
   * Writes the current pose into the bones, then collapses the hidden ones.
   */
  private applyPose(): void {
    const boneCount: number = this.binds.length / FLOATS_PER_BONE;
    const frame: IPosedFrame = this.resolveFrame(boneCount);

    for (let bone: number = 0; bone < boneCount; bone += 1) {
      this.poseBone(bone, frame);
    }

    for (const bone of this.hiddenBones) {
      this.hideBone(bone);
    }

    // The overlay keeps drawing the whole skeleton, hidden bones included. It is the only thing left that says where a
    // hidden part sits, and an author turning a scope off is usually asking where the scope was.
    this.poseOverlay(frame);
  }

  /**
   * Decides which buffer this pose reads from.
   *
   * @param boneCount - Bones this skeleton carries, which is what makes a motion frame's stride.
   * @returns The buffer to pose from and how to index it.
   */
  private resolveFrame(boneCount: number): IPosedFrame {
    const { transforms, frame, floatsPerBone } = this.pose;
    const stride: number = boneCount * floatsPerBone;
    const base: number = frame * stride;

    return transforms && floatsPerBone > 0 && transforms.length >= base + stride
      ? { base, source: transforms, stride: floatsPerBone }
      : { base: 0, source: this.binds, stride: FLOATS_PER_BONE };
  }

  /**
   * Collapses one bone to nothing.
   *
   * @param bone - Bone index to collapse.
   */
  private hideBone(bone: number): void {
    const target: Optional<Bone> = this.bones[bone];

    if (!target) {
      return;
    }

    target.matrix.elements.fill(0);
    target.matrix.elements[15] = 1;
    target.matrixWorldNeedsUpdate = true;
  }

  /**
   * Writes one bone's transform out of a frame into its matrix.
   *
   * @param bone - Bone index, which is also its index in the frame.
   * @param frame - Frame to read this bone's transform out of.
   */
  private poseBone(bone: number, frame: IPosedFrame): void {
    const target: Optional<Bone> = this.bones[bone];

    if (!target) {
      return;
    }

    const { source } = frame;
    const offset: number = frame.base + bone * frame.stride;
    const elements: Array<number> = target.matrix.elements;

    elements[0] = source[offset];
    elements[1] = source[offset + 1];
    elements[2] = source[offset + 2];
    elements[4] = source[offset + 3];
    elements[5] = source[offset + 4];
    elements[6] = source[offset + 5];
    elements[8] = source[offset + 6];
    elements[9] = source[offset + 7];
    elements[10] = source[offset + 8];
    elements[12] = source[offset + 9];
    elements[13] = source[offset + 10];
    elements[14] = source[offset + 11];

    target.matrixWorldNeedsUpdate = true;
  }

  /**
   * Moves the overlay's segment endpoints to where the posed bones now are.
   *
   * @param frame - Frame the bones were posed from, read again here for their translations.
   */
  private poseOverlay(frame: IPosedFrame): void {
    if (!this.overlay || !this.pairs) {
      return;
    }

    const { source } = frame;
    const attribute: BufferAttribute = this.overlay.geometry.getAttribute("position") as BufferAttribute;

    for (let segment: number = 0; segment < this.pairs.length / 2; segment += 1) {
      const child: number = frame.base + this.pairs[segment * 2] * frame.stride + TRANSLATION_OFFSET;
      const parent: number = frame.base + this.pairs[segment * 2 + 1] * frame.stride + TRANSLATION_OFFSET;

      attribute.array.set(source.subarray(child, child + 3), segment * 6);
      attribute.array.set(source.subarray(parent, parent + 3), segment * 6 + 3);
    }

    attribute.needsUpdate = true;

    this.overlay.geometry.computeBoundingSphere();
  }
}
