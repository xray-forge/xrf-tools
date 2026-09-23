import { Maybe } from "@xrf/types";
import { float, uv } from "three/tsl";
import {
  BufferAttribute,
  BufferGeometry,
  LineBasicNodeMaterial,
  LineSegments,
  Object3D,
  PerspectiveCamera,
  Scene,
  Sprite,
  SpriteNodeMaterial,
  Vector3,
} from "three/webgpu";

import { ERendererOverlay, TRendererOverlay } from "#/contract/scene/renderer-overlay";
import { RendererSkeletonEntry, RendererSkeletons } from "#/scene/renderer-skeletons";

/** Drawn after every surface, so a helper that ignores depth is never covered by one drawn later. */
const OVERLAY_RENDER_ORDER: number = 1;

/** How far out the sun marker stands, as a share of the far plane: well inside it, and past whatever is near. */
const SUN_REACH: number = 0.5;

/** One overlay as three draws it, and what refreshes it each frame. */
interface IOverlayEntry {
  overlay: TRendererOverlay;
  objects: Array<Object3D>;
  /** The skeleton pose its segments were copied at, for a skeleton overlay. */
  version: number;
}

/**
 * The helpers a consumer put, by key, as the scene the overlay pass draws.
 */
export class RendererOverlays {
  public readonly scene: Scene = new Scene();

  private readonly entries: Map<string, IOverlayEntry> = new Map();
  private readonly skeletons: RendererSkeletons;
  /** The direction sunlight travels, in world space, read each frame. */
  private readonly sunDirection: Vector3;

  public constructor(skeletons: RendererSkeletons, sunDirection: Vector3) {
    this.skeletons = skeletons;
    this.sunDirection = sunDirection;
  }

  public put(key: string, overlay: TRendererOverlay): void {
    this.release(key);

    const entry: IOverlayEntry = { objects: RendererOverlays.create(overlay), overlay, version: -1 };

    entry.objects.forEach((object: Object3D) => this.scene.add(object));
    this.entries.set(key, entry);
  }

  public release(key: string): void {
    const entry: Maybe<IOverlayEntry> = this.entries.get(key);

    if (entry) {
      entry.objects.forEach(RendererOverlays.dispose);
      this.entries.delete(key);
    }
  }

  /**
   * Brings every overlay up to date for the frame about to be drawn.
   *
   * @param camera - The drawing camera, which a point's pixel size is measured against.
   * @param height - The drawing buffer's height, in device pixels.
   */
  public update(camera: PerspectiveCamera, height: number): void {
    // A sprite without attenuation is scaled by its depth, so this scale is pixels over the view's height at depth one.
    const pixel: number = (2 * Math.tan((camera.fov * Math.PI) / 360)) / Math.max(height, 1);

    this.entries.forEach((entry: IOverlayEntry) => {
      const { overlay } = entry;

      if (overlay.kind === ERendererOverlay.POINTS) {
        entry.objects.forEach((sprite: Object3D) => sprite.scale.setScalar(overlay.size * pixel));
      } else if (overlay.kind === ERendererOverlay.SUN) {
        // Against the light's travel from wherever the camera stands, as a sun far enough away does.
        const [sun] = entry.objects;

        sun.position.copy(camera.position).addScaledVector(this.sunDirection, -camera.far * SUN_REACH);
        sun.scale.setScalar(overlay.size * pixel);
      } else if (overlay.kind === ERendererOverlay.SKELETON) {
        this.follow(entry, overlay.skeleton);
      }
    });
  }

  public dispose(): void {
    [...this.entries.keys()].forEach((key: string) => this.release(key));
  }

  /** Copies a skeleton overlay's segments from its pose, when the pose moved. */
  private follow(entry: IOverlayEntry, key: string): void {
    const skeleton: Maybe<RendererSkeletonEntry> = this.skeletons.get(key);
    const lines = entry.objects[0] as LineSegments;

    lines.visible = Boolean(skeleton?.segments);

    if (!skeleton?.segments || skeleton.version === entry.version) {
      return;
    }

    const attribute: Maybe<BufferAttribute> = lines.geometry.getAttribute("position") as Maybe<BufferAttribute>;

    if (attribute?.array.length === skeleton.segments.length) {
      attribute.array.set(skeleton.segments);
      attribute.needsUpdate = true;
    } else {
      lines.geometry.setAttribute("position", new BufferAttribute(new Float32Array(skeleton.segments), 3));
    }

    lines.geometry.computeBoundingSphere();
    entry.version = skeleton.version;
  }

  private static create(overlay: TRendererOverlay): Array<Object3D> {
    switch (overlay.kind) {
      case ERendererOverlay.LINES: {
        const geometry: BufferGeometry = new BufferGeometry();

        geometry.setAttribute("position", new BufferAttribute(overlay.positions, 3));
        geometry.setAttribute("color", new BufferAttribute(overlay.colors, 3));

        return [RendererOverlays.toLines(geometry, overlay.isDepthTested, true)];
      }

      case ERendererOverlay.SKELETON: {
        const lines: LineSegments = RendererOverlays.toLines(new BufferGeometry(), overlay.isDepthTested, false);

        (lines.material as LineBasicNodeMaterial).color.setRGB(...overlay.color);
        // Posed every frame, so never measured against a stale bound.
        lines.frustumCulled = false;

        return [lines];
      }

      case ERendererOverlay.SUN: {
        const material: SpriteNodeMaterial = new SpriteNodeMaterial({ sizeAttenuation: false });
        const sun: Sprite = new Sprite(material);

        material.color.setRGB(...overlay.color);
        // A disc rather than the sprite's square.
        material.opacityNode = uv().sub(0.5).length().lessThan(0.5).select(float(1), float(0));
        material.alphaTest = 0.5;
        material.depthTest = false;
        material.depthWrite = false;
        sun.renderOrder = OVERLAY_RENDER_ORDER;
        // Placed every frame, so never measured against where it last stood.
        sun.frustumCulled = false;

        return [sun];
      }

      case ERendererOverlay.POINTS:
        return Array.from({ length: overlay.positions.length / 3 }, (_, at: number) => {
          const material: SpriteNodeMaterial = new SpriteNodeMaterial({ sizeAttenuation: false });
          const sprite: Sprite = new Sprite(material);

          material.color.setRGB(...overlay.color);
          material.depthTest = overlay.isDepthTested;
          material.depthWrite = false;
          sprite.position.fromArray(overlay.positions, at * 3);
          sprite.renderOrder = OVERLAY_RENDER_ORDER;

          return sprite;
        });
    }
  }

  private static toLines(geometry: BufferGeometry, isDepthTested: boolean, hasColors: boolean): LineSegments {
    const material: LineBasicNodeMaterial = new LineBasicNodeMaterial({ vertexColors: hasColors });

    material.depthTest = isDepthTested;
    material.depthWrite = false;

    const lines: LineSegments = new LineSegments(geometry, material);

    lines.renderOrder = OVERLAY_RENDER_ORDER;

    return lines;
  }

  private static dispose(object: Object3D): void {
    object.removeFromParent();

    if (object instanceof LineSegments || object instanceof Sprite) {
      object.geometry.dispose();
      (object.material as { dispose(): void }).dispose();
    }
  }
}
