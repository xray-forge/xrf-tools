import { Maybe, Nullable } from "@xrf/types";
import { BufferAttribute, BufferGeometry, LineBasicNodeMaterial, LineSegments } from "three/webgpu";

import { ERendererOverlay, TRendererOverlay } from "#/contract/scene/renderer-overlay";
import { disposeOverlayObjects, IOverlayDrawing } from "#/scene/overlay/overlay-drawing";
import { createOverlayLines } from "#/scene/overlay/overlay-lines";
import { RendererSkeletonEntry } from "#/scene/skeleton/renderer-skeleton-entry";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";

/**
 * A skeleton's bones as segments, copied from its pose whenever the pose moved or the skeleton was put again.
 */
export class SkeletonOverlay implements IOverlayDrawing {
  public readonly objects: ReadonlyArray<LineSegments>;

  private readonly lines: LineSegments;
  private readonly skeletons: RendererSkeletons;
  private readonly skeleton: string;
  /** The skeleton and the pose its segments were copied from. */
  private copied: Nullable<RendererSkeletonEntry> = null;
  private version: number = -1;

  public constructor(
    overlay: Extract<TRendererOverlay, { kind: ERendererOverlay.SKELETON }>,
    skeletons: RendererSkeletons
  ) {
    this.skeletons = skeletons;
    this.skeleton = overlay.skeleton;
    this.lines = createOverlayLines(new BufferGeometry(), overlay.isDepthTested, false);
    (this.lines.material as LineBasicNodeMaterial).color.setRGB(...overlay.color);
    // Posed every frame, so never measured against a stale bound.
    this.lines.frustumCulled = false;
    this.objects = [this.lines];
  }

  public update(): void {
    const skeleton: Maybe<RendererSkeletonEntry> = this.skeletons.get(this.skeleton);

    this.lines.visible = Boolean(skeleton?.segments);

    if (!skeleton?.segments || (skeleton === this.copied && skeleton.version === this.version)) {
      return;
    }

    const attribute: Maybe<BufferAttribute> = this.lines.geometry.getAttribute("position") as Maybe<BufferAttribute>;

    if (attribute?.array.length === skeleton.segments.length) {
      attribute.array.set(skeleton.segments);
      attribute.needsUpdate = true;
    } else {
      // A new geometry for another count of segments: disposing the old one is what frees its buffer.
      const geometry: BufferGeometry = new BufferGeometry();

      geometry.setAttribute("position", new BufferAttribute(new Float32Array(skeleton.segments), 3));
      this.lines.geometry.dispose();
      this.lines.geometry = geometry;
    }

    this.lines.geometry.computeBoundingSphere();
    this.copied = skeleton;
    this.version = skeleton.version;
  }

  public dispose(): void {
    disposeOverlayObjects(this.objects);
  }
}
