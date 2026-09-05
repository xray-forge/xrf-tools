import { Group, Object3D, Texture } from "three";

import { IVisualMeshMaterialOptions, VisualPreviewMeshes } from "@/core/visuals/components/scene/VisualPreviewMeshes";
import {
  IVisualPreviewSkeletonConfig,
  VisualPreviewSkeleton,
} from "@/core/visuals/components/scene/VisualPreviewSkeleton";
import { IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { Nullable } from "@/lib/types/general";

/** What drawing one model needs beyond the model itself. */
export interface IVisualPreviewModelOptions extends IVisualPreviewSkeletonConfig {
  meshColor: number;
  /** Stands in for every texture while the uv checkerboard is on, and outlives any one model. */
  checker: Texture;
  /** How far down each collapse chain to draw on arrival: 0 is full detail, 1 is the coarsest each submesh has. */
  detail: number;
}

/**
 * One model on screen: its meshes, its skeleton, and the node both hang from.
 */
export class VisualPreviewModel {
  /**
   * Builds a model and attaches it.
   *
   * @param views - What the backend packed, already turned into renderer views.
   * @param parent - Scene node the model's own group attaches to.
   * @param options - Colours, the checkerboard, and the detail to draw at.
   * @returns The drawn model.
   */
  public static create(
    views: IVisualModelViews,
    parent: Object3D,
    options: IVisualPreviewModelOptions
  ): VisualPreviewModel {
    return new VisualPreviewModel(views, parent, options);
  }

  /**
   * The node everything this model draws hangs from.
   */
  public readonly root: Group = new Group();

  private readonly parent: Object3D;
  private readonly meshes: VisualPreviewMeshes;
  private readonly skeleton: Nullable<VisualPreviewSkeleton>;

  private constructor(views: IVisualModelViews, parent: Object3D, options: IVisualPreviewModelOptions) {
    this.parent = parent;
    this.parent.add(this.root);

    // The skeleton first, because a skinned mesh binds to its skin as it is built.
    this.skeleton = VisualPreviewSkeleton.create(views, this.root, { skeletonColor: options.skeletonColor });
    this.meshes = VisualPreviewMeshes.create(views, this.root, {
      checker: options.checker,
      detail: options.detail,
      meshColor: options.meshColor,
      skin: this.skeleton?.getSkin() ?? null,
    });
  }

  /**
   * @returns Whether this model animates, which is what makes a pose mean anything.
   */
  public hasSkeleton(): boolean {
    return this.skeleton !== null;
  }

  /**
   * Poses the model from one frame of a baked motion, or returns it to its bind pose.
   *
   * @param transforms - Every frame's bone transforms, frame major, or null to show the bind pose again.
   * @param frame - Which frame of that buffer to show.
   * @param floatsPerBone - Floats one bone occupies, as the bake reported it.
   */
  public setPose(transforms: Nullable<Float32Array>, frame: number, floatsPerBone: number): void {
    this.skeleton?.setPose(transforms, frame, floatsPerBone);
  }

  /**
   * Collapses some of the model's bones, the way the engine hides a part that is not attached.
   *
   * @param bones - Indices of bones to collapse, already including their descendants.
   */
  public setHiddenBones(bones: ReadonlySet<number>): void {
    this.skeleton?.setHiddenBones(bones);
  }

  /**
   * Draws every mesh at a different point along its collapse chain.
   *
   * @param detail - How far down each chain to go: 0 is full detail, 1 is the coarsest each submesh has.
   */
  public setDetailLevel(detail: number): void {
    this.meshes.setDetailLevel(detail);
  }

  /**
   * Puts a loaded texture on one of this model's submeshes.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param texture - Texture to draw with, owned by whoever loaded it.
   */
  public applyTexture(submeshIndex: number, texture: Texture): void {
    this.meshes.applyTexture(submeshIndex, texture);
  }

  /**
   * Shades one of this model's submeshes with its bump pair.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param textures - The uploaded pair, owned by whoever loaded it.
   */
  public applyBump(submeshIndex: number, textures: IVisualBumpTextures): void {
    this.meshes.applyBump(submeshIndex, textures);
  }

  /**
   * Applies the view toggles a model answers to, which are the surface ones and the overlay.
   *
   * @param options - Wireframe and checkerboard for the surfaces, and whether the bind pose overlay draws.
   */
  public applyViewOptions(options: IVisualMeshMaterialOptions & { isSkeletonVisible: boolean }): void {
    this.meshes.applyMaterialOptions(options);
    this.skeleton?.setOverlayVisible(options.isSkeletonVisible);
  }

  /** Detaches the model and frees everything the renderer uploaded for it. */
  public dispose(): void {
    this.meshes.dispose();
    this.skeleton?.dispose();

    this.parent.remove(this.root);
  }
}
